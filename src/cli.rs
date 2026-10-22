//! Argumentos de linea de comandos.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Abre la ventana y dibuja la escena.
    Window,
    /// Compara las posiciones que genera cada relleno, caso por caso.
    CompareFills,
    /// Rasteriza muchas veces y reporta tiempos, sin abrir la ventana.
    BenchmarkTriangles,
    /// Guarda una imagen por cada modo de dibujo.
    DumpImages,
    Help,
}

pub const HELP: &str = "\
filling_triangles: rasterizador por software

Uso: filling_triangles [opcion]

  (sin opcion)            abre la ventana con la escena de triangulos
  --compare-fills         compara las posiciones de los dos rellenos
  --benchmark-triangles   mide la generacion de fragmentos sin abrir la ventana
  --dump-images           guarda out/<modo>.ppm para cada modo de dibujo
  -h, --help              muestra esta ayuda
";

pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Command, String> {
    let mut command = Command::Window;
    for arg in args {
        command = match arg.as_str() {
            "--compare-fills" => Command::CompareFills,
            "--benchmark-triangles" => Command::BenchmarkTriangles,
            "--dump-images" => Command::DumpImages,
            "-h" | "--help" => Command::Help,
            otro => return Err(format!("argumento desconocido: {otro}")),
        };
    }
    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::{parse, Command};

    fn args(valores: &[&str]) -> Vec<String> {
        valores.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn sin_argumentos_abre_la_ventana() {
        assert_eq!(parse(args(&[])), Ok(Command::Window));
    }

    #[test]
    fn reconoce_el_modo_de_medicion() {
        assert_eq!(
            parse(args(&["--benchmark-triangles"])),
            Ok(Command::BenchmarkTriangles)
        );
    }

    #[test]
    fn rechaza_lo_que_no_conoce() {
        assert!(parse(args(&["--otra-cosa"])).is_err());
    }
}
