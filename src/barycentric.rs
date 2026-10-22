//! Coordenadas baricentricas de un punto respecto a un triangulo.

use crate::vertex::{cross2, Vec2};

/// Pesos de `p` respecto a los vertices a, b y c. Suman 1 y son todos
/// positivos o cero exactamente cuando `p` esta dentro del triangulo.
///
/// Devuelve `None` cuando el area del triangulo es cero: en ese caso no hay
/// nada entre que interpolar y dividir no tendria sentido.
pub fn barycentric(a: Vec2, b: Vec2, c: Vec2, p: Vec2) -> Option<[f64; 3]> {
    let area2 = cross2(a, b, c);
    if area2 == 0.0 {
        return None;
    }
    // Cada peso es el area del subtriangulo opuesto al vertice, con signo.
    // Al dividir entre el area total, el signo del orden de los vertices se
    // cancela: invertir a, b y c da los mismos pesos.
    Some([
        cross2(b, c, p) / area2,
        cross2(c, a, p) / area2,
        cross2(a, b, p) / area2,
    ])
}

/// Un punto esta dentro si sus tres coordenadas son positivas o cero. Los
/// puntos justo sobre una arista cuentan como interiores.
pub fn inside(weights: [f64; 3]) -> bool {
    weights[0] >= 0.0 && weights[1] >= 0.0 && weights[2] >= 0.0
}

#[cfg(test)]
mod tests {
    use super::{barycentric, inside};
    use crate::vertex::Vec2;

    const A: Vec2 = Vec2::new(0.0, 0.0);
    const B: Vec2 = Vec2::new(6.0, 0.0);
    const C: Vec2 = Vec2::new(0.0, 6.0);

    #[test]
    fn en_un_vertice_el_peso_es_uno() {
        let w = barycentric(A, B, C, A).unwrap();
        assert_eq!(w, [1.0, 0.0, 0.0]);
        assert!(inside(w));
    }

    #[test]
    fn en_el_centroide_los_pesos_son_iguales() {
        let w = barycentric(A, B, C, Vec2::new(2.0, 2.0)).unwrap();
        for peso in w {
            assert!((peso - 1.0 / 3.0).abs() < 1e-9);
        }
        assert!(inside(w));
    }

    #[test]
    fn fuera_del_triangulo_hay_un_peso_negativo() {
        let w = barycentric(A, B, C, Vec2::new(5.0, 5.0)).unwrap();
        assert!(!inside(w));
    }

    #[test]
    fn sobre_una_arista_cuenta_como_dentro() {
        let w = barycentric(A, B, C, Vec2::new(3.0, 0.0)).unwrap();
        assert_eq!(w[2], 0.0);
        assert!(inside(w));
    }

    #[test]
    fn invertir_el_orden_no_cambia_los_pesos() {
        let p = Vec2::new(1.0, 2.0);
        let directo = barycentric(A, B, C, p).unwrap();
        let invertido = barycentric(C, B, A, p).unwrap();
        assert_eq!(directo[0], invertido[2]);
        assert_eq!(directo[1], invertido[1]);
        assert_eq!(directo[2], invertido[0]);
        assert!(inside(directo) && inside(invertido));
    }

    #[test]
    fn el_triangulo_degenerado_no_tiene_coordenadas() {
        let colineales = barycentric(
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(1.0, 1.0),
        );
        assert!(colineales.is_none());
    }
}
