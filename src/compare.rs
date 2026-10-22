//! Comparacion de los dos rellenos mirando las posiciones que genera cada uno.
//!
//! Comparar las ventanas a ojo no sirve para detectar una fila de diferencia en
//! un borde, asi que se comparan los conjuntos de fragmentos.

use crate::render::Triangle;
use crate::triangle::{fill_barycentric, fill_scanline};
use std::collections::BTreeSet;

pub struct Comparison {
    pub barycentric_count: usize,
    pub scanline_count: usize,
    /// Posiciones que solo genera el relleno baricentrico.
    pub only_barycentric: Vec<(i32, i32)>,
    /// Posiciones que solo genera el relleno por lineas horizontales.
    pub only_scanline: Vec<(i32, i32)>,
    /// Fragmentos repetidos dentro de un mismo algoritmo.
    pub barycentric_duplicates: usize,
    pub scanline_duplicates: usize,
}

impl Comparison {
    /// Las dos implementaciones producen la misma figura.
    pub fn matches(&self) -> bool {
        self.only_barycentric.is_empty() && self.only_scanline.is_empty()
    }
}

pub fn compare_fills(triangle: &Triangle) -> Comparison {
    let [a, b, c] = triangle;
    let color = a.color;
    let baricentrico = fill_barycentric(a.position, b.position, c.position, color);
    let por_filas = fill_scanline(a.position, b.position, c.position, color);

    let set_bar: BTreeSet<(i32, i32)> = baricentrico.iter().map(|f| f.position()).collect();
    let set_fil: BTreeSet<(i32, i32)> = por_filas.iter().map(|f| f.position()).collect();

    Comparison {
        barycentric_count: baricentrico.len(),
        scanline_count: por_filas.len(),
        only_barycentric: set_bar.difference(&set_fil).copied().collect(),
        only_scanline: set_fil.difference(&set_bar).copied().collect(),
        barycentric_duplicates: baricentrico.len() - set_bar.len(),
        scanline_duplicates: por_filas.len() - set_fil.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::compare_fills;
    use crate::scene::test_cases;

    #[test]
    fn los_dos_rellenos_coinciden_en_todos_los_casos() {
        for caso in test_cases() {
            let r = compare_fills(&caso.triangle);
            assert!(
                r.matches(),
                "{}: {} solo baricentrico, {} solo scanline",
                caso.name,
                r.only_barycentric.len(),
                r.only_scanline.len()
            );
            assert_eq!(r.barycentric_duplicates, 0, "{}", caso.name);
            assert_eq!(r.scanline_duplicates, 0, "{}", caso.name);
        }
    }
}
