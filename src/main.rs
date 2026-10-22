use filling_triangles::bench::benchmark;
use filling_triangles::cli::{self, Command, HELP};
use filling_triangles::compare::compare_fills;
use filling_triangles::profile::{count_barycentric, count_scanline};
use filling_triangles::render::{process_fragments, rasterize};
use filling_triangles::scene::{
    demo_triangles, new_framebuffer, render_scene, test_cases, HEIGHT, WIDTH,
};
use filling_triangles::triangle::TriangleMode;
use minifb::{Key, Window, WindowOptions};
use std::hint::black_box;
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
        Command::BenchmarkTriangles => benchmark_command(),
        Command::ProfilePhases => profile_phases_command(),
    }
    ExitCode::SUCCESS
}

/// Rasteriza la escena muchas veces con cada modo y reporta los tiempos.
/// Termina sin abrir la ventana, que es lo que necesita el perfilador.
fn benchmark_command() {
    const ITERATIONS: usize = 100;
    const REPEATS: usize = 11;

    let triangles = demo_triangles();
    println!(
        "escena: {} triangulos, {} rasterizaciones por tanda, {} tandas
",
        triangles.len(),
        ITERATIONS,
        REPEATS
    );
    println!(
        "{:<14} {:>12} {:>14} {:>12} {:>12} {:>10}",
        "modo", "fragmentos", "mediana/tanda", "por corrida", "ns/fragmento", "mejor"
    );

    for mode in TriangleMode::ALL {
        let r = benchmark(&triangles, mode, ITERATIONS, REPEATS);
        println!(
            "{:<14} {:>12} {:>14?} {:>12?} {:>12.2} {:>10?}",
            r.mode.name(),
            r.fragments,
            r.median,
            r.per_run(),
            r.nanos_per_fragment(),
            r.best
        );
    }

    benchmark_per_case();
}

/// Mide cada caso por separado: un triangulo chico y uno grande no se
/// comportan igual aunque el algoritmo sea el mismo.
fn benchmark_per_case() {
    const ITERATIONS: usize = 20;
    const REPEATS: usize = 7;

    println!(
        "
por caso ({ITERATIONS} rasterizaciones por tanda, {REPEATS} tandas)
"
    );
    println!(
        "{:<22} {:>12} {:>14} {:>14} {:>10}",
        "caso", "fragmentos", "baricentrico", "scanline", "relacion"
    );

    for caso in test_cases() {
        let triangulo = [caso.triangle];
        let bar = benchmark(
            &triangulo,
            TriangleMode::FilledBarycentric,
            ITERATIONS,
            REPEATS,
        );
        let scan = benchmark(
            &triangulo,
            TriangleMode::FilledScanline,
            ITERATIONS,
            REPEATS,
        );
        let relacion =
            bar.per_run().as_secs_f64() / scan.per_run().as_secs_f64().max(f64::MIN_POSITIVE);
        println!(
            "{:<22} {:>12} {:>14?} {:>14?} {:>9.2}x",
            caso.name,
            bar.fragments,
            bar.per_run(),
            scan.per_run(),
            relacion
        );
    }
}

/// Mide la geometria sola (contando fragmentos sin crearlos) contra la
/// rasterizacion completa. La diferencia es lo que cuesta armar el `Vec` de
/// fragmentos.
fn profile_phases_command() {
    const ITERATIONS: usize = 100;
    const REPEATS: usize = 11;

    let triangles = demo_triangles();
    let geometria = |contar: fn(_, _, _) -> usize| {
        let mut tandas = Vec::with_capacity(REPEATS);
        for _ in 0..REPEATS {
            let inicio = Instant::now();
            for _ in 0..ITERATIONS {
                let mut total = 0;
                for [a, b, c] in black_box(&triangles) {
                    total += contar(a.position, b.position, c.position);
                }
                black_box(total);
            }
            tandas.push(inicio.elapsed() / ITERATIONS as u32);
        }
        tandas.sort_unstable();
        tandas[tandas.len() / 2]
    };

    println!(
        "{:<14} {:>14} {:>14} {:>14} {:>12}",
        "modo", "solo geometria", "completo", "fragmentos", "geometria %"
    );

    for (mode, contar) in [
        (
            TriangleMode::FilledBarycentric,
            count_barycentric as fn(_, _, _) -> usize,
        ),
        (
            TriangleMode::FilledScanline,
            count_scanline as fn(_, _, _) -> usize,
        ),
    ] {
        let solo_geometria = geometria(contar);
        let completo = benchmark(&triangles, mode, ITERATIONS, REPEATS);
        let porcentaje = 100.0 * solo_geometria.as_secs_f64() / completo.per_run().as_secs_f64();
        println!(
            "{:<14} {:>14?} {:>14?} {:>14} {:>11.1}%",
            mode.name(),
            solo_geometria,
            completo.per_run(),
            completo.fragments,
            porcentaje
        );
    }
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
    let mut window = Window::new("Filling triangles", WIDTH, HEIGHT, WindowOptions::default())
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
