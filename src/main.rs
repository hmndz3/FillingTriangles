use filling_triangles::cli::{self, Command, HELP};
use filling_triangles::compare::compare_fills;
use filling_triangles::scene::{render_scene, test_cases};
use filling_triangles::triangle::TriangleMode;
use std::process::ExitCode;

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
        Command::Window | Command::BenchmarkTriangles => {
            println!("modo todavia no implementado: {command:?}");
        }
    }
    ExitCode::SUCCESS
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
