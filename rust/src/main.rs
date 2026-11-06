//! Run a Turing machine from machines/<name>.json on natural numbers, log M(x), and draw the
//! config.
//!
//!     cargo run --release -- reverse 6
//!
//! The numbers n_1, ..., n_k become the input x = encode(n_1, ..., n_k); a single number n
//! becomes x = NtS(n), its binary representation (most significant bit first).

mod window;

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use clap::builder::PossibleValuesParser;
use num_bigint::BigUint;

use simple_tm::tm::{Configuration, encode, load_machine};
use window::Settings;

const MACHINES_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../machines");

/// The names of the machines.
fn machine_names() -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(MACHINES_PATH)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .filter_map(|path| Some(path.file_stem()?.to_str()?.to_owned()))
        .collect();
    names.sort();
    names
}

/// Run a Turing machine from machines/<name>.json on natural numbers, log M(x), and draw the
/// config.
#[derive(Parser)]
#[command(allow_negative_numbers = true)]
struct Args {
    #[arg(value_parser = PossibleValuesParser::new(machine_names()))]
    machine: String,
    #[arg(required = true)]
    n: Vec<BigUint>,
    #[arg(long)]
    no_window: bool,
    #[arg(long, default_value_t = 1)]
    scale: usize,
    #[arg(long, default_value_t = 10.0)]
    speed: f64,
    #[arg(long, default_value_t = 10_000_000)]
    max_steps: u64,
    #[arg(long, default_value_t = 32)]
    max_cells: usize,
}

/// Check the arguments.
fn check_args(args: &Args) -> Result<(), String> {
    if args.scale < 1 {
        return Err(format!("Invalid args.scale={}", args.scale));
    }
    if args.speed.is_nan() || args.speed <= 0.0 {
        return Err(format!("Invalid args.speed={}", args.speed));
    }
    if args.max_steps < 1 {
        return Err(format!("Invalid args.max_steps={}", args.max_steps));
    }
    if args.max_cells < 1 {
        return Err(format!("Invalid args.max_cells={}", args.max_cells));
    }
    Ok(())
}

/// Log M(x) if the machine halted, or that it did not halt within `max_steps` steps.
fn log_result(config: &Configuration, max_steps: u64) {
    if config.halted {
        eprintln!("{}", config.output());
    } else {
        eprintln!("M did not halt within {} steps", window::thousands(max_steps));
    }
}

fn main() -> ExitCode {
    let args = Args::parse();
    if let Err(message) = check_args(&args) {
        eprintln!("{message}");
        return ExitCode::FAILURE;
    }

    // Load the machine.
    let path = Path::new(MACHINES_PATH).join(format!("{}.json", args.machine));
    let machine = match load_machine(&path) {
        Ok(machine) => machine,
        Err(e) => {
            eprintln!("{}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    if args.n.len() != machine.n_inputs {
        eprintln!("{} takes {} numbers, got {}", machine.name, machine.n_inputs, args.n.len());
        return ExitCode::FAILURE;
    }
    let x = encode(&args.n);

    // Run the machine.
    let max_steps = args.max_steps;
    if args.no_window {
        let mut config = Configuration::new(&machine, &x);
        config.run(max_steps);
        log_result(&config, max_steps);
        ExitCode::from(u8::from(!config.halted))
    } else {
        let settings =
            Settings { scale: args.scale, speed: args.speed, max_steps, max_cells: args.max_cells };
        window::show(machine, x, settings, move |run| log_result(run, max_steps))
    }
}
