//! Turing machines as defined in Barak, Introduction to Theoretical Computer Science, Section
//! 7.1.2 (<https://introtcs.org/public/lec_06_loops.html>), with m >= 1 tapes.
//!
//! A machine M with k states and m tapes over Sigma = {0, 1, ▷, ∅} is a transition function
//! delta_M : [k] x Sigma^m -> [k] x Sigma^m x {L, R, S, H}^m. Pairs (s, sigma) that are not
//! listed halt without changing anything, and the output is read off the first tape:
//! T_1[1] T_1[2] ... up to (not including) the first ∅.

use std::fmt;
use std::fs;
use std::path::Path;

use num_bigint::BigUint;
use serde::Deserialize;

/// The alphabet Sigma = {0, 1, ▷, ∅}, in this order.
pub const SIGMA: [char; 4] = ['0', '1', '▷', '∅'];

/// The most tapes a machine may have, which keeps its table of k * 4^m entries small.
pub const MAX_TAPES: usize = 8;

/// A symbol of Sigma, stored as its index in `SIGMA`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Symbol {
    Zero,
    One,
    Start,
    Blank,
}

impl Symbol {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '0' => Some(Self::Zero),
            '1' => Some(Self::One),
            '▷' => Some(Self::Start),
            '∅' => Some(Self::Blank),
            _ => None,
        }
    }

    pub fn to_char(self) -> char {
        SIGMA[self as usize]
    }
}

/// An error in a machine's definition or its JSON file.
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    Invalid(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::Invalid(what) => write!(f, "Invalid {what}"),
        }
    }
}

impl std::error::Error for Error {}

/// One row of delta: (s, sigma, s', sigma', D).
pub type Row = (usize, String, usize, String, String);

/// delta(s, sigma) prepared for running: the next state, the symbols to write, how far each head
/// moves, and whether the machine halts.
#[derive(Debug)]
struct Entry {
    state: usize,
    written: Vec<Symbol>,
    offsets: Vec<isize>,
    halts: bool,
}

#[derive(Debug)]
pub struct TuringMachine {
    pub name: String,
    pub description: String,
    pub n_states: usize,
    pub n_inputs: usize,
    pub n_tapes: usize,
    pub delta: Vec<Row>,
    /// table[s * 4^m + code(sigma)], where code reads sigma as a base-4 number.
    table: Vec<Option<Entry>>,
}

/// sigma as symbols, if every character is in Sigma.
fn symbols(sigma: &str) -> Option<Vec<Symbol>> {
    sigma.chars().map(Symbol::from_char).collect()
}

/// sigma read as a base-4 number, tape 1 first.
fn code(sigma: impl IntoIterator<Item = Symbol>) -> usize {
    sigma.into_iter().fold(0, |c, sigma| 4 * c + sigma as usize)
}

impl TuringMachine {
    /// A machine with these states, inputs, tapes, and transitions, checked against the
    /// definition of a machine.
    ///
    /// # Errors
    ///
    /// `Error::Invalid` if there are no states, inputs, or tapes, more than `MAX_TAPES` tapes, or
    /// a row of delta that does not fit them (or repeats an earlier row's (s, sigma)).
    pub fn new(
        name: &str,
        description: &str,
        n_states: usize,
        n_inputs: usize,
        n_tapes: usize,
        delta: Vec<Row>,
    ) -> Result<Self, Error> {
        let (k, m) = (n_states, n_tapes);
        if k < 1 {
            return Err(Error::Invalid(k.to_string()));
        }
        if n_inputs < 1 {
            return Err(Error::Invalid(n_inputs.to_string()));
        }
        if !(1..=MAX_TAPES).contains(&m) {
            return Err(Error::Invalid(m.to_string()));
        }

        let mut table: Vec<Option<Entry>> = (0..k << (2 * m)).map(|_| None).collect();
        for row in &delta {
            let (s, sigma, s2, sigma2, d) = row;
            let invalid = || Error::Invalid(format!("{row:?}"));
            let (Some(sigma), Some(written)) = (symbols(sigma), symbols(sigma2)) else {
                return Err(invalid());
            };
            let offsets: Option<Vec<isize>> = d
                .chars()
                .map(|c| match c {
                    'L' => Some(-1),
                    'R' => Some(1),
                    'S' | 'H' => Some(0),
                    _ => None,
                })
                .collect();
            let Some(offsets) = offsets else {
                return Err(invalid());
            };
            if *s >= k || *s2 >= k || sigma.len() != m || written.len() != m || offsets.len() != m {
                return Err(invalid());
            }
            let entry = &mut table[(s << (2 * m)) + code(sigma)];
            if entry.is_some() {
                return Err(invalid());
            }
            *entry = Some(Entry { state: *s2, written, offsets, halts: d.contains('H') });
        }

        Ok(Self {
            name: name.to_owned(),
            description: description.to_owned(),
            n_states,
            n_inputs,
            n_tapes,
            delta,
            table,
        })
    }
}

