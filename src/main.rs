use filling_triangles::cli::{self, Command, HELP};
use filling_triangles::compare::compare_fills;
use filling_triangles::scene::test_cases;
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
        Command::Window | Command::BenchmarkTriangles | Command::DumpImages => {
            println!("modo todavia no implementado: {command:?}");
        }
    }
    ExitCode::SUCCESS
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
