mod barycentric;
mod color;
mod fragment;
mod framebuffer;
mod line;
mod triangle;
mod vertex;

use framebuffer::Framebuffer;

fn main() {
    let fb = Framebuffer::new(800, 600);
    println!(
        "filling_triangles: framebuffer de {}x{}",
        fb.width(),
        fb.height()
    );
}
