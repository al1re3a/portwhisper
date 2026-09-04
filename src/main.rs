use std::{env, fs, process};

fn usage() {
    eprintln!("Usage: portwhisper [--format table|json|mermaid]\n       portwhisper --input FILE --source windows|linux|mac [--format ...]");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut format = "table".to_string();
    let mut input = None;
    let mut source = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                usage();
                return;
            }
            "--format" | "--input" | "--source" => {
                let option = args[i].as_str();
                let Some(value) = args.get(i + 1) else {
                    eprintln!("missing value for {option}");
                    process::exit(2)
                };
                match option {
                    "--format" => format = value.clone(),
                    "--input" => input = Some(value.clone()),
                    _ => source = Some(value.clone()),
                }
                i += 2;
            }
            value => {
                eprintln!("unknown option: {value}");
                usage();
                process::exit(2);
            }
        }
    }
    let listeners = if let Some(path) = input {
        let value = fs::read_to_string(&path).unwrap_or_else(|error| {
            eprintln!("portwhisper: {path}: {error}");
            process::exit(1)
        });
        match source.as_deref() {
            Some("windows") => portwhisper::parse_windows_netstat(&value),
            Some("linux") => portwhisper::parse_linux_ss(&value),
            Some("mac") | Some("macos") => portwhisper::parse_macos_lsof(&value),
            _ => {
                eprintln!("--source windows|linux|mac is required with --input");
                process::exit(2);
            }
        }
    } else {
        portwhisper::discover().unwrap_or_else(|error| {
            eprintln!("portwhisper: discovery failed: {error}");
            process::exit(1)
        })
    };
    match format.as_str() {
        "table" => print!("{}", portwhisper::render_table(&listeners)),
        "json" => println!("{}", portwhisper::render_json(&listeners)),
        "mermaid" => print!("{}", portwhisper::render_mermaid(&listeners)),
        _ => {
            eprintln!("unsupported format: {format}");
            process::exit(2);
        }
    }
}
