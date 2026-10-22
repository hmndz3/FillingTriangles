//! Rasterizacion de triangulos.
//!
//! Un triangulo son siempre los mismos tres vertices; lo que cambia entre
//! modos es como se generan los fragmentos.

use crate::barycentric::{barycentric, inside};
use crate::bounds::pixel_bounds;
use crate::color::Color;
use crate::fragment::Fragment;
use crate::line::line;
use crate::vertex::{cross2, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriangleMode {
    /// Solo el contorno: tres llamadas a `line`.
    Wireframe,
    /// Relleno evaluando coordenadas baricentricas en el rectangulo contenedor.
    FilledBarycentric,
    /// Relleno avanzando una fila de la imagen a la vez.
    FilledScanline,
}

impl TriangleMode {
    pub const ALL: [TriangleMode; 3] = [
        TriangleMode::Wireframe,
        TriangleMode::FilledBarycentric,
        TriangleMode::FilledScanline,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            TriangleMode::Wireframe => "wireframe",
            TriangleMode::FilledBarycentric => "barycentric",
            TriangleMode::FilledScanline => "scanline",
        }
    }
}

/// Genera los fragmentos del triangulo a-b-c segun el modo pedido.
///
/// Cambiar de algoritmo es cambiar este argumento: el ensamblaje de primitivas
/// y el procesamiento de fragmentos no se enteran.
pub fn triangle(a: Vec2, b: Vec2, c: Vec2, mode: TriangleMode, color: Color) -> Vec<Fragment> {
    match mode {
        TriangleMode::Wireframe => wireframe(a, b, c, color),
        TriangleMode::FilledBarycentric => fill_barycentric(a, b, c, color),
        TriangleMode::FilledScanline => fill_scanline(a, b, c, color),
    }
}

/// Contorno del triangulo: las tres aristas como lineas.
pub fn wireframe(a: Vec2, b: Vec2, c: Vec2, color: Color) -> Vec<Fragment> {
    let mut fragments = line(a, b, color);
    fragments.extend(line(b, c, color));
    fragments.extend(line(c, a, color));
    fragments
}

/// Relleno con coordenadas baricentricas: se recorre el rectangulo contenedor
/// y se prueba el centro de cada pixel.
///
/// Un triangulo de area cero no genera fragmentos: `barycentric` devuelve
/// `None` y se trata como caso degenerado en lugar de dividir entre cero.
pub fn fill_barycentric(a: Vec2, b: Vec2, c: Vec2, color: Color) -> Vec<Fragment> {
    let Some(bounds) = pixel_bounds(&[a, b, c]) else {
        return Vec::new();
    };

    if cross2(a, b, c) == 0.0 {
        // Caso degenerado: se descarta antes de reservar memoria para el
        // rectangulo contenedor, que puede ser grande aunque el area sea cero.
        return Vec::new();
    }

    // Un triangulo ocupa aproximadamente la mitad de su rectangulo contenedor.
    let mut fragments = Vec::with_capacity(bounds.pixel_count() / 2 + 1);
    for y in bounds.rows() {
        for x in bounds.columns() {
            let centro = Vec2::pixel_center(x, y);
            // El area ya se verifico, asi que aqui siempre hay coordenadas.
            let Some(pesos) = barycentric(a, b, c, centro) else {
                return Vec::new();
            };
            if inside(pesos) {
                fragments.push(Fragment::new(x, y, color));
            }
        }
    }
    fragments
}

