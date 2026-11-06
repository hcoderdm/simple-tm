use std::path::{Path, PathBuf};
use std::process::Command;

use num_bigint::BigUint;

use simple_tm::tm::{Configuration, Row, SIGMA, TuringMachine, encode};

fn machines() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../machines")
}

fn machine(name: &str) -> TuringMachine {
    TuringMachine::from_path(machines().join(format!("{name}.json"))).unwrap()
}

fn rows(delta: &[(usize, &str, usize, &str, &str)]) -> Vec<Row> {
    delta.iter().map(|&(s, a, s2, b, d)| (s, a.into(), s2, b.into(), d.into())).collect()
}

/// Every binary string of length n.
fn strings(n: usize) -> impl Iterator<Item = String> {
    (0..1u32 << n).map(move |k| if n == 0 { String::new() } else { format!("{k:0n$b}") })
}

#[test]
fn identity_one_and_zero() {
    let machines = [machine("identity"), machine("one"), machine("zero")];
    for n in 0..7 {
        for x in strings(n) {
            for (m, want) in machines.iter().zip([x.as_str(), "1", "0"]) {
                let mut run = Configuration::new(m, &x).unwrap();
                assert!(run.run(100) && run.output() == want, "{} {x}", m.name());
            }
        }
    }
}

#[test]
fn alphabet_is_strictly_binary() {
    assert_eq!(SIGMA, ['0', '1', '▷', '∅']);
}

#[test]
fn stay_move_keeps_the_head_in_place() {
    // S writes but does not move: tape 1 stays on cell 1 (writing 0 then 1), tape 2 moves right.
    let delta =
        rows(&[(0, "▷▷", 1, "▷▷", "RR"), (1, "1∅", 2, "0∅", "SR"), (2, "0∅", 2, "1∅", "SH")]);
    let m = TuringMachine::new("t", "", 3, 1, 2, &delta).unwrap();
    let mut run = Configuration::new(&m, "1").unwrap();
    assert!(run.run(10) && run.steps() == 3);
    assert_eq!(run.heads(), [1, 2]);
    assert_eq!(char::from(run.tapes()[0][1]), '1');
}

#[test]
fn moving_left_from_0_stays_at_0_and_output_stops_at_blank() {
    // delta(0, start) = (1, start, L) stays at 0; then state 1 moves right, blanking 1s, and halts on 0.
    let delta = rows(&[
        (0, "▷", 1, "▷", "L"),
        (1, "▷", 1, "▷", "R"),
        (1, "1", 1, "∅", "R"),
        (1, "0", 1, "0", "H"),
    ]);
    let m = TuringMachine::new("t", "", 2, 1, 1, &delta).unwrap();
    let mut run = Configuration::new(&m, "10").unwrap();
    run.run(1);
    assert_eq!(run.heads(), [0]);
    assert!(run.run(10) && run.output().is_empty());
}

#[test]
fn reverse_uses_two_tapes() {
    let reverse = machine("reverse");
    assert_eq!((reverse.num_states(), reverse.num_tapes()), (2, 2));
    for n in 0..8 {
        for x in strings(n) {
            let mut run = Configuration::new(&reverse, &x).unwrap();
            assert!(run.run(1000) && run.output() == x.chars().rev().collect::<String>(), "{x}");
            // Tape 2 still holds the copy.
            let copy: String =
                run.tapes()[1][1..=n].iter().map(|&sigma| char::from(sigma)).collect();
            assert_eq!(copy, x);
        }
    }
}

#[test]
fn multi_tape_halts_when_any_move_is_h() {
    let m = TuringMachine::new("t", "", 1, 1, 2, &rows(&[(0, "▷▷", 0, "▷▷", "RH")])).unwrap();
    let mut run = Configuration::new(&m, "1").unwrap();
    assert!(run.run(10) && run.steps() == 1);
    assert_eq!(run.heads(), [1, 0]);
}

#[test]
fn new_checks_states_are_below_n_states() {
    assert!(TuringMachine::new("bad", "", 2, 1, 1, &rows(&[(0, "▷", 2, "▷", "R")])).is_err());
}

#[test]
fn new_rejects_repeated_rows() {
    let delta = rows(&[(0, "▷", 0, "▷", "R"), (0, "▷", 0, "▷", "L")]);
    assert!(TuringMachine::new("bad", "", 1, 1, 1, &delta).is_err());
}

