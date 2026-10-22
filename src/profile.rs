//! Instrumentacion para separar en que se va el tiempo de cada relleno.
//!
//! Las funciones de aqui repiten la geometria de `triangle.rs` pero en vez de
//! crear fragmentos solo los cuentan. Comparando su tiempo con el de
//! `rasterize` se ve cuanto cuesta la geometria y cuanto cuesta crear y guardar
//! los fragmentos.

use crate::barycentric::{barycentric, inside};
use crate::bounds::pixel_bounds;
use crate::framebuffer::Framebuffer;
use crate::render::{process_fragments, rasterize, Triangle};
use crate::scene::new_framebuffer;
use crate::triangle::TriangleMode;
use crate::vertex::{cross2, Vec2};
use std::hint::black_box;
use std::time::{Duration, Instant};

/// Recorre el rectangulo contenedor probando cada centro, sin generar nada.
pub fn count_barycentric(a: Vec2, b: Vec2, c: Vec2) -> usize {
    let Some(bounds) = pixel_bounds(&[a, b, c]) else {
        return 0;
    };
    let mut total = 0;
    for y in bounds.rows() {
        for x in bounds.columns() {
            let centro = Vec2::pixel_center(x, y);
            match barycentric(a, b, c, centro) {
                Some(pesos) if inside(pesos) => total += 1,
                Some(_) => {}
                None => return 0,
            }
        }
    }
    total
}

/// Calcula los tramos fila por fila y suma su largo, sin generar nada.
pub fn count_scanline(a: Vec2, b: Vec2, c: Vec2) -> usize {
    let Some(bounds) = pixel_bounds(&[a, b, c]) else {
        return 0;
    };
    let area2 = cross2(a, b, c);
    if area2 == 0.0 {
        return 0;
    }
    let signo: f64 = if area2 > 0.0 { 1.0 } else { -1.0 };
    let aristas = [(a, b), (b, c), (c, a)];

    let mut total = 0;
    for y in bounds.rows() {
        let fila = y as f64 + 0.5;
        let mut desde = bounds.min_x as f64 + 0.5;
        let mut hasta = bounds.max_x as f64 + 0.5;
        let mut fila_vacia = false;

        for (p, q) in aristas {
            let (px, py) = (p.x as f64, p.y as f64);
            let (qx, qy) = (q.x as f64, q.y as f64);
            let pendiente = -signo * (qy - py);
            let termino = signo * ((qx - px) * (fila - py) + (qy - py) * px);

            if pendiente > 0.0 {
                desde = desde.max(-termino / pendiente);
            } else if pendiente < 0.0 {
                hasta = hasta.min(-termino / pendiente);
            } else if termino < 0.0 {
                fila_vacia = true;
                break;
            }
        }

        if fila_vacia {
            continue;
        }
        let inicio = (desde - 0.5).ceil() as i32;
        let fin = (hasta - 0.5).floor() as i32;
        if fin >= inicio {
            total += (fin - inicio + 1) as usize;
        }
    }
    total
}

/// Recorre las tres aristas con Bresenham contando pixeles, sin generarlos.
pub fn count_wireframe(a: Vec2, b: Vec2, c: Vec2) -> usize {
    count_line(a, b) + count_line(b, c) + count_line(c, a)
}

fn count_line(a: Vec2, b: Vec2) -> usize {
    let p = (a.x.floor() as i32, a.y.floor() as i32);
    let q = (b.x.floor() as i32, b.y.floor() as i32);
    // Bresenham visita un pixel por paso en el eje mas largo, mas el inicial.
    let dx = (q.0 - p.0).abs();
    let dy = (q.1 - p.1).abs();
    (dx.max(dy) + 1) as usize
}

/// La version que solo cuenta, para el modo pedido.
pub fn count_for(mode: TriangleMode) -> fn(Vec2, Vec2, Vec2) -> usize {
    match mode {
        TriangleMode::Wireframe => count_wireframe,
        TriangleMode::FilledBarycentric => count_barycentric,
        TriangleMode::FilledScanline => count_scanline,
    }
}

/// Reparto del tiempo de un cuadro entre sus fases.
pub struct PhaseProfile {
    pub mode: TriangleMode,
    /// Recorrer la geometria decidiendo que pixeles entran.
    pub geometry: Duration,
    /// Crear y guardar los fragmentos: la diferencia contra la rasterizacion
    /// completa.
    pub fragments: Duration,
    /// Limpiar el framebuffer antes de dibujar.
    pub clear: Duration,
    /// Escribir los fragmentos en el framebuffer.
    pub paint: Duration,
    pub fragment_count: usize,
}

impl PhaseProfile {
    pub fn rasterize_total(&self) -> Duration {
        self.geometry + self.fragments
    }

    pub fn frame_total(&self) -> Duration {
        self.rasterize_total() + self.clear + self.paint
    }
}

/// Mediana de `repeats` tandas de `iterations` corridas de `f`.
fn median<F: FnMut()>(iterations: usize, repeats: usize, mut f: F) -> Duration {
    let mut tandas = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let inicio = Instant::now();
        for _ in 0..iterations {
            f();
        }
        tandas.push(inicio.elapsed() / iterations as u32);
    }
    tandas.sort_unstable();
    tandas[tandas.len() / 2]
}

