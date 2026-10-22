//! Instrumentacion para separar en que se va el tiempo de cada relleno.
//!
//! Las funciones de aqui repiten la geometria de `triangle.rs` pero en vez de
//! crear fragmentos solo los cuentan. Comparando su tiempo con el de
//! `rasterize` se ve cuanto cuesta la geometria y cuanto cuesta crear y guardar
//! los fragmentos.

use crate::barycentric::{barycentric, inside};
use crate::bounds::pixel_bounds;
use crate::vertex::{cross2, Vec2};

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

#[cfg(test)]
mod tests {
    use super::{count_barycentric, count_scanline};
    use crate::color::Color;
    use crate::scene::test_cases;
    use crate::triangle::{fill_barycentric, fill_scanline};

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
        }
    }
}
