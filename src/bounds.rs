//! Rectangulo contenedor de un triangulo, medido en pixeles enteros.

use crate::vertex::Vec2;

/// Rango de pixeles cuyo centro puede caer dentro del triangulo. Los limites
/// son inclusivos.
///
/// No se recorta contra el framebuffer a proposito: los dos algoritmos de
/// relleno generan los mismos fragmentos aunque el triangulo se salga de la
/// imagen, y el recorte ocurre al pintar (`Framebuffer::point`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelBounds {
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
}

impl PixelBounds {
    pub fn rows(&self) -> std::ops::RangeInclusive<i32> {
        self.min_y..=self.max_y
    }

    pub fn columns(&self) -> std::ops::RangeInclusive<i32> {
        self.min_x..=self.max_x
    }

    pub fn pixel_count(&self) -> usize {
        let ancho = (self.max_x - self.min_x + 1).max(0) as usize;
        let alto = (self.max_y - self.min_y + 1).max(0) as usize;
        ancho * alto
    }
}

/// El pixel entero `x` entra en el rango si su centro `x + 0.5` cae dentro del
/// intervalo que ocupan los vertices. De ahi salen el `ceil` y el `floor`.
pub fn pixel_bounds(points: &[Vec2; 3]) -> Option<PixelBounds> {
    let min_x = points.iter().fold(f32::INFINITY, |m, p| m.min(p.x));
    let max_x = points.iter().fold(f32::NEG_INFINITY, |m, p| m.max(p.x));
    let min_y = points.iter().fold(f32::INFINITY, |m, p| m.min(p.y));
    let max_y = points.iter().fold(f32::NEG_INFINITY, |m, p| m.max(p.y));

    if !(min_x.is_finite() && max_x.is_finite() && min_y.is_finite() && max_y.is_finite()) {
        return None;
    }

    let bounds = PixelBounds {
        min_x: (min_x - 0.5).ceil() as i32,
        max_x: (max_x - 0.5).floor() as i32,
        min_y: (min_y - 0.5).ceil() as i32,
        max_y: (max_y - 0.5).floor() as i32,
    };

    if bounds.min_x > bounds.max_x || bounds.min_y > bounds.max_y {
        return None;
    }
    Some(bounds)
}

#[cfg(test)]
mod tests {
    use super::{pixel_bounds, PixelBounds};
    use crate::vertex::Vec2;

    #[test]
    fn cubre_los_centros_de_pixel_del_triangulo() {
        let b = pixel_bounds(&[
            Vec2::new(1.0, 1.0),
            Vec2::new(5.0, 1.0),
            Vec2::new(1.0, 4.0),
        ])
        .unwrap();
        assert_eq!(
            b,
            PixelBounds {
                min_x: 1,
                max_x: 4,
                min_y: 1,
                max_y: 3
            }
        );
        assert_eq!(b.pixel_count(), 4 * 3);
    }

    #[test]
    fn admite_coordenadas_negativas() {
        let b = pixel_bounds(&[
            Vec2::new(-3.5, -2.0),
            Vec2::new(0.0, -2.0),
            Vec2::new(-3.5, 1.0),
        ])
        .unwrap();
        assert_eq!(b.min_x, -4);
        assert_eq!(b.min_y, -2);
    }

    #[test]
    fn un_triangulo_mas_chico_que_un_pixel_no_tiene_centros() {
        let b = pixel_bounds(&[
            Vec2::new(2.6, 2.6),
            Vec2::new(2.9, 2.6),
            Vec2::new(2.6, 2.9),
        ]);
        assert!(b.is_none());
    }
}
