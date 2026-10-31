# simple-tm

Runs Turing machines (as defined in [introtcs §7.1.2](https://introtcs.org/public/lec_06_loops.html)) from JSON files in the top-level `machines/` directory, prints the output, and draws the current step in a window: one row of pixels per tape, one pixel per tape cell.

```bash
uv run main.py identity 5                 # outputs its input, 5 = 101 in binary
uv run main.py one 5                      # outputs 1 on every input
uv run main.py zero 5 --no-window         # just print the result
uv run main.py reverse 6 --scale 8        # a 2-tape machine: tapes stacked
uv run main.py reverse 300 --scale 4 --speed 60   # bigger pixels, faster replay
uv run main.py add 5 3                    # machines with two inputs take two numbers
uv run main.py multiply 6 7 --scale 6
uv run main.py --help
uv run pytest                             # run the tests
```

Numbers are written in binary, most significant bit first and without leading zeros, both as input and as output, so a machine's description can say just F(a, b) = a + b. When a machine takes several numbers, every number but the last has each of its bits doubled and is followed by 01 (the prefix-free encoding of [introtcs §2.5.2](https://introtcs.org/public/lec_02_representation.html)), so `add 2 3` runs on 1100 01 11. The arithmetic machines `add`, `subtract` (truncated at 0), `multiply`, `divide` (rounded down), and `mod` take two numbers and output the result in binary; `divide` and `mod` never halt when dividing by 0. `compare` (a ≤ b) and `equal` (a = b) output 1 or 0. The logic gates `and`, `or`, `nand`, `nor`, `xor`, and `xnor` work bit by bit, padding the shorter number with 0s at the front, and `not` flips each bit of one number; their results are numbers too, so leading zeros are dropped (`xnor 5 1` compares 101 with 001 and outputs 11). `increment` outputs n + 1, and `palindrome` outputs 1 if the binary digits of n read the same backwards. See `ATTRIBUTIONS.md` for where the algorithms come from.

To add a machine, put a JSON file in `../machines/` (see `machines/one.json` for the format, `machines/reverse.json` for a machine with several tapes, and `machines/add.json` for one with two inputs).