/// Mide cada fase por separado comparando variantes del mismo trabajo.
///
/// No es un perfilador de muestreo: en vez de interrumpir el programa, se corre
/// la geometria sin crear fragmentos y se le resta a la rasterizacion completa.
/// Medir con un reloj dentro del ciclo de pixeles costaria mas que el propio
/// trabajo que se quiere medir.
pub fn measure_phases(
    triangles: &[Triangle],
    mode: TriangleMode,
    iterations: usize,
    repeats: usize,
) -> PhaseProfile {
    let contar = count_for(mode);
    let fragments = rasterize(triangles, mode);
    let fragment_count = fragments.len();

    let geometry = median(iterations, repeats, || {
        let mut total = 0;
        for [a, b, c] in black_box(triangles) {
            total += contar(a.position, b.position, c.position);
        }
        black_box(total);
    });

    let completo = median(iterations, repeats, || {
        black_box(rasterize(black_box(triangles), black_box(mode)));
    });

    let mut framebuffer: Framebuffer = new_framebuffer();
    // Se pasa el buffer entero por `black_box`: con solo mirar su largo, el
    // optimizador borraba la escritura de los pixeles y las dos fases median
    // cero.
    let clear = median(iterations, repeats, || {
        framebuffer.clear();
        black_box(framebuffer.buffer());
    });
    let paint = median(iterations, repeats, || {
        process_fragments(&mut framebuffer, black_box(&fragments));
        black_box(framebuffer.buffer());
    });

    PhaseProfile {
        mode,
        geometry,
        // La resta puede dar cero si la geometria se optimiza casi por
        // completo; saturating_sub evita un panico en ese caso.
        fragments: completo.saturating_sub(geometry),
        clear,
        paint,
        fragment_count,
    }
}

/// Convierte el reparto de fases al formato de pilas plegadas que entienden los
/// visores de flamegraph: una linea por pila y, como cantidad de muestras, los
/// nanosegundos medidos.
pub fn folded_stacks(profiles: &[PhaseProfile]) -> String {
    let mut salida = String::new();
    for p in profiles {
        let modo = p.mode.name();
        for (pila, tiempo) in [
            (format!("{modo};rasterize;geometria"), p.geometry),
            (format!("{modo};rasterize;crear_fragmentos"), p.fragments),
            (format!("{modo};cuadro;limpiar_framebuffer"), p.clear),
            (format!("{modo};cuadro;pintar_fragmentos"), p.paint),
        ] {
            let nanos = tiempo.as_nanos();
            if nanos > 0 {
                salida.push_str(&format!(
                    "{pila} {nanos}
"
                ));
            }
        }
    }
    salida
}

#[cfg(test)]
mod tests {
    use super::{
        count_barycentric, count_scanline, count_wireframe, folded_stacks, measure_phases,
    };
    use crate::color::Color;
    use crate::scene::demo_triangles;
    use crate::scene::test_cases;
    use crate::triangle::TriangleMode;
    use crate::triangle::{fill_barycentric, fill_scanline, wireframe};

    /// La instrumentacion tiene que contar exactamente los mismos fragmentos
    /// que generan los rellenos de verdad.
    #[test]
    fn el_conteo_coincide_con_el_relleno() {
        for caso in test_cases() {
            let [a, b, c] = caso.triangle;
            let (pa, pb, pc) = (a.position, b.position, c.position);
            assert_eq!(
                count_barycentric(pa, pb, pc),
                fill_barycentric(pa, pb, pc, Color::WHITE).len(),
                "{}",
                caso.name
            );
            assert_eq!(
                count_scanline(pa, pb, pc),
                fill_scanline(pa, pb, pc, Color::WHITE).len(),
                "{}",
                caso.name
            );
            assert_eq!(
                count_wireframe(pa, pb, pc),
                wireframe(pa, pb, pc, Color::WHITE).len(),
                "{}",
                caso.name
            );
        }
    }

    #[test]
    fn el_reparto_de_fases_cubre_el_cuadro_completo() {
        let triangulos = demo_triangles();
        let p = measure_phases(&triangulos, TriangleMode::FilledScanline, 2, 3);
        assert!(p.fragment_count > 0);
        assert!(p.frame_total() >= p.rasterize_total());
        assert!(p.paint > std::time::Duration::ZERO);
    }

    #[test]
    fn las_pilas_plegadas_tienen_el_formato_esperado() {
        let triangulos = demo_triangles();
        let p = measure_phases(&triangulos, TriangleMode::FilledBarycentric, 2, 3);
        let texto = folded_stacks(&[p]);
        assert!(texto.contains("barycentric;rasterize;"));
        for linea in texto.lines() {
            let (pila, muestras) = linea.rsplit_once(' ').expect("pila y muestras");
            assert!(pila.contains(';'));
            assert!(muestras.parse::<u128>().is_ok());
        }
    }
}
