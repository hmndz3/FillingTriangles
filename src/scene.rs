//! Los triangulos con los que se prueban y se miden los dos algoritmos.
//!
//! Son los mismos para el dibujo, para las comparaciones y para la medicion:
//! asi ninguna comparacion termina midiendo figuras distintas.

use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::render::{assemble_triangles, process_fragments, rasterize, Triangle};
use crate::triangle::TriangleMode;
use crate::vertex::{Vec2, Vertex};

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 600;

/// Un caso de prueba con nombre, para poder decir cual fallo.
pub struct TestCase {
    pub name: &'static str,
    pub triangle: Triangle,
}

fn v(x: f32, y: f32, color: Color) -> Vertex {
    Vertex::new(Vec2::new(x, y), color)
}

/// Los casos que pide el laboratorio: tamanos distintos, aristas alineadas con
/// los ejes, el orden inverso, un triangulo que se sale de la imagen y tres
/// vertices colineales.
pub fn test_cases() -> Vec<TestCase> {
    let grande = [
        v(120.0, 40.0, Color::BLUE),
        v(700.0, 120.0, Color::BLUE),
        v(380.0, 420.0, Color::BLUE),
    ];

    vec![
        TestCase {
            name: "pequeno",
            triangle: [
                v(60.5, 60.5, Color::YELLOW),
                v(72.5, 62.0, Color::YELLOW),
                v(66.0, 73.5, Color::YELLOW),
            ],
        },
        TestCase {
            name: "grande",
            triangle: grande,
        },
        TestCase {
            name: "orden_inverso",
            triangle: [grande[2], grande[1], grande[0]],
        },
        TestCase {
            name: "arista_horizontal",
            triangle: [
                v(90.0, 500.0, Color::GREEN),
                v(290.0, 500.0, Color::GREEN),
                v(190.0, 380.0, Color::GREEN),
            ],
        },
        TestCase {
            name: "arista_vertical",
            triangle: [
                v(620.0, 300.0, Color::RED),
                v(620.0, 520.0, Color::RED),
                v(770.0, 410.0, Color::RED),
            ],
        },
        TestCase {
            name: "parcialmente_afuera",
            triangle: [
                v(-120.0, 150.0, Color::WHITE),
                v(180.0, 260.0, Color::WHITE),
                v(-40.0, 420.0, Color::WHITE),
            ],
        },
        TestCase {
            name: "colineales",
            triangle: [
                v(300.0, 560.0, Color::WHITE),
                v(400.0, 570.0, Color::WHITE),
                v(500.0, 580.0, Color::WHITE),
            ],
        },
    ]
}

/// Flujo de vertices de la escena que se dibuja en la ventana. Se omite el
/// orden inverso porque queda exactamente encima del triangulo grande.
pub fn demo_vertices() -> Vec<Vertex> {
    test_cases()
        .into_iter()
        .filter(|caso| caso.name != "orden_inverso")
        .flat_map(|caso| caso.triangle)
        .collect()
}

/// Las primitivas de la escena, ya ensambladas.
pub fn demo_triangles() -> Vec<Triangle> {
    assemble_triangles(&demo_vertices())
}

/// Dibuja la escena completa con un modo y devuelve el framebuffer.
pub fn render_scene(mode: TriangleMode) -> Framebuffer {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    framebuffer.set_background_color(Color::new(18, 18, 24));
    framebuffer.clear();
    let fragments = rasterize(&demo_triangles(), mode);
    process_fragments(&mut framebuffer, &fragments);
    framebuffer
}

#[cfg(test)]
mod tests {
    use super::{demo_triangles, render_scene, test_cases};
    use crate::triangle::TriangleMode;

    #[test]
    fn la_escena_tiene_los_casos_pedidos() {
        let nombres: Vec<&str> = test_cases().iter().map(|c| c.name).collect();
        assert!(nombres.contains(&"pequeno"));
        assert!(nombres.contains(&"grande"));
        assert!(nombres.contains(&"orden_inverso"));
        assert!(nombres.contains(&"parcialmente_afuera"));
        assert!(nombres.contains(&"colineales"));
        assert_eq!(demo_triangles().len(), nombres.len() - 1);
    }

    #[test]
    fn el_relleno_pinta_mas_pixeles_que_el_contorno() {
        let fondo = render_scene(TriangleMode::Wireframe);
        let contorno = fondo.buffer().iter().filter(|&&p| p != 0x0012_1218).count();
        let relleno = render_scene(TriangleMode::FilledBarycentric)
            .buffer()
            .iter()
            .filter(|&&p| p != 0x0012_1218)
            .count();
        assert!(contorno > 0);
        assert!(relleno > contorno * 10);
    }
}
