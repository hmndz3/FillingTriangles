mod color;

use color::Color;

fn main() {
    println!("filling_triangles: rasterizador por software");
    println!("color de fondo: {:#08X}", Color::BLACK.to_u32());
}
