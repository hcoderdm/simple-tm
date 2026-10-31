# simple-tm

Runs Turing machines (as defined in [introtcs §7.1.2](https://introtcs.org/public/lec_06_loops.html)) from JSON files in the top-level `machines/` directory, prints the output, and draws the current step in a window: one row of pixels per tape, one pixel per tape cell.

```bash
uv run main.py identity 5                 # outputs its input, NtS(5) = 101
uv run main.py one 5                      # outputs 1 on every input
uv run main.py zero 5 --no-window         # just print the result
uv run main.py reverse 6 --scale 8        # a 2-tape machine: tapes stacked
uv run main.py reverse 300 --scale 4 --speed 60   # bigger pixels, slower replay
uv run main.py --help
uv run pytest                             # run the tests
```

To add a machine, put a JSON file in `../machines/` (see `machines/one.json` for the format, and `machines/reverse.json` for a machine with several tapes).
