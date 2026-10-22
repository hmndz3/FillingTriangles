//! Rasterizacion de lineas con el algoritmo de Bresenham.

use crate::color::Color;
use crate::fragment::Fragment;
use crate::vertex::Vec2;

/// Genera los fragmentos de la linea que une `a` con `b`, ambos extremos
/// incluidos. Las posiciones flotantes se redondean al pixel que las contiene.
pub fn line(a: Vec2, b: Vec2, color: Color) -> Vec<Fragment> {
    let mut x0 = a.x.floor() as i32;
    let mut y0 = a.y.floor() as i32;
    let x1 = b.x.floor() as i32;
    let y1 = b.y.floor() as i32;

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

#[cfg(test)]
mod tests {
    use super::line;
    use crate::color::Color;
    use crate::vertex::Vec2;

    fn posiciones(a: Vec2, b: Vec2) -> Vec<(i32, i32)> {
        line(a, b, Color::WHITE).iter().map(|f| f.position()).collect()
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
    fn un_punto_cuando_los_extremos_coinciden() {
        let p = posiciones(Vec2::new(7.0, 9.0), Vec2::new(7.0, 9.0));
        assert_eq!(p, vec![(7, 9)]);
    }

}
