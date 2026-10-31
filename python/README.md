# simple-tm

Runs Turing machines (as defined in [introtcs §7.1.2](https://introtcs.org/public/lec_06_loops.html)) from JSON files in the top-level `machines/` directory, prints the output, and draws the current step in a window: one row of pixels per tape, one pixel per tape cell.

```bash
uv run main.py identity 5                 # outputs its input, NtS(5) = 101
uv run main.py one 5                      # outputs 1 on every input
uv run main.py zero 5 --no-window         # just print the result
uv run main.py reverse 6 --scale 8        # a 2-tape machine: tapes stacked
uv run main.py reverse 300 --scale 4 --speed 60   # bigger pixels, faster replay
uv run main.py add 5 3                    # machines with two inputs take two numbers
uv run main.py multiply 6 7 --scale 6
uv run main.py --help
uv run pytest                             # run the tests
```

A machine with `"n_inputs": 1` runs on x = NtS(n), the binary representation of n. A machine with k > 1 inputs runs on x = pf(NtS(n_1)) ⋯ pf(NtS(n_{k-1})) NtS(n_k), where pf(z) = z_0 z_0 z_1 z_1 ⋯ z_{|z|-1} z_{|z|-1} 0 1 is the prefix-free encoding of [introtcs §2.5.2](https://introtcs.org/public/lec_02_representation.html). The arithmetic machines `add`, `subtract` (truncated at 0), `multiply`, and `divide` (rounded down; never halts when dividing by 0) take two numbers and output the result in binary; see `ATTRIBUTIONS.md` for where their algorithms come from.

To add a machine, put a JSON file in `../machines/` (see `machines/one.json` for the format, `machines/reverse.json` for a machine with several tapes, and `machines/add.json` for one with two inputs).