/// A machine's JSON file: {"name", "description", "n_states", "n_inputs", "n_tapes", "delta"}.
#[derive(Deserialize)]
struct MachineFile {
    name: String,
    description: String,
    n_states: usize,
    n_inputs: usize,
    n_tapes: usize,
    delta: Vec<Row>,
}

/// Read a machine from its JSON file.
///
/// # Errors
///
/// If the file cannot be read or parsed, or the machine is invalid (see `TuringMachine::new`).
pub fn load_machine(path: &Path) -> Result<TuringMachine, Error> {
    let text = fs::read_to_string(path).map_err(Error::Io)?;
    let file: MachineFile = serde_json::from_str(&text).map_err(Error::Json)?;
    TuringMachine::new(
        &file.name,
        &file.description,
        file.n_states,
        file.n_inputs,
        file.n_tapes,
        file.delta,
    )
}

/// x for the inputs n_1, ..., n_k: pf(NtS(n_1)) ... pf(NtS(n_{k-1})) NtS(n_k), where
/// pf(z) = z_0 z_0 z_1 z_1 ... z_{|z|-1} z_{|z|-1} 0 1 is the prefix-free encoding of introtcs
/// §2.5.2.
///
/// # Panics
///
/// If `numbers` is empty.
pub fn encode(numbers: &[BigUint]) -> String {
    let (last, init) = numbers.split_last().expect("at least one number");
    let mut x = String::new();
    for n in init {
        for c in n.to_str_radix(2).chars() {
            x.extend([c, c]);
        }
        x.push_str("01");
    }
    x + &last.to_str_radix(2)
}

/// M running on input x: the tapes T, head positions i, and state s.
#[derive(Debug)]
pub struct Configuration<'a> {
    pub machine: &'a TuringMachine,
    pub tapes: Vec<Vec<Symbol>>,
    pub heads: Vec<usize>,
    pub state: usize,
    pub halted: bool,
    pub n_steps: u64,
}

impl<'a> Configuration<'a> {
    /// M at its start on x: tape 1 is ▷ x ∅, every other tape ▷ ∅, all heads at 0, state 0.
    ///
    /// # Panics
    ///
    /// If x is not a binary string.
    pub fn new(machine: &'a TuringMachine, x: &str) -> Self {
        let x = symbols(x)
            .filter(|x| x.iter().all(|&c| matches!(c, Symbol::Zero | Symbol::One)))
            .expect("x is a binary string");
        let mut tapes = vec![[vec![Symbol::Start], x, vec![Symbol::Blank]].concat()];
        tapes.resize(machine.n_tapes, vec![Symbol::Start, Symbol::Blank]);
        Self {
            machine,
            tapes,
            heads: vec![0; machine.n_tapes],
            state: 0,
            halted: false,
            n_steps: 0,
        }
    }

    /// Run at most `max_steps` steps. Return true once M has halted.
    pub fn run(&mut self, max_steps: u64) -> bool {
        let m = self.machine.n_tapes;
        let mut n = 0;
        while n < max_steps && !self.halted {
            n += 1;
            let sigma = self.tapes.iter().zip(&self.heads).map(|(tape, &i)| tape[i]);
            let Some(entry) = &self.machine.table[(self.state << (2 * m)) + code(sigma)] else {
                // No entry: halt without changing anything.
                self.halted = true;
                break;
            };
            self.state = entry.state;
            for (j, tape) in self.tapes.iter_mut().enumerate() {
                let i = &mut self.heads[j];
                tape[*i] = entry.written[j];
                *i = i.saturating_add_signed(entry.offsets[j]);
                if *i == tape.len() {
                    tape.push(Symbol::Blank);
                }
            }
            self.halted = entry.halts;
        }
        self.n_steps += n;
        self.halted
    }

    /// T_1[1] T_1[2] ... up to the first ∅.
    pub fn output(&self) -> String {
        self.tapes[0][1..]
            .iter()
            .take_while(|&&sigma| sigma != Symbol::Blank)
            .map(|sigma| sigma.to_char())
            .collect()
    }
}
