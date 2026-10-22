//! Rasterizacion de triangulos.
//!
//! Un triangulo son siempre los mismos tres vertices; lo que cambia entre
//! modos es como se generan los fragmentos.

use crate::color::Color;
use crate::fragment::Fragment;
use crate::line::line;
use crate::vertex::Vec2;

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

/// Contorno del triangulo: las tres aristas como lineas.
pub fn wireframe(a: Vec2, b: Vec2, c: Vec2, color: Color) -> Vec<Fragment> {
    let mut fragments = line(a, b, color);
    fragments.extend(line(b, c, color));
    fragments.extend(line(c, a, color));
    fragments
}

#[cfg(test)]
mod tests {
    use super::{wireframe, TriangleMode};
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

    #[test]
    fn cada_modo_tiene_nombre() {
        assert_eq!(TriangleMode::ALL.len(), 3);
        assert_eq!(TriangleMode::Wireframe.name(), "wireframe");
    }
}