#[test]
fn encode_uses_prefix_free_pairs() {
    let encode = |ns: &[u32]| encode(&ns.iter().map(|&n| BigUint::from(n)).collect::<Vec<_>>());
    assert_eq!(encode(&[6]), "110");
    assert_eq!(encode(&[2, 3]), "1100".to_owned() + "01" + "11");
    assert_eq!(encode(&[0, 1, 5]), "00".to_owned() + "01" + "11" + "01" + "101");
}

#[test]
fn every_machine_loads() {
    for entry in machines().read_dir().unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "json") {
            TuringMachine::from_path(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        }
    }
}

fn simple_tm(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_simple-tm")).args(args).output().unwrap()
}

#[test]
fn check_args_rejects_bad_values() {
    for args in [
        ["identity", "-1", "--no-window"].as_slice(),
        &["identity", "5", "--scale", "0", "--no-window"],
        &["identity", "5", "--speed", "0", "--no-window"],
        &["identity", "5", "--max-steps", "0", "--no-window"],
        &["identity", "5", "--max-cells", "0", "--no-window"],
        &["identity", "5", "3", "--no-window"],
    ] {
        assert!(!simple_tm(args).status.success(), "{args:?}");
    }
}

#[test]
fn main_logs_result() {
    let output = simple_tm(&["reverse", "6", "--no-window"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "011\n"); // M(x) for x = NtS(6) = 110
}

/// F(a, b), or None where F is undefined.
type TwoInputs = fn(u32, u32) -> Option<u32>;

/// F(n).
type OneInput = fn(u32) -> u32;

/// 1 in every bit of NtS(n), so gates pad the shorter input with 0s at the front.
fn mask(n: u32) -> u32 {
    (1 << (u32::BITS - n.leading_zeros()).max(1)) - 1
}

#[test]
fn two_inputs() {
    let machines: [(&str, TwoInputs); 13] = [
        ("add", |a, b| Some(a + b)),
        ("subtract", |a, b| Some(a.saturating_sub(b))),
        ("multiply", |a, b| Some(a * b)),
        ("divide", |a, b| a.checked_div(b)),
        ("mod", |a, b| a.checked_rem(b)),
        ("compare", |a, b| Some(u32::from(a <= b))),
        ("equal", |a, b| Some(u32::from(a == b))),
        ("and", |a, b| Some(a & b)),
        ("or", |a, b| Some(a | b)),
        ("xor", |a, b| Some(a ^ b)),
        ("nand", |a, b| Some(!(a & b) & mask(a.max(b)))),
        ("nor", |a, b| Some(!(a | b) & mask(a.max(b)))),
        ("xnor", |a, b| Some(!(a ^ b) & mask(a.max(b)))),
    ];
    for (name, f) in machines {
        let m = machine(name);
        assert_eq!(m.num_inputs(), 2);
        for a in 0..32 {
            for b in 0..32 {
                let mut run = Configuration::new(&m, &encode(&[a.into(), b.into()])).unwrap();
                match f(a, b) {
                    // Undefined: M never halts.
                    None => assert!(!run.run(10_000), "{name} {a} {b}"),
                    Some(want) => {
                        assert!(run.run(1_000_000), "{name} {a} {b}");
                        assert_eq!(run.output(), format!("{want:b}"), "{name} {a} {b}");
                    }
                }
            }
        }
    }
}

#[test]
fn one_input() {
    let machines: [(&str, OneInput); 3] = [
        ("increment", |n| n + 1),
        ("palindrome", |n| {
            let x = format!("{n:b}");
            u32::from(x.chars().eq(x.chars().rev()))
        }),
        ("not", |n| !n & mask(n)),
    ];
    for (name, f) in machines {
        let m = machine(name);
        assert_eq!(m.num_inputs(), 1);
        for n in 0..1024 {
            let mut run = Configuration::new(&m, &encode(&[n.into()])).unwrap();
            assert!(run.run(1_000_000), "{name} {n}");
            assert_eq!(run.output(), format!("{:b}", f(n)), "{name} {n}");
        }
    }
}

#[test]
fn new_rejects_non_binary_input() {
    let m = machine("identity");
    assert!(Configuration::new(&m, "12").is_err());
    assert!(Configuration::new(&m, "▷").is_err());
}
