//! Posiciones en el plano de la pantalla y vertices con atributos.

use crate::color::Color;

/// Posicion en coordenadas de pantalla. El pixel entero (x, y) tiene su centro
/// en (x + 0.5, y + 0.5); toda la rasterizacion muestrea esos centros.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Centro del pixel entero (x, y).
    pub fn pixel_center(x: i32, y: i32) -> Self {
        Self::new(x as f32 + 0.5, y as f32 + 0.5)
    }
}

/// Producto cruz en 2D de (b - a) x (c - a). Su signo indica el orden de los
/// vertices y su magnitud es el doble del area del triangulo a-b-c.
pub fn cross2(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub position: Vec2,
    pub color: Color,
}

impl Vertex {
    pub const fn new(position: Vec2, color: Color) -> Self {
        Self { position, color }
    }
}

#[cfg(test)]
mod tests {
    use super::{cross2, Vec2};

    #[test]
    fn centro_del_pixel() {
        assert_eq!(Vec2::pixel_center(3, 7), Vec2::new(3.5, 7.5));
    }

    #[test]
    fn el_signo_del_cruz_depende_del_orden() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(4.0, 0.0);
        let c = Vec2::new(0.0, 3.0);
        assert_eq!(cross2(a, b, c), 12.0);
        assert_eq!(cross2(a, c, b), -12.0);
    }

    #[test]
    fn colineales_dan_cero() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(2.0, 2.0);
        let c = Vec2::new(5.0, 5.0);
        assert_eq!(cross2(a, b, c), 0.0);
    }
}
