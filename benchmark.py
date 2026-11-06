"""Time the Python and Rust runners on the same machines and inputs, without the window.

    python3 benchmark.py

Builds the Rust runner in release mode, runs each command `REPEATS` times, and prints a Markdown
table of the median wall-clock times (process start-up included).
"""

from __future__ import annotations

import random
import statistics
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).parent
sys.path.insert(0, str(ROOT / "python"))

from tm import Configuration, encode, load_machine  # noqa: E402

REPEATS = 5
MAX_STEPS = 10**10
RUST = ROOT / "rust" / "target" / "release" / "simple-tm"


def bits(n: int, rng: random.Random) -> int:
    """A random number with exactly n bits."""
    return rng.getrandbits(n) | 1 << (n - 1)


def workloads() -> list[tuple[str, list[int]]]:
    rng = random.Random(0)
    runs = [("identity", [bits(64, rng)]), ("reverse", [bits(2000, rng)])]
    for name in ("add", "subtract"):
        runs += [(name, [bits(2000, rng), bits(2000, rng)])]
    for n in (64, 512, 1024):
        runs += [("multiply", [bits(n, rng), bits(n, rng)])]
    for n in (128, 1024):
        runs += [("divide", [bits(2 * n, rng), bits(n, rng)])]
    return [(name, numbers) for name, numbers in runs if (ROOT / "machines" / f"{name}.json").exists()]


def median_time(command: list[str], cwd: Path, want: str) -> float:
    """Median seconds to run `command`, checking that it logs `want`."""
    times = []
    for _ in range(REPEATS):
        start = time.perf_counter()
        result = subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=True)
        times.append(time.perf_counter() - start)
        if result.stderr.strip() != want:
            raise SystemExit(f"{command[:2]} logged {result.stderr.strip()[:50]!r}, expected {want[:50]!r}")
    return statistics.median(times)


def main() -> None:
    subprocess.run(["cargo", "build", "--release", "--quiet"], cwd=ROOT / "rust", check=True)
    print("| machine | input bits | steps | Python (s) | Rust (s) | speed-up | Python steps/s | Rust steps/s |")
    print("|---|---|---|---|---|---|---|---|")
    for name, numbers in workloads():
        config = Configuration(load_machine(ROOT / "machines" / f"{name}.json"), encode(numbers))
        config.run(MAX_STEPS)
        args = [name, *map(str, numbers), "--no-window", "--max-steps", str(MAX_STEPS)]
        py = median_time([sys.executable, "main.py", *args], ROOT / "python", config.output())
        rs = median_time([str(RUST), *args], ROOT / "rust", config.output())
        n = config.n_steps
        sizes = " × ".join(str(m.bit_length()) for m in numbers)
        print(f"| {name} | {sizes} | {n:,} | {py:.3f} | {rs:.4f} | {py / rs:.0f}× | {n / py:,.0f} | {n / rs:,.0f} |")


if __name__ == "__main__":
    main()
