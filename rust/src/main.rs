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

use clap::builder::{PossibleValuesParser, RangedU64ValueParser};
use clap::{Parser, value_parser};
use num_bigint::BigUint;

use simple_tm::tm::{Configuration, TuringMachine, encode};
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

/// A float greater than 0.
fn positive(s: &str) -> Result<f64, String> {
    match s.parse::<f64>() {
        Ok(x) if x > 0.0 => Ok(x),
        Ok(_) => Err("must be greater than 0".to_owned()),
        Err(e) => Err(e.to_string()),
    }
}

/// Run a Turing machine from machines/<name>.json on natural numbers, log M(x), and draw the
/// config.
#[derive(Parser)]
#[command(allow_negative_numbers = true)]
struct Args {
    /// The machine to run, from machines/<name>.json.
    #[arg(value_parser = PossibleValuesParser::new(machine_names()))]
    machine: String,
    /// The natural numbers n_1, ..., n_k to encode as the input x.
    #[arg(required = true)]
    n: Vec<BigUint>,
    /// Print M(x) without opening the window.
    #[arg(long)]
    no_window: bool,
    /// Pixels per tape cell.
    #[arg(long, default_value_t = 1, value_parser = RangedU64ValueParser::<usize>::new().range(1..))]
    scale: usize,
    /// Steps per second.
    #[arg(long, default_value_t = 10.0, value_parser = positive)]
    speed: f64,
    /// Give up after this many steps.
    #[arg(long, default_value_t = 10_000_000, value_parser = value_parser!(u64).range(1..))]
    max_steps: u64,
    /// The most cells of each tape to show.
    #[arg(long, default_value_t = 32, value_parser = RangedU64ValueParser::<usize>::new().range(1..))]
    max_cells: usize,
}

/// Log M(x) if the machine halted, or that it did not halt within `max_steps` steps.
fn log_result(config: &Configuration, max_steps: u64) {
    if config.is_halted() {
        eprintln!("{}", config.output());
    } else {
        eprintln!("M did not halt within {} steps", window::thousands(max_steps));
    }
}

fn main() -> ExitCode {
    let args = Args::parse();

    // Load the machine.
    let path = Path::new(MACHINES_PATH).join(format!("{}.json", args.machine));
    let machine = match TuringMachine::from_path(&path) {
        Ok(machine) => machine,
        Err(e) => {
            eprintln!("{}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    if args.n.len() != machine.num_inputs() {
        eprintln!(
            "{} takes {} numbers, got {}",
            machine.name(),
            machine.num_inputs(),
            args.n.len()
        );
        return ExitCode::FAILURE;
    }
    let x = encode(&args.n);

    // Run the machine.
    let max_steps = args.max_steps;
    if args.no_window {
        let mut config = Configuration::new(&machine, &x).expect("encode gives a binary string");
        config.run(max_steps);
        log_result(&config, max_steps);
        ExitCode::from(u8::from(!config.is_halted()))
    } else {
        let settings =
            Settings { scale: args.scale, speed: args.speed, max_steps, max_cells: args.max_cells };
        window::show(machine, x, settings, move |run| log_result(run, max_steps))
    }
}
