use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use simplelog::{ConfigBuilder, WriteLogger};

use thigpen::Lib;

fn get_default_cwd() -> OsString {
    std::env::current_dir().unwrap().into_os_string()
}

#[derive(Clone, Debug, Default, ValueEnum)]
enum OutputType {
    #[default]
    Mermaid,
}

impl std::fmt::Display for OutputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mermaid => write!(f, "mermaid"),
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = false)]
    debug: bool,
    #[arg(default_value=get_default_cwd())]
    path: PathBuf,
    #[arg(long, default_value_t = OutputType::Mermaid)]
    output_type: OutputType,
    #[arg(short)]
    output: Option<String>,
}

fn find_cargo_toml(path: &Path) -> Option<PathBuf> {
    assert!(path.is_dir());

    let mut path = path.to_path_buf();
    loop {
        path.push("Cargo.toml");
        if path.is_file() {
            return Some(path);
        }
        if !(path.pop() && path.pop()) {
            return None;
        }
    }
}

fn main() {
    let args = Args::parse();

    if args.debug {
        let config = ConfigBuilder::new()
            .set_max_level(log::LevelFilter::Debug)
            .build();
        WriteLogger::init(
            log::LevelFilter::Debug,
            config,
            std::fs::File::create("thigpen-debug.log").expect("could not create debug log"),
        )
        .expect("Could not initialize logging");
    }
    log::debug!("Using path: {:?}", args.path);

    let cargo_toml_path = match find_cargo_toml(&args.path) {
        Some(path) => path,
        None => {
            eprintln!(
                "Could not find Cargo.toml from any parent of {}",
                args.path.to_str().unwrap()
            );
            std::process::exit(1);
        }
    };
    log::debug!("Found manifest at {:?}", cargo_toml_path);

    let manifest = match cargo_toml::Manifest::from_path(cargo_toml_path) {
        Ok(manifest) => manifest,
        Err(err) => {
            eprintln!("Could not parse Cargo.toml - {err}");
            std::process::exit(1);
        }
    };
    log::debug!("Manifest: {:?}", manifest);

    if let Some(product) = manifest.lib {
        log::debug!("Analyzing library as {}", product.name.as_ref().unwrap());
        let mut path = args.path.clone();
        path.push(product.clone().path.unwrap());
        let crate_ = Lib::from_path(product.name.as_ref().unwrap(), path.as_path());

        let contents = crate_.create_mermaid();
        if let Some(filename) = args.output {
            let mut outputfile = std::fs::OpenOptions::new()
                .truncate(true)
                .create(true)
                .write(true)
                .read(false)
                .open(filename)
                .unwrap();
            write!(&mut outputfile, "{contents}").unwrap();
        } else {
            let mut handle = std::io::stdout().lock();
            write!(&mut handle, "{contents}").unwrap();
        };
    } else {
        eprintln!("thigpen does not (yet) support non-lib crates");
        std::process::exit(1);
    }
}
