use filling_triangles::cli::{self, Command, HELP};
use filling_triangles::compare::compare_fills;
use filling_triangles::render::{process_fragments, rasterize};
use filling_triangles::scene::{demo_triangles, new_framebuffer, render_scene, test_cases, HEIGHT, WIDTH};
use filling_triangles::triangle::TriangleMode;
use minifb::{Key, Window, WindowOptions};
use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    let command = match cli::parse(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("{error}\n\n{HELP}");
            return ExitCode::FAILURE;
        }
    };

    match command {
        Command::Help => print!("{HELP}"),
        Command::CompareFills => compare_fills_command(),
        Command::DumpImages => {
            if let Err(error) = dump_images_command() {
                eprintln!("no se pudo guardar la imagen: {error}");
                return ExitCode::FAILURE;
            }
        }
        Command::Window => window_command(),
        Command::BenchmarkTriangles => {
            println!("modo todavia no implementado: {command:?}");
        }
    }
    ExitCode::SUCCESS
}

/// Lee el modo que pide el teclado: 1 contorno, 2 baricentrico, 3 scanline.
fn mode_from_keys(window: &Window) -> Option<TriangleMode> {
    if window.is_key_down(Key::Key1) {
        Some(TriangleMode::Wireframe)
    } else if window.is_key_down(Key::Key2) {
        Some(TriangleMode::FilledBarycentric)
    } else if window.is_key_down(Key::Key3) {
        Some(TriangleMode::FilledScanline)
    } else {
        None
    }
}

/// Abre la ventana y dibuja la escena. La escena se vuelve a rasterizar solo
/// cuando cambia el modo; el tiempo que aparece en el titulo es unicamente el
/// de generar los fragmentos.
fn window_command() {
    let mut window = Window::new(
        "Filling triangles",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("no se pudo abrir la ventana");
    window.set_target_fps(60);

    let triangles = demo_triangles();
    let mut framebuffer = new_framebuffer();
    let mut mode = TriangleMode::Wireframe;
    let mut redibujar = true;

    println!("teclas: 1 contorno, 2 baricentrico, 3 scanline, Esc para salir");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Some(pedido) = mode_from_keys(&window) {
            if pedido != mode {
                mode = pedido;
                redibujar = true;
            }
        }

        if redibujar {
            let inicio = Instant::now();
            let fragments = rasterize(&triangles, mode);
            let transcurrido = inicio.elapsed();

            framebuffer.clear();
            process_fragments(&mut framebuffer, &fragments);
            window.set_title(&format!(
                "Filling triangles - {} - {} fragmentos - {:?}",
                mode.name(),
                fragments.len(),
                transcurrido
            ));
            redibujar = false;
        }

        window
            .update_with_buffer(framebuffer.buffer(), WIDTH, HEIGHT)
            .expect("no se pudo actualizar la ventana");
    }
}

/// Guarda una imagen por modo para poder compararlas fuera de la ventana.
fn dump_images_command() -> std::io::Result<()> {
    for mode in TriangleMode::ALL {
        let framebuffer = render_scene(mode);
        let ruta = format!("out/{}.ppm", mode.name());
        framebuffer.save_ppm(&ruta)?;
        println!("{ruta}");
    }
    Ok(())
}

/// Compara los dos rellenos caso por caso y reporta las diferencias.
fn compare_fills_command() {
    println!(
        "{:<22} {:>10} {:>10} {:>12} {:>12}",
        "caso", "baricentr", "scanline", "solo bar", "solo scan"
    );
    let mut iguales = true;
    for caso in test_cases() {
        let r = compare_fills(&caso.triangle);
        iguales &= r.matches();
        println!(
            "{:<22} {:>10} {:>10} {:>12} {:>12}",
            caso.name,
            r.barycentric_count,
            r.scanline_count,
            r.only_barycentric.len(),
            r.only_scanline.len()
        );
        for p in r.only_barycentric.iter().take(5) {
            println!("    solo baricentrico: {p:?}");
        }
        for p in r.only_scanline.iter().take(5) {
            println!("    solo scanline:     {p:?}");
        }
    }
    if iguales {
        println!("\nLos dos algoritmos generan exactamente las mismas posiciones.");
    } else {
        println!("\nHay posiciones distintas: revisar la regla de borde de cada relleno.");
    }
}
