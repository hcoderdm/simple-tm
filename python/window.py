# window.py

from __future__ import annotations

import sys
import time
import tkinter as tk
from typing import Callable

from tm import Configuration

# Colors from Open Color by heeyeun (MIT License), https://github.com/yeun/open-color.
# See ATTRIBUTIONS.md at the top of the repository.
COLOR_1 = "#ffffff"  # white
COLOR_0 = "#000000"  # black
COLOR_EMPTY = "#343a40"  # gray 8
COLOR_START = "#339af0"  # blue 5
COLOR_HEADS = ["#ff8787", "#ff6b6b", "#fa5252", "#f03e3e", "#e03131", "#c92a2a"]  # red 4 to red 9
COLOR_TEXT = "#adb5bd"  # gray 5
BACKGROUND = "#212529"  # gray 9, around the tapes
SYMBOL_COLOR = {"1": COLOR_1, "0": COLOR_0, "∅": COLOR_EMPTY, "▷": COLOR_START}

GAP = 3  # pixels between tapes
MIN_SIZE = (360, 240)
FRAME_BUDGET = 0.012  # seconds of stepping per tick
RETURN_KEY = "RETURN" if sys.platform == "darwin" else "ENTER"
SHIFTED_DIGITS = "!@#$%^&*("  # Shift + 1 .. Shift + 9 on a US keyboard


def row_colors(T: list[str], i: int, s: int) -> list[str]:
    """Colors for one tape, with the head at i drawn in the shade of red for state s."""
    colors = [SYMBOL_COLOR[sigma] for sigma in T]
    colors[i] = COLOR_HEADS[s % len(COLOR_HEADS)]
    return colors


def shift_digit(event: tk.Event) -> int | None:
    """n if the key is Shift + n (n = 1..9), else None."""
    if event.char and event.char in SHIFTED_DIGITS:
        return SHIFTED_DIGITS.index(event.char) + 1
    if event.state & 1 and event.char and event.char in "123456789":
        return int(event.char)
    return None


