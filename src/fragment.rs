//! Fragmento: una posicion entera de la imagen con el color que le corresponde.

use crate::color::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fragment {
    pub x: i32,
    pub y: i32,
    pub color: Color,
}

impl Fragment {
    pub const fn new(x: i32, y: i32, color: Color) -> Self {
        Self { x, y, color }
    }

    /// Posicion sin color: sirve para comparar dos algoritmos de relleno
    /// mirando solo que pixeles genera cada uno.
    pub const fn position(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::Fragment;
    use crate::color::Color;

    #[test]
    fn la_posicion_ignora_el_color() {
        let a = Fragment::new(2, 5, Color::RED);
        let b = Fragment::new(2, 5, Color::BLUE);
        assert_eq!(a.position(), b.position());
        assert_ne!(a, b);
    }
}
