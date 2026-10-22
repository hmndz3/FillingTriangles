//! Rasterizador por software: framebuffer, lineas y triangulos.
//!
//! Los modulos estan expuestos como biblioteca para poder usarlos tanto desde
//! el binario como desde las pruebas de integracion en `tests/`.

pub mod barycentric;
pub mod bench;
pub mod bounds;
pub mod cli;
pub mod color;
pub mod compare;
pub mod fragment;
pub mod framebuffer;
pub mod line;
pub mod profile;
pub mod render;
pub mod scene;
pub mod triangle;
pub mod vertex;
