# simple-tm

Runs Turing machines (as defined in [introtcs §7.1.2](https://introtcs.org/public/lec_06_loops.html)) from the JSON files in `machines/`, prints the output, and draws each step in a pixel window. There are two runners with the same options and behavior: `python/` and `rust/`. Each has its own README.

## Python vs. Rust

`python3 benchmark.py` runs both runners with `--no-window` on the same machines and inputs (random numbers from a fixed seed). The table shows the median of 5 runs, including process start-up, measured on an Apple M4 with Python 3.14 and Rust 1.99 (release build).

| machine | input bits | steps | Python (s) | Rust (s) | speed-up | Python steps/s | Rust steps/s |
|---|---|---|---|---|---|---|---|
| identity | 64 | 1 | 0.135 | 0.0091 | 15× | 7 | 110 |
| reverse | 2000 | 6,004 | 0.137 | 0.0088 | 16× | 43,758 | 684,984 |
| add | 2000 × 2000 | 14,009 | 0.149 | 0.0101 | 15× | 94,045 | 1,390,955 |
| subtract | 2000 × 2000 | 14,008 | 0.149 | 0.0098 | 15× | 94,328 | 1,423,402 |
| multiply | 64 × 64 | 13,064 | 0.151 | 0.0093 | 16× | 86,522 | 1,412,013 |
| multiply | 512 × 512 | 791,561 | 0.770 | 0.0236 | 33× | 1,028,385 | 33,511,433 |
| multiply | 1024 × 1024 | 3,158,024 | 2.605 | 0.0650 | 40× | 1,212,222 | 48,553,798 |
| divide | 256 × 128 | 34,827 | 0.152 | 0.0081 | 19× | 228,813 | 4,314,277 |
| divide | 2048 × 1024 | 2,113,547 | 1.940 | 0.0481 | 40× | 1,089,535 | 43,920,484 |

On short runs the gap is mostly start-up time: about 0.14 s for Python and under 0.01 s for Rust. On long runs it is the speed of stepping the machine.
