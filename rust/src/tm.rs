//! Turing machines as defined in Barak, Introduction to Theoretical Computer Science, Section
//! 7.1.2 (<https://introtcs.org/public/lec_06_loops.html>), with m >= 1 tapes.
//!
//! A machine M with k states and m tapes over Sigma = {0, 1, ▷, ∅} is a transition function
//! delta_M : [k] x Sigma^m -> [k] x Sigma^m x {L, R, S, H}^m. Pairs (s, sigma) that are not
//! listed halt without changing anything, and the output is read off the first tape:
//! T_1[1] T_1[2] ... up to (not including) the first ∅.

use std::fmt;
use std::fs;
use std::io;
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

impl TryFrom<char> for Symbol {
    type Error = char;

    fn try_from(c: char) -> Result<Self, char> {
        match c {
            '0' => Ok(Self::Zero),
            '1' => Ok(Self::One),
            '▷' => Ok(Self::Start),
            '∅' => Ok(Self::Blank),
            _ => Err(c),
        }
    }
}

impl From<Symbol> for char {
    fn from(sigma: Symbol) -> char {
        SIGMA[sigma as usize]
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

/// A head move: one of {L, R, S, H}.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Move {
    Left,
    Right,
    Stay,
    Halt,
}

impl TryFrom<char> for Move {
    type Error = char;

    fn try_from(c: char) -> Result<Self, char> {
        match c {
            'L' => Ok(Self::Left),
            'R' => Ok(Self::Right),
            'S' => Ok(Self::Stay),
            'H' => Ok(Self::Halt),
            _ => Err(c),
        }
    }
}

impl Move {
    /// How far the head moves.
    fn offset(self) -> isize {
        match self {
            Self::Left => -1,
            Self::Right => 1,
            Self::Stay | Self::Halt => 0,
        }
    }
}

/// An error in a machine's definition, its JSON file, or its input.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Json(serde_json::Error),
    NoStates,
    NoInputs,
    Tapes(usize),
    Row { row: Row, reason: &'static str },
    Input(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::NoStates => write!(f, "a machine needs at least 1 state"),
            Self::NoInputs => write!(f, "a machine needs at least 1 input"),
            Self::Tapes(m) => write!(f, "a machine needs 1 to {MAX_TAPES} tapes, not {m}"),
            Self::Row { row, reason } => write!(f, "invalid row {row:?}: {reason}"),
            Self::Input(x) => write!(f, "input {x:?} is not a binary string"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// One row of delta as written in a machine's JSON file: (s, sigma, s', sigma', D).
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
    name: String,
    description: String,
    num_states: usize,
    num_inputs: usize,
    num_tapes: usize,
    /// table[s * 4^m + code(sigma)], where code reads sigma as a base-4 number.
    table: Vec<Option<Entry>>,
}

/// The characters of `s`, if each one converts to a `T`.
fn parse<T: TryFrom<char>>(s: &str) -> Option<Vec<T>> {
    s.chars().map(|c| T::try_from(c).ok()).collect()
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
    /// If there are no states or inputs, not 1 to `MAX_TAPES` tapes, or a row of delta that does
    /// not fit them (or repeats an earlier row's (s, sigma)).
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        num_states: usize,
        num_inputs: usize,
        num_tapes: usize,
        delta: &[Row],
    ) -> Result<Self, Error> {
        let (k, m) = (num_states, num_tapes);
        if k < 1 {
            return Err(Error::NoStates);
        }
        if num_inputs < 1 {
            return Err(Error::NoInputs);
        }
        if !(1..=MAX_TAPES).contains(&m) {
            return Err(Error::Tapes(m));
        }

        let mut table: Vec<Option<Entry>> = (0..k << (2 * m)).map(|_| None).collect();
        for row in delta {
            let (s, sigma, s2, sigma2, d) = row;
            let invalid = |reason| Error::Row { row: row.clone(), reason };
            let (Some(sigma), Some(written)) = (parse::<Symbol>(sigma), parse::<Symbol>(sigma2))
            else {
                return Err(invalid("symbols must be in {0, 1, ▷, ∅}"));
            };
            let Some(moves) = parse::<Move>(d) else {
                return Err(invalid("moves must be in {L, R, S, H}"));
            };
            if *s >= k || *s2 >= k {
                return Err(invalid("state out of range"));
            }
            if sigma.len() != m || written.len() != m || moves.len() != m {
                return Err(invalid("needs one symbol and move per tape"));
            }
            let entry = &mut table[(s << (2 * m)) + code(sigma)];
            if entry.is_some() {
                return Err(invalid("repeats an earlier (s, sigma)"));
            }
            *entry = Some(Entry {
                state: *s2,
                written,
                offsets: moves.iter().map(|d| d.offset()).collect(),
                halts: moves.contains(&Move::Halt),
            });
        }

        Ok(Self {
            name: name.into(),
            description: description.into(),
            num_states,
            num_inputs,
            num_tapes,
            table,
        })
    }

    /// Read a machine from its JSON file: {"name", "description", "n_states", "n_inputs",
    /// "n_tapes", "delta"}.
    ///
    /// # Errors
    ///
    /// If the file cannot be read or parsed, or the machine is invalid (see `TuringMachine::new`).
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file: MachineFile = serde_json::from_str(&fs::read_to_string(path)?)?;
        Self::new(
            file.name,
            file.description,
            file.n_states,
            file.n_inputs,
            file.n_tapes,
            &file.delta,
        )
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub fn num_states(&self) -> usize {
        self.num_states
    }

    #[must_use]
    pub fn num_inputs(&self) -> usize {
        self.num_inputs
    }

    #[must_use]
    pub fn num_tapes(&self) -> usize {
        self.num_tapes
    }

    /// delta(s, sigma), or None if it is not listed.
    fn delta(&self, s: usize, sigma: impl IntoIterator<Item = Symbol>) -> Option<&Entry> {
        self.table[(s << (2 * self.num_tapes)) + code(sigma)].as_ref()
    }
}

/// A machine's JSON file.
#[derive(Deserialize)]
struct MachineFile {
    name: String,
    description: String,
    n_states: usize,
    n_inputs: usize,
    n_tapes: usize,
    delta: Vec<Row>,
}

/// x for the inputs n_1, ..., n_k: pf(NtS(n_1)) ... pf(NtS(n_{k-1})) NtS(n_k), where
/// pf(z) = z_0 z_0 z_1 z_1 ... z_{|z|-1} z_{|z|-1} 0 1 is the prefix-free encoding of introtcs
/// §2.5.2.
///
/// # Panics
///
/// If `numbers` is empty.
#[must_use]
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
#[derive(Clone, Debug)]
pub struct Configuration<'a> {
    machine: &'a TuringMachine,
    tapes: Vec<Vec<Symbol>>,
    heads: Vec<usize>,
    state: usize,
    halted: bool,
    steps: u64,
}

impl<'a> Configuration<'a> {
    /// M at its start on x: tape 1 is ▷ x ∅, every other tape ▷ ∅, all heads at 0, state 0.
    ///
    /// # Errors
    ///
    /// `Error::Input` if x is not a binary string.
    pub fn new(machine: &'a TuringMachine, x: &str) -> Result<Self, Error> {
        let x = parse::<Symbol>(x)
            .filter(|x| x.iter().all(|&c| matches!(c, Symbol::Zero | Symbol::One)))
            .ok_or_else(|| Error::Input(x.to_owned()))?;
        let mut tapes = vec![[vec![Symbol::Start], x, vec![Symbol::Blank]].concat()];
        tapes.resize(machine.num_tapes, vec![Symbol::Start, Symbol::Blank]);
        Ok(Self {
            machine,
            tapes,
            heads: vec![0; machine.num_tapes],
            state: 0,
            halted: false,
            steps: 0,
        })
    }