class Window:
    """Runs `run` for at most `max_steps` steps while drawing it, showing at most `max_cells`
    cells of each tape (fewer if they would not fit on the screen); calls `on_stop(run)` once
    per run when the machine halts or reaches `max_steps`.

    Keys: Escape closes the window; Return finishes the run without drawing, reports the result,
    and closes; holding Shift + n (n = 1..9) runs at n times the starting speed; R (once halted)
    starts the machine again on the same input."""

    def __init__(
            self,
            run: Configuration,
            scale: int,
            speed: float,
            max_steps: int,
            max_cells: int,
            on_stop: Callable[[Configuration], None] | None = None
        ):
        self.scale, self.speed, self.base_speed = scale, speed, speed
        self.max_steps, self.on_stop = max_steps, on_stop
        self.input = "".join(run.T[0][1:-1])  # x, from the starting tape 0 = ▷ x ∅

        self.root = tk.Tk()
        self.root.title("Turing Machine")
        self.root.protocol("WM_DELETE_WINDOW", self.close)
        self.root.bind("<Escape>", lambda _: self.close())
        self.root.bind("q", lambda _: self.close())
        self.root.bind("<Return>", lambda _: self.finish())
        self.root.bind("<KP_Enter>", lambda _: self.finish())
        self.root.bind("r", lambda _: self.restart())
        self.root.bind("R", lambda _: self.restart())
        self.root.bind("<Key>", self.on_key)
        self.root.bind("<KeyRelease>", self.on_key_release)

        self.canvas = tk.Canvas(self.root, background=BACKGROUND, highlightthickness=0, borderwidth=0, width=MIN_SIZE[0], height=MIN_SIZE[1])
        self.canvas.pack(fill="both", expand=True)
        self.canvas.bind("<Configure>", lambda _: self.fit())

        text = {"foreground": COLOR_TEXT, "background": BACKGROUND, "font": "TkFixedFont"}
        self.info = tk.Label(self.root, justify="left", **text)
        self.info.place(in_=self.canvas, x=6, y=4)
        self.keys = tk.Label(self.root, justify="right", **text)
        self.keys.place(in_=self.canvas, relx=1.0, x=-6, y=4, anchor="ne")

        screen_w, screen_h = self.root.winfo_screenwidth(), self.root.winfo_screenheight()
        self.max_size = (screen_w - 100, screen_h - 200)
        self.cells = min(max_cells, self.max_size[0] // scale)
        self.root.geometry(f"+{(screen_w - MIN_SIZE[0]) // 2}+{(screen_h - MIN_SIZE[1]) // 3}")
        self.root.lift()
        self.root.attributes("-topmost", True)
        self.root.after_idle(self.root.attributes, "-topmost", False)
        self.root.focus_force()

        self.item = self.canvas.create_image(0, 0, anchor="center")
        self.pending = None
        self.start(run)

    def start(self, run: Configuration) -> None:
        """Begin drawing `run` from a clean image."""
        self.run, self.reported = run, False
        self.image = tk.PhotoImage(master=self.root)
        self.canvas.itemconfigure(self.item, image=self.image)
        self.width = 0
        self.offsets = [0] * run.M.n_tapes
        self.carry, self.last = 0.0, time.perf_counter()
        self.tick()

    def close(self) -> None:
        if self.pending:
            self.root.after_cancel(self.pending)
        self.root.destroy()

    def finish(self) -> None:
        """Run the machine until it halts or reaches `max_steps` (without drawing), report the result, and close the window."""
        self.run.run(self.max_steps - self.run.n_steps)
        self.report()
        self.close()

    def report(self) -> None:
        """Call `on_stop` for the current run, once."""
        if self.on_stop and not self.reported:
            self.on_stop(self.run)
            self.reported = True

    def restart(self) -> None:
        """Once the machine has halted, start it again on the same input."""
        if self.run.halted:
            self.start(Configuration(self.run.M, self.input))

    def on_key(self, event: tk.Event) -> None:
        if n := shift_digit(event):
            self.speed = self.base_speed * n

    def on_key_release(self, event: tk.Event) -> None:
        if event.keysym in ("Shift_L", "Shift_R") or shift_digit(event):
            self.speed = self.base_speed

    def size(self) -> tuple[int, int]:
        """Width and height of the drawing, in pixels."""
        k, m = self.scale, len(self.run.T)
        return self.width * k, m * k + GAP * (m - 1)

    def draw(self) -> None:
        """Draw the view of each tape as `scale` rows of pixels, tape 0 on top."""
        k, run = self.scale, self.run
        for j, (T, i) in enumerate(zip(run.T, run.i)):
            o = self.offsets[j] = min(max(self.offsets[j], i - self.cells + 1), i)
            view = T[o : o + self.cells]
            self.width = max(self.width, len(view))
            row = "{" + " ".join(c for c in row_colors(view, i - o, run.s) for _ in range(k)) + "}"
            self.image.put(" ".join([row] * k), to=(0, j * (k + GAP)))

    def fit(self) -> None:
        """Grow the window to fit the drawing (up to the screen) and center the drawing in it."""
        w, h = self.size()
        asked_w, asked_h = int(self.canvas.cget("width")), int(self.canvas.cget("height"))
        want_w = min(max(MIN_SIZE[0], w), self.max_size[0])
        want_h = min(max(MIN_SIZE[1], h), self.max_size[1])
        if want_w > asked_w or want_h > asked_h:
            self.canvas.configure(width=max(want_w, asked_w), height=max(want_h, asked_h))
        self.canvas.coords(self.item, self.canvas.winfo_width() // 2, self.canvas.winfo_height() // 2)

    def tick(self) -> None:
        """Run as many steps as `speed` allows since the last tick, draw the result, update the
        labels, and schedule the next tick until the machine stops."""
        now = time.perf_counter()
        want = self.speed * (now - self.last) + self.carry
        n, self.carry, self.last = int(want), want % 1, now
        deadline = now + FRAME_BUDGET
        run = self.run
        while n > 0 and not run.halted and run.n_steps < self.max_steps:
            run.run(1)
            n -= 1
            if time.perf_counter() > deadline:
                self.carry = 0.0
                break
        self.draw()
        self.fit()

        # Tape 0 read as a binary number: T_1[1] T_1[2] ... up to the first blank.
        y = run.output()
        value = "-" if not y else int(y, 2) if len(y) <= 64 else f"({len(y)} bits)"
        halted = " H" if run.halted else ""
        self.info.configure(text=f"tape 0: {value}\nstep: {run.n_steps:,}{halted}\nstate: {run.s}")
        restart = "\n[R]" if run.halted else ""
        self.keys.configure(text=f"[ESC]\n[{RETURN_KEY}]\n[SHIFT]+[1-9]{restart}")

        if run.halted or run.n_steps >= self.max_steps:
            self.report()
        else:
            self.pending = self.root.after(16, self.tick)
