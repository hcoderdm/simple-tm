"""Run a Turing machine from machines/<name>.json on a natural number, log M(x), and draw the config.

    uv run main.py reverse 6

The number n becomes the input x = NtS(n), its binary representation (most significant bit first).
"""

from __future__ import annotations

import argparse
import logging
from pathlib import Path

from tm import SIGMA, Configuration, load_machine
from window import Window

MACHINES_PATH = Path(__file__).parent.parent / "machines"
LOGGER = logging.getLogger(__name__)


def parse_args() -> argparse.Namespace:
    """Parse the arguments."""

    # Get the names of the machines.
    names = sorted(path.stem for path in MACHINES_PATH.glob("*.json"))
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    
    # Add arguments to the parser.
    parser.add_argument("machine", choices=names)
    parser.add_argument("n", type=int)
    parser.add_argument("--no-window", action="store_true")
    parser.add_argument("--scale", type=int, default=1)
    parser.add_argument("--speed", type=float, default=240)
    parser.add_argument("--max-steps", type=int, default=10**7)
    parser.add_argument("--max-cells", type=int, default=1000)

    args = parser.parse_args()
    check_args(args)
    return args


def check_args(args: argparse.Namespace) -> None:
    """Check the arguments."""
    if args.n < 0:
        raise SystemExit(f"Invalid {args.n=}")
    if args.scale < 1:
        raise SystemExit(f"Invalid {args.scale=}")
    if args.speed <= 0:
        raise SystemExit(f"Invalid {args.speed=}")
    if args.max_steps < 1:
        raise SystemExit(f"Invalid {args.max_steps=}")
    if args.max_cells < 1:
        raise SystemExit(f"Invalid {args.max_cells=}")


def log_result(config: Configuration, max_steps: int) -> None:
    """Log M(x) if the machine halted, or that it did not halt within `max_steps` steps."""
    if config.halted:
        LOGGER.info("%s", config.output())
    else:
        LOGGER.warning("M did not halt within %s steps", f"{max_steps:,}")


def main() -> int:
    
    logging.basicConfig(level=logging.INFO, format="%(message)s")
    args = parse_args()
    
    # Load the machine.
    M = load_machine(MACHINES_PATH / f"{args.machine}.json")
    x = f"{args.n:b}"

    # Run the machine.
    config = Configuration(M, x)
    if args.no_window:
        config.run(args.max_steps)
        log_result(config, args.max_steps)
    else:
        window = Window(
            config, 
            args.scale, 
            args.speed, 
            args.max_steps, 
            args.max_cells,
            on_stop=lambda run: log_result(run, args.max_steps)
            )
        window.root.mainloop()
        config = window.run

    return not config.halted

if __name__ == "__main__":
    raise SystemExit(main())
