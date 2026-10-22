//! Rasterizacion de lineas con el algoritmo de Bresenham.

use crate::color::Color;
use crate::fragment::Fragment;
use crate::vertex::Vec2;

/// Genera los fragmentos de la linea que une `a` con `b`, ambos extremos
/// incluidos. Las posiciones flotantes se redondean al pixel que las contiene.
///
/// Bresenham rompe los empates siempre hacia el mismo lado, asi que recorrer la
/// linea al reves puede devolver otro conjunto de pixeles. Para que el contorno
/// de un triangulo no dependa del orden de sus vertices, los extremos se
/// ordenan antes de recorrerlos.
pub fn line(a: Vec2, b: Vec2, color: Color) -> Vec<Fragment> {
    let (inicio, fin) = ordenar(a, b);

    let mut x0 = inicio.0;
    let mut y0 = inicio.1;
    let x1 = fin.0;
    let y1 = fin.1;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    let mut fragments = Vec::with_capacity((dx.max(-dy) + 1) as usize);
    loop {
        fragments.push(Fragment::new(x0, y0, color));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let double_error = 2 * error;
        if double_error >= dy {
            error += dy;
            x0 += sx;
        }
        if double_error <= dx {
            error += dx;
            y0 += sy;
        }
    }
    fragments
}

/// Devuelve los dos extremos en pixeles enteros, siempre en el mismo orden.
fn ordenar(a: Vec2, b: Vec2) -> ((i32, i32), (i32, i32)) {
    let p = (a.x.floor() as i32, a.y.floor() as i32);
    let q = (b.x.floor() as i32, b.y.floor() as i32);
    if (p.1, p.0) <= (q.1, q.0) {
        (p, q)
    } else {
        (q, p)
    }
}

#[cfg(test)]
mod tests {
    use super::line;
    use crate::color::Color;
    use crate::vertex::Vec2;

    fn posiciones(a: Vec2, b: Vec2) -> Vec<(i32, i32)> {
        line(a, b, Color::WHITE)
            .iter()
            .map(|f| f.position())
            .collect()
    }

    #[test]
    fn linea_horizontal() {
        let p = posiciones(Vec2::new(2.0, 1.0), Vec2::new(5.0, 1.0));
        assert_eq!(p, vec![(2, 1), (3, 1), (4, 1), (5, 1)]);
    }

    #[test]
    fn linea_vertical() {
        let p = posiciones(Vec2::new(2.0, 1.0), Vec2::new(2.0, 4.0));
        assert_eq!(p, vec![(2, 1), (2, 2), (2, 3), (2, 4)]);
    }

    #[test]
    fn diagonal_de_45_grados() {
        let p = posiciones(Vec2::new(0.0, 0.0), Vec2::new(3.0, 3.0));
        assert_eq!(p, vec![(0, 0), (1, 1), (2, 2), (3, 3)]);
    }

    #[test]
    fn invertir_los_extremos_da_el_mismo_conjunto() {
        let a = Vec2::new(-3.0, 2.0);
        let b = Vec2::new(9.0, 7.0);
        let mut ida = posiciones(a, b);
        let mut vuelta = posiciones(b, a);
        ida.sort_unstable();
        vuelta.sort_unstable();
        assert_eq!(ida, vuelta);
    }

    #[test]
    fn un_punto_cuando_los_extremos_coinciden() {
        let p = posiciones(Vec2::new(7.0, 9.0), Vec2::new(7.0, 9.0));
        assert_eq!(p, vec![(7, 9)]);
    }
}
