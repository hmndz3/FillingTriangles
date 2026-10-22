//! Rasterizacion de triangulos.
//!
//! Un triangulo son siempre los mismos tres vertices; lo que cambia entre
//! modos es como se generan los fragmentos.

use crate::barycentric::{barycentric, inside};
use crate::bounds::pixel_bounds;
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

/// Relleno con coordenadas baricentricas: se recorre el rectangulo contenedor
/// y se prueba el centro de cada pixel.
///
/// Un triangulo de area cero no genera fragmentos: `barycentric` devuelve
/// `None` y se trata como caso degenerado en lugar de dividir entre cero.
pub fn fill_barycentric(a: Vec2, b: Vec2, c: Vec2, color: Color) -> Vec<Fragment> {
    let Some(bounds) = pixel_bounds(&[a, b, c]) else {
        return Vec::new();
    };

    // Un triangulo ocupa aproximadamente la mitad de su rectangulo contenedor.
    let mut fragments = Vec::with_capacity(bounds.pixel_count() / 2 + 1);
    for y in bounds.rows() {
        for x in bounds.columns() {
            let centro = Vec2::pixel_center(x, y);
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

#[cfg(test)]
mod tests {
    use super::{fill_barycentric, wireframe, TriangleMode};
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

    #[test]
    fn cada_modo_tiene_nombre() {
        assert_eq!(TriangleMode::ALL.len(), 3);
        assert_eq!(TriangleMode::Wireframe.name(), "wireframe");
    }
}