/// Relleno por lineas horizontales: se avanza una fila de la imagen a la vez y
/// se buscan los dos extremos del tramo interior en esa fila.
///
/// En lugar de recorrer la fila pixel por pixel, cada arista se despeja como
/// una desigualdad en x. La arista que sube acota el tramo por la izquierda y
/// la que baja lo acota por la derecha; el tramo es la interseccion de las tres
/// restricciones.
pub fn fill_scanline(a: Vec2, b: Vec2, c: Vec2, color: Color) -> Vec<Fragment> {
    let Some(bounds) = pixel_bounds(&[a, b, c]) else {
        return Vec::new();
    };

    let area2 = cross2(a, b, c);
    if area2 == 0.0 {
        // Caso degenerado: los tres vertices estan en una linea.
        return Vec::new();
    }
    // Con este signo las tres desigualdades apuntan al interior sin importar el
    // orden en que vengan los vertices.
    let signo: f64 = if area2 > 0.0 { 1.0 } else { -1.0 };
    let aristas = [(a, b), (b, c), (c, a)];

    let mut fragments = Vec::with_capacity(bounds.pixel_count() / 2 + 1);
    for y in bounds.rows() {
        let fila = y as f64 + 0.5;
        let mut desde = bounds.min_x as f64 + 0.5;
        let mut hasta = bounds.max_x as f64 + 0.5;
        let mut fila_vacia = false;

        for (p, q) in aristas {
            // signo * cross2(p, q, (x, fila)) = pendiente * x + termino >= 0
            let (px, py) = (p.x as f64, p.y as f64);
            let (qx, qy) = (q.x as f64, q.y as f64);
            let pendiente = -signo * (qy - py);
            let termino = signo * ((qx - px) * (fila - py) + (qy - py) * px);

            if pendiente > 0.0 {
                // La arista cruza la fila por la izquierda del tramo.
                desde = desde.max(-termino / pendiente);
            } else if pendiente < 0.0 {
                // La arista cruza la fila por la derecha del tramo.
                hasta = hasta.min(-termino / pendiente);
            } else if termino < 0.0 {
                // Arista horizontal que deja esta fila completamente afuera.
                fila_vacia = true;
                break;
            }
        }

        if fila_vacia {
            continue;
        }

        // Se generan los pixeles cuyo centro cae dentro del tramo.
        let inicio = (desde - 0.5).ceil() as i32;
        let fin = (hasta - 0.5).floor() as i32;
        for x in inicio..=fin {
            fragments.push(Fragment::new(x, y, color));
        }
    }
    fragments
}

#[cfg(test)]
mod tests {
    use super::{fill_barycentric, fill_scanline, triangle, wireframe, TriangleMode};
    use crate::color::Color;
    use crate::vertex::Vec2;
    use std::collections::HashSet;

    #[test]
    fn el_contorno_recorre_las_tres_aristas() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(4.0, 0.0);
        let c = Vec2::new(0.0, 4.0);
        let posiciones: HashSet<(i32, i32)> = wireframe(a, b, c, Color::WHITE)
            .iter()
            .map(|f| f.position())
            .collect();

