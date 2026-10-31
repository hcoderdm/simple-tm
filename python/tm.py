# tm.py

from __future__ import annotations

import json
from dataclasses import dataclass
from functools import cached_property
from pathlib import Path

SIGMA = ("0", "1", "▷", "∅")
OFFSETS = {"L": -1, "R": 1, "S": 0, "H": 0}
"""How far each move takes a head."""


@dataclass
class TuringMachine:
    name: str
    description: str
    n_states: int
    n_tapes: int
    delta: dict[tuple[int, str], tuple[int, str, str]]

    @cached_property
    def table(self) -> list[dict[str, tuple[int, str, list[int], bool]]]:
        """delta prepared for running: table[s][sigma] = (s', sigma', head offsets, halts)."""
        table = [{} for _ in range(self.n_states)]
        for (s, sigma), (s2, sigma2, d) in self.delta.items():
            table[s][sigma] = (s2, sigma2, [OFFSETS[c] for c in d], "H" in d)
        return table


def check_machine(machine: TuringMachine) -> None:
    """States, tapes, and transitions match the definition of a machine."""
    k, m = machine.n_states, machine.n_tapes
    if k < 1:
        raise ValueError(f"Invalid {k}")
    if m < 1:
        raise ValueError(f"Invalid {m}")
    for (s, sigma), (s2, sigma2, d) in machine.delta.items():
        row = (s, sigma, s2, sigma2, d)
        if (not 0 <= s < k or not 0 <= s2 < k or len(sigma) != m or len(sigma2) != m or len(d) != m
                or not set(sigma + sigma2) <= set(SIGMA) or not set(d) <= set("LRSH")):
            raise ValueError(f"Invalid {row}")


def load_machine(path: Path) -> TuringMachine:
    """Read a machine from its JSON file."""
    data = json.loads(path.read_text(encoding="utf-8"))
    delta = {}
    for s, sigma, s2, sigma2, d in data["delta"]:
        if (s, sigma) in delta:
            raise ValueError(f"Invalid {(s, sigma)}")
        delta[s, sigma] = (s2, sigma2, d)
    machine = TuringMachine(data["name"], data["description"], data["n_states"], data["n_tapes"], delta)
    check_machine(machine)
    return machine


class Configuration:
    """M running on input x: the tapes T, head positions i, and state s."""

    def __init__(self, M: TuringMachine, x: str):
        self.M = M
        self.T = [["▷", *x, "∅"]] + [["▷", "∅"] for _ in range(M.n_tapes - 1)]
        self.i = [0] * M.n_tapes
        self.s = 0
        self.halted = False
        self.n_steps = 0

    def _run_one_tape(self, max_steps: int) -> None:
        table, tape = self.M.table, self.T[0]
        s, p, n, halted = self.s, self.i[0], 0, self.halted
        while n < max_steps and not halted:
            entry = table[s].get(tape[p])
            n += 1
            if entry is None:
                halted = True
                break
            s, tape[p], offsets, halted = entry
            p += offsets[0]
            if p < 0:
                p = 0
            elif p == len(tape):
                tape.append("∅")
        self.s, self.i[0], self.halted = s, p, halted
        self.n_steps += n

    def _run_many_tapes(self, max_steps: int) -> None:
        table, T, i = self.M.table, self.T, self.i
        s, n, halted = self.s, 0, self.halted
        while n < max_steps and not halted:
            entry = table[s].get("".join(map(list.__getitem__, T, i)))
            n += 1
            if entry is None:  # no entry: halt without changing anything
                halted = True
                break
            s, written, offsets, halted = entry
            for j, tape in enumerate(T):
                tape[i[j]] = written[j]
                p = i[j] + offsets[j]
                if p < 0:
                    p = 0
                elif p == len(tape):
                    tape.append("∅")
                i[j] = p
        self.s, self.halted = s, halted
        self.n_steps += n

    def run(self, max_steps: int) -> bool:
        """Run at most `max_steps` steps. Return True once M has halted."""
        
        if self.M.n_tapes == 1:
            # Specific optimization for n_tapes == 1.
            self._run_one_tape(max_steps)
        else:
            # General case for n_tapes > 1.
            self._run_many_tapes(max_steps)    

        return self.halted

    def output(self) -> str:
        """T_1[1] T_1[2] ... up to the first ∅."""
        return "".join(self.T[0][1:]).split("∅")[0]
