//! Etapas de la tuberia: ensamblaje de primitivas y rasterizacion.

use crate::fragment::Fragment;
use crate::triangle::{triangle, TriangleMode};
use crate::vertex::Vertex;

/// Una primitiva ya ensamblada.
pub type Triangle = [Vertex; 3];

/// Ensamblaje de primitivas: agrupa el flujo de vertices de tres en tres.
/// Los vertices sobrantes se descartan.
pub fn assemble_triangles(vertices: &[Vertex]) -> Vec<Triangle> {
    vertices
        .chunks_exact(3)
        .map(|v| [v[0], v[1], v[2]])
        .collect()
}

/// Rasteriza las primitivas con el modo indicado.
///
/// El color es por ahora uniforme y se toma del primer vertice; las
/// coordenadas baricentricas quedan disponibles para interpolar atributos mas
/// adelante.
pub fn rasterize(triangles: &[Triangle], mode: TriangleMode) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    for [a, b, c] in triangles {
        fragments.extend(triangle(
            a.position,
            b.position,
            c.position,
            mode,
            a.color,
        ));
    }
    fragments
}

#[cfg(test)]
mod tests {
    use super::{assemble_triangles, rasterize};
    use crate::color::Color;
    use crate::triangle::TriangleMode;
    use crate::vertex::{Vec2, Vertex};

    fn vertice(x: f32, y: f32, color: Color) -> Vertex {
        Vertex::new(Vec2::new(x, y), color)
    }

    #[test]
    fn agrupa_los_vertices_de_tres_en_tres() {
        let vertices: Vec<Vertex> = (0..7)
            .map(|i| vertice(i as f32, 0.0, Color::WHITE))
            .collect();
        let triangulos = assemble_triangles(&vertices);
        assert_eq!(triangulos.len(), 2);
        assert_eq!(triangulos[1][0].position.x, 3.0);
    }

    #[test]
    fn rasteriza_todas_las_primitivas() {
        let vertices = vec![
            vertice(0.0, 0.0, Color::RED),
            vertice(6.0, 0.0, Color::RED),
            vertice(0.0, 6.0, Color::RED),
            vertice(20.0, 20.0, Color::BLUE),
            vertice(26.0, 20.0, Color::BLUE),
            vertice(20.0, 26.0, Color::BLUE),
        ];
        let triangulos = assemble_triangles(&vertices);
        let fragmentos = rasterize(&triangulos, TriangleMode::FilledBarycentric);

        assert!(fragmentos.iter().any(|f| f.color == Color::RED));
        assert!(fragmentos.iter().any(|f| f.color == Color::BLUE));
    }
}