        // Los vertices y el centro de cada arista estan en el contorno.
        for p in [(0, 0), (4, 0), (0, 4), (2, 0), (0, 2), (2, 2)] {
            assert!(posiciones.contains(&p), "falta {p:?}");
        }
        // El interior no se dibuja en este modo.
        assert!(!posiciones.contains(&(1, 1)));
    }

    fn posiciones_baricentricas(a: Vec2, b: Vec2, c: Vec2) -> HashSet<(i32, i32)> {
        fill_barycentric(a, b, c, Color::WHITE)
            .iter()
            .map(|f| f.position())
            .collect()
    }

    #[test]
    fn el_relleno_cubre_el_interior() {
        let p = posiciones_baricentricas(
            Vec2::new(0.0, 0.0),
            Vec2::new(6.0, 0.0),
            Vec2::new(0.0, 6.0),
        );
        assert!(p.contains(&(0, 0)));
        assert!(p.contains(&(1, 1)));
        assert!(p.contains(&(2, 2)));
        // Fuera de la hipotenusa no hay fragmentos.
        assert!(!p.contains(&(5, 5)));
        assert!(!p.contains(&(3, 3)));
    }

    #[test]
    fn invertir_el_orden_de_los_vertices_no_borra_el_triangulo() {
        let a = Vec2::new(1.0, 1.0);
        let b = Vec2::new(9.0, 2.0);
        let c = Vec2::new(4.0, 8.0);
        let directo = posiciones_baricentricas(a, b, c);
        let invertido = posiciones_baricentricas(c, b, a);
        assert!(!directo.is_empty());
        assert_eq!(directo, invertido);
    }

    #[test]
    fn tres_vertices_en_una_linea_no_generan_fragmentos() {
        let p = posiciones_baricentricas(
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(10.0, 10.0),
        );
        assert!(p.is_empty());
    }

    fn posiciones_scanline(a: Vec2, b: Vec2, c: Vec2) -> HashSet<(i32, i32)> {
        fill_scanline(a, b, c, Color::WHITE)
            .iter()
            .map(|f| f.position())
            .collect()
    }

    #[test]
    fn el_scanline_cubre_el_mismo_interior() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(6.0, 0.0);
        let c = Vec2::new(0.0, 6.0);
        assert_eq!(
            posiciones_scanline(a, b, c),
            posiciones_baricentricas(a, b, c)
        );
    }

    #[test]
    fn el_scanline_no_depende_del_orden_de_los_vertices() {
        let a = Vec2::new(1.0, 1.0);
        let b = Vec2::new(9.0, 2.0);
        let c = Vec2::new(4.0, 8.0);
        let directo = posiciones_scanline(a, b, c);
        assert!(!directo.is_empty());
        assert_eq!(directo, posiciones_scanline(c, b, a));
    }

    #[test]
    fn el_scanline_descarta_el_caso_degenerado() {
        let p = posiciones_scanline(
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(10.0, 10.0),
        );
        assert!(p.is_empty());
    }

    #[test]
    fn el_scanline_no_duplica_fragmentos() {
        let fragmentos = fill_scanline(
            Vec2::new(2.0, 1.0),
            Vec2::new(11.0, 4.0),
            Vec2::new(5.0, 9.0),
            Color::WHITE,
        );
        let unicos: HashSet<(i32, i32)> = fragmentos.iter().map(|f| f.position()).collect();
        assert_eq!(fragmentos.len(), unicos.len());
    }

    /// Casos donde un vertice o una arista cae justo sobre la fila que se esta
    /// recorriendo. Las dos implementaciones usan la misma regla (el centro del
    /// pixel cuenta como dentro cuando esta sobre la arista), asi que tienen que
    /// generar exactamente las mismas posiciones.
    #[test]
    fn las_aristas_horizontales_siguen_la_misma_regla() {
        let casos: [[Vec2; 3]; 6] = [
            // Arista horizontal abajo, sobre el borde de los pixeles.
            [
                Vec2::new(2.0, 2.0),
                Vec2::new(10.0, 2.0),
                Vec2::new(6.0, 9.0),
            ],
            // Arista horizontal justo sobre los centros de una fila.
            [
                Vec2::new(2.0, 2.5),
                Vec2::new(10.0, 2.5),
                Vec2::new(6.0, 9.5),
            ],
            // Arista horizontal arriba.
            [
                Vec2::new(3.0, 11.0),
                Vec2::new(12.0, 11.0),
                Vec2::new(5.0, 3.0),
            ],
            // Arista vertical a la izquierda.
            [
                Vec2::new(4.0, 1.0),
                Vec2::new(4.0, 10.0),
                Vec2::new(13.0, 6.0),
            ],
            // Vertice con la coordenada y en el centro exacto de una fila.
            [
                Vec2::new(1.5, 4.5),
                Vec2::new(9.5, 4.5),
                Vec2::new(5.5, 12.5),
            ],
            // Triangulo muy delgado: una fila puede quedarse sin centros.
            [
                Vec2::new(1.0, 1.0),
                Vec2::new(14.0, 1.4),
                Vec2::new(14.0, 1.6),
            ],
        ];

        for [a, b, c] in casos {
            assert_eq!(
                posiciones_scanline(a, b, c),
                posiciones_baricentricas(a, b, c),
                "difieren para {a:?} {b:?} {c:?}"
            );
        }
    }

    #[test]
    fn el_modo_elige_el_algoritmo() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(8.0, 0.0);
        let c = Vec2::new(0.0, 8.0);
        let contorno = triangle(a, b, c, TriangleMode::Wireframe, Color::WHITE);
        let relleno = triangle(a, b, c, TriangleMode::FilledBarycentric, Color::WHITE);
        let filas = triangle(a, b, c, TriangleMode::FilledScanline, Color::WHITE);

        assert!(contorno.len() < relleno.len());
        assert_eq!(relleno.len(), filas.len());
    }

    #[test]
    fn cada_modo_tiene_nombre() {
        assert_eq!(TriangleMode::ALL.len(), 3);
        assert_eq!(TriangleMode::Wireframe.name(), "wireframe");
    }
}
