//! Color RGB de 8 bits por canal.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Empaqueta el color en 0x00RRGGBB, que es el formato que espera la ventana.
    pub const fn to_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const WHITE: Color = Color::new(255, 255, 255);
    pub const RED: Color = Color::new(220, 60, 60);
    pub const GREEN: Color = Color::new(80, 200, 120);
    pub const BLUE: Color = Color::new(70, 130, 220);
    pub const YELLOW: Color = Color::new(230, 200, 70);
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn empaqueta_en_rgb() {
        assert_eq!(Color::new(0x12, 0x34, 0x56).to_u32(), 0x0012_3456);
        assert_eq!(Color::WHITE.to_u32(), 0x00FF_FFFF);
    }
}
