# simple-tm (Rust)

A port of the Python runner in `../python`: it runs the same Turing machines (as defined in [introtcs §7.1.2](https://introtcs.org/public/lec_06_loops.html)) from the JSON files in the top-level `machines/` directory, with the same command-line options, input encoding, output, and pixel window.

```bash
cargo run --release -- identity 5                 # outputs its input, 5 = 101 in binary
cargo run --release -- reverse 6 --scale 8        # a 2-tape machine: tapes stacked
cargo run --release -- add 5 3 --no-window        # just print the result
cargo run --release -- multiply 6 7 --scale 6
cargo run --release -- --help
cargo test                                        # run the tests
```

The code mirrors the Python modules: `src/tm.rs` is `tm.py` (machines, loading, `encode`, and `Configuration`), `src/window.rs` is `window.py` (drawn with [macroquad](https://macroquad.rs) instead of tkinter), and `src/main.rs` is `main.py` (arguments parsed with [clap](https://docs.rs/clap)). The window is meant to look the same as the Python one: its labels use the font Tk picks on macOS (Menlo at 11 pixels, read from the system, with Tk's 7 × 13 pixel character grid and label padding), and it draws at full Retina resolution as Tk does. The differences are small: the window grows up to a fixed 1400 × 800 rather than to the size of the screen, which macroquad cannot report, and a machine may have at most 8 tapes, so that delta fits in a table of k · 4^m entries indexed directly by the state and the symbols under the heads.

See the top-level `README.md` for a speed comparison with the Python runner.
