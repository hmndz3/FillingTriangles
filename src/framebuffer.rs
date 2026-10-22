//! Framebuffer en memoria: una rejilla de pixeles que luego se envia a la ventana.

use crate::color::Color;

pub struct Framebuffer {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
    background: Color,
    current: Color,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![Color::BLACK.to_u32(); width * height],
            background: Color::BLACK,
            current: Color::WHITE,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn buffer(&self) -> &[u32] {
        &self.buffer
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current = color;
    }

    pub fn clear(&mut self) {
        self.buffer.fill(self.background.to_u32());
    }

    /// Pinta un pixel con el color actual. Las posiciones fuera del framebuffer
    /// se descartan en silencio: el recorte ocurre aqui y no en la rasterizacion.
    pub fn point(&mut self, x: i32, y: i32) {
        self.point_with(x, y, self.current);
    }

    pub fn point_with(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        let index = y as usize * self.width + x as usize;
        self.buffer[index] = color.to_u32();
    }

    pub fn get(&self, x: i32, y: i32) -> Option<u32> {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return None;
        }
        Some(self.buffer[y as usize * self.width + x as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::Framebuffer;
    use crate::color::Color;

    #[test]
    fn pinta_dentro_del_area() {
        let mut fb = Framebuffer::new(4, 3);
        fb.set_current_color(Color::RED);
        fb.point(2, 1);
        assert_eq!(fb.get(2, 1), Some(Color::RED.to_u32()));
        assert_eq!(fb.get(0, 0), Some(Color::BLACK.to_u32()));
    }

    #[test]
    fn descarta_fuera_del_area() {
        let mut fb = Framebuffer::new(4, 3);
        fb.point(-1, 0);
        fb.point(0, -1);
        fb.point(4, 0);
        fb.point(0, 3);
        assert!(fb.buffer().iter().all(|&p| p == Color::BLACK.to_u32()));
        assert_eq!(fb.get(4, 0), None);
    }

    #[test]
    fn limpia_con_el_color_de_fondo() {
        let mut fb = Framebuffer::new(2, 2);
        fb.set_background_color(Color::BLUE);
        fb.clear();
        assert!(fb.buffer().iter().all(|&p| p == Color::BLUE.to_u32()));
    }
}