    /// Run at most `max_steps` steps. Return true once M has halted.
    pub fn run(&mut self, max_steps: u64) -> bool {
        let mut n = 0;
        while n < max_steps && !self.halted {
            n += 1;
            let sigma = self.tapes.iter().zip(&self.heads).map(|(tape, &i)| tape[i]);
            let Some(entry) = self.machine.delta(self.state, sigma) else {
                // No entry: halt without changing anything.
                self.halted = true;
                break;
            };
            self.state = entry.state;
            for (j, (tape, i)) in self.tapes.iter_mut().zip(&mut self.heads).enumerate() {
                tape[*i] = entry.written[j];
                *i = i.saturating_add_signed(entry.offsets[j]);
                if *i == tape.len() {
                    tape.push(Symbol::Blank);
                }
            }
            self.halted = entry.halts;
        }
        self.steps += n;
        self.halted
    }

    /// T_1[1] T_1[2] ... up to the first ∅.
    #[must_use]
    pub fn output(&self) -> String {
        self.tapes[0][1..]
            .iter()
            .take_while(|&&sigma| sigma != Symbol::Blank)
            .map(|&sigma| char::from(sigma))
            .collect()
    }

    #[must_use]
    pub fn machine(&self) -> &'a TuringMachine {
        self.machine
    }

    /// The tapes, each as far as it has been written or visited.
    #[must_use]
    pub fn tapes(&self) -> &[Vec<Symbol>] {
        &self.tapes
    }

    #[must_use]
    pub fn heads(&self) -> &[usize] {
        &self.heads
    }

    #[must_use]
    pub fn state(&self) -> usize {
        self.state
    }

    #[must_use]
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// The number of steps run so far.
    #[must_use]
    pub fn steps(&self) -> u64 {
        self.steps
    }
}
