//! Los dos algoritmos de relleno tienen que producir la misma figura.

use filling_triangles::color::Color;
use filling_triangles::compare::compare_fills;
use filling_triangles::framebuffer::Framebuffer;
use filling_triangles::render::{process_fragments, rasterize, Triangle};
use filling_triangles::scene::{test_cases, HEIGHT, WIDTH};
use filling_triangles::triangle::TriangleMode;
use filling_triangles::vertex::{Vec2, Vertex};

/// Generador simple y determinista: las pruebas tienen que fallar siempre con
/// los mismos triangulos.
struct Lcg(u64);

impl Lcg {
    fn next_f32(&mut self, min: f32, max: f32) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let unidad = ((self.0 >> 33) as f32) / ((1u64 << 31) as f32);
        min + unidad * (max - min)
    }
}

fn triangulo(a: Vec2, b: Vec2, c: Vec2) -> Triangle {
    [
        Vertex::new(a, Color::WHITE),
        Vertex::new(b, Color::WHITE),
        Vertex::new(c, Color::WHITE),
    ]
}

fn pintar(triangulo: &Triangle, mode: TriangleMode) -> Vec<u32> {
    let mut fb = Framebuffer::new(WIDTH, HEIGHT);
    fb.clear();
    let fragmentos = rasterize(std::slice::from_ref(triangulo), mode);
    process_fragments(&mut fb, &fragmentos);
    fb.buffer().to_vec()
}

#[test]
fn los_casos_del_laboratorio_dan_la_misma_figura() {
    for caso in test_cases() {
        let resultado = compare_fills(&caso.triangle);
        assert!(
            resultado.matches(),
            "{}: {} posiciones solo en baricentrico, {} solo en scanline",
            caso.name,
            resultado.only_barycentric.len(),
            resultado.only_scanline.len()
        );
        assert_eq!(
            resultado.barycentric_count, resultado.scanline_count,
            "{}: distinta cantidad de fragmentos",
            caso.name
        );
    }
}

#[test]
fn los_casos_del_laboratorio_pintan_la_misma_imagen() {
    for caso in test_cases() {
        assert_eq!(
            pintar(&caso.triangle, TriangleMode::FilledBarycentric),
            pintar(&caso.triangle, TriangleMode::FilledScanline),
            "{}: las imagenes difieren",
            caso.name
        );
    }
}

#[test]
fn coinciden_en_triangulos_al_azar() {
    let mut rng = Lcg(0x5EED);
    for _ in 0..80 {
        let a = Vec2::new(rng.next_f32(-50.0, 850.0), rng.next_f32(-50.0, 650.0));
        let b = Vec2::new(rng.next_f32(-50.0, 850.0), rng.next_f32(-50.0, 650.0));
        let c = Vec2::new(rng.next_f32(-50.0, 850.0), rng.next_f32(-50.0, 650.0));
        let t = triangulo(a, b, c);
        let resultado = compare_fills(&t);
        assert!(
            resultado.matches(),
            "difieren en {a:?} {b:?} {c:?}: solo bar {:?}, solo scan {:?}",
            resultado.only_barycentric.len(),
            resultado.only_scanline.len()
        );
    }
}

#[test]
fn coinciden_en_triangulos_diminutos() {
    let mut rng = Lcg(0xC0FFEE);
    for _ in 0..300 {
        let base = Vec2::new(rng.next_f32(10.0, 50.0), rng.next_f32(10.0, 50.0));
        let a = base;
        let b = Vec2::new(base.x + rng.next_f32(0.0, 3.0), base.y + rng.next_f32(0.0, 3.0));
        let c = Vec2::new(base.x + rng.next_f32(0.0, 3.0), base.y + rng.next_f32(0.0, 3.0));
        let resultado = compare_fills(&triangulo(a, b, c));
        assert!(resultado.matches(), "difieren en {a:?} {b:?} {c:?}");
    }
}

#[test]
fn ningun_relleno_sale_del_contorno() {
    // El contorno y el relleno son el mismo triangulo: todo fragmento del
    // relleno tiene que caer dentro del rectangulo que ocupa el contorno.
    for caso in test_cases() {
        let contorno = rasterize(std::slice::from_ref(&caso.triangle), TriangleMode::Wireframe);
        let relleno = rasterize(
            std::slice::from_ref(&caso.triangle),
            TriangleMode::FilledBarycentric,
        );
        if relleno.is_empty() {
            continue;
        }
        let min_x = contorno.iter().map(|f| f.x).min().unwrap();
        let max_x = contorno.iter().map(|f| f.x).max().unwrap();
        let min_y = contorno.iter().map(|f| f.y).min().unwrap();
        let max_y = contorno.iter().map(|f| f.y).max().unwrap();
        for f in &relleno {
            assert!(
                f.x >= min_x && f.x <= max_x && f.y >= min_y && f.y <= max_y,
                "{}: el fragmento {:?} se sale del contorno",
                caso.name,
                f.position()
            );
        }
    }
}
