//! Medicion de la generacion de fragmentos.
//!
//! Se mide solo `rasterize`: no entran ni la actualizacion de la textura, ni el
//! dibujo de la ventana, ni la espera entre cuadros.

use crate::render::{rasterize, Triangle};
use crate::triangle::TriangleMode;
use std::hint::black_box;
use std::time::{Duration, Instant};

pub struct BenchResult {
    pub mode: TriangleMode,
    /// Rasterizaciones por tanda.
    pub iterations: usize,
    /// Cantidad de tandas medidas.
    pub repeats: usize,
    pub median: Duration,
    pub best: Duration,
    pub worst: Duration,
    /// Fragmentos que produce una rasterizacion de la escena.
    pub fragments: usize,
}

impl BenchResult {
    /// Tiempo de una sola rasterizacion de la escena.
    pub fn per_run(&self) -> Duration {
        self.median / self.iterations as u32
    }

    /// Nanosegundos por fragmento generado.
    pub fn nanos_per_fragment(&self) -> f64 {
        if self.fragments == 0 {
            return 0.0;
        }
        self.per_run().as_nanos() as f64 / self.fragments as f64
    }
}

/// Corre `repeats` tandas de `iterations` rasterizaciones y se queda con la
/// mediana: una sola medicion se ve afectada por cualquier cosa que este
/// haciendo la maquina.
///
/// `black_box` evita que el optimizador borre el trabajo que se quiere medir.
pub fn benchmark(
    triangles: &[Triangle],
    mode: TriangleMode,
    iterations: usize,
    repeats: usize,
) -> BenchResult {
    // Una corrida de calentamiento: la primera siempre paga el costo de llenar
    // las caches.
    let fragments = rasterize(triangles, mode).len();

    let mut tandas = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let inicio = Instant::now();
        for _ in 0..iterations {
            let fragments = rasterize(black_box(triangles), black_box(mode));
            black_box(fragments);
        }
        tandas.push(inicio.elapsed());
    }
    tandas.sort_unstable();

    BenchResult {
        mode,
        iterations,
        repeats,
        median: tandas[tandas.len() / 2],
        best: tandas[0],
        worst: tandas[tandas.len() - 1],
        fragments,
    }
}

#[cfg(test)]
mod tests {
    use super::benchmark;
    use crate::scene::demo_triangles;
    use crate::triangle::TriangleMode;

    #[test]
    fn la_medicion_reporta_tiempos_y_fragmentos() {
        let triangulos = demo_triangles();
        let r = benchmark(&triangulos, TriangleMode::FilledScanline, 2, 3);
        assert_eq!(r.repeats, 3);
        assert!(r.fragments > 0);
        assert!(r.median >= r.best);
        assert!(r.worst >= r.median);
        assert!(r.nanos_per_fragment() > 0.0);
    }
}
