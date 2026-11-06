//! A window that runs a (multi-tape) Turing machine and draws its current step: one row of pixels
//! per tape, tape 0 on top, one pixel per tape cell, with each head drawn in a shade of red for
//! the current state.
//!
//! Each row is always `cells` cells wide, padded with ∅ past the end of the tape, and the window
//! grows to fit the rows up to `MAX_SIZE`.

use macroquad::prelude::*;

use simple_tm::tm::{Configuration, Symbol, TuringMachine};

/// An `0xRRGGBB` color.
const fn hex(rgb: u32) -> Color {
    Color::from_rgba((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255)
}

// Colors from Open Color by heeyeun (MIT License), https://github.com/yeun/open-color.
// See ATTRIBUTIONS.md at the top of the repository.
pub const COLOR_1: Color = hex(0xffffff); // white
pub const COLOR_0: Color = hex(0x000000); // black
pub const COLOR_EMPTY: Color = hex(0x343a40); // gray 8
pub const COLOR_START: Color = hex(0x339af0); // blue 5
/// Red 4 to red 9.
pub const COLOR_HEADS: [Color; 6] =
    [hex(0xff8787), hex(0xff6b6b), hex(0xfa5252), hex(0xf03e3e), hex(0xe03131), hex(0xc92a2a)];
const COLOR_TEXT: Color = hex(0xadb5bd); // gray 5
const BACKGROUND: Color = hex(0x212529); // gray 9, around the tapes

const GAP: usize = 3; // pixels between tapes
const MIN_SIZE: (f32, f32) = (360.0, 240.0);
/// The largest the window grows (macroquad cannot ask for the size of the screen).
const MAX_SIZE: (f32, f32) = (1400.0, 800.0);
const FRAME_BUDGET: f64 = 0.012; // seconds of stepping per tick

// The labels copy Tk's TkFixedFont on macOS, Menlo at 8 points: Tk rounds 8 points * 1.3335
// pixels per point to an 11-pixel font and puts each character on a grid of 7 by 13 pixels.
// Menlo is Apple's, so it is read from the system rather than shipped with this program.
const FONT_PATH: &str = "/System/Library/Fonts/Menlo.ttc";
const FONT_SIZE: u16 = 11;
const ADVANCE: f32 = 7.0;
const ASCENT: f32 = 10.0;
const LINESPACE: f32 = 13.0;
/// How far a label's text sits inside it: Tk's borderwidth 2 plus padx and pady 1.
const LABEL_INSET: f32 = 3.0;
const RETURN_KEY: &str = if cfg!(target_os = "macos") { "RETURN" } else { "ENTER" };
const DIGITS: [KeyCode; 9] = [
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Key5,
    KeyCode::Key6,
    KeyCode::Key7,
    KeyCode::Key8,
    KeyCode::Key9,
];

fn symbol_color(sigma: Symbol) -> Color {
    match sigma {
        Symbol::One => COLOR_1,
        Symbol::Zero => COLOR_0,
        Symbol::Blank => COLOR_EMPTY,
        Symbol::Start => COLOR_START,
    }
}

/// Colors for one tape, with the head at i drawn in the shade of red for state s.
pub fn row_colors(tape: &[Symbol], i: usize, s: usize) -> Vec<Color> {
    let mut colors: Vec<Color> = tape.iter().map(|&sigma| symbol_color(sigma)).collect();
    colors[i] = COLOR_HEADS[s % COLOR_HEADS.len()];
    colors
}

/// n if Shift + n (n = 1..9) is held down, else None.
fn shift_digit() -> Option<u32> {
    if !is_key_down(KeyCode::LeftShift) && !is_key_down(KeyCode::RightShift) {
        return None;
    }
    (1..).zip(DIGITS).find_map(|(n, key)| is_key_down(key).then_some(n))
}

/// n with commas between groups of three digits.
pub fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (j, c) in digits.chars().enumerate() {
        if j > 0 && (digits.len() - j).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

pub struct Settings {
    pub scale: usize,
    pub speed: f64,
    pub max_steps: u64,
    pub max_cells: usize,
}

/// Runs `run` for at most `max_steps` steps while drawing it, showing at most `max_cells` cells of
/// each tape (fewer if they would not fit in `MAX_SIZE`); calls `on_stop(run)` once per run when
/// the machine halts or reaches `max_steps`.
///
/// Keys: Escape closes the window; Return finishes the run without drawing, reports the result,
/// and closes; holding Shift + n (n = 1..9) runs at n times the starting speed; R (once halted)
/// starts the machine again on the same input.
struct Window<'a, F: Fn(&Configuration)> {
    run: Configuration<'a>,
    /// `run` at its start, for restarting.
    initial: Configuration<'a>,
    reported: bool,
    scale: usize,
    speed: f64,
    base_speed: f64,
    max_steps: u64,
    cells: usize,
    on_stop: F,
    font: Option<Font>,
    image: Image,
    texture: Texture2D,
    asked: (f32, f32),
    offsets: Vec<usize>,
    carry: f64,
    last: f64,
}

impl<'a, F: Fn(&Configuration)> Window<'a, F> {
    fn new(run: Configuration<'a>, settings: &Settings, font: Option<Font>, on_stop: F) -> Self {
        let mut window = Self {
            reported: false,
            scale: settings.scale,
            speed: settings.speed,
            base_speed: settings.speed,
            max_steps: settings.max_steps,
            cells: settings.max_cells.min(MAX_SIZE.0 as usize / settings.scale),
            on_stop,
            font,
            image: Image::empty(),
            texture: Texture2D::empty(),
            asked: MIN_SIZE,
            offsets: Vec::new(),
            carry: 0.0,
            last: 0.0,
            initial: run.clone(),
            run,
        };
        window.start();
        window
    }

    /// Begin drawing `run` from a clean image.
    fn start(&mut self) {
        self.reported = false;
        let (w, h) = self.size();
        self.image = Image::gen_image_color(w as u16, h as u16, BLANK);
        self.texture = Texture2D::from_image(&self.image);
        self.texture.set_filter(FilterMode::Nearest);
        self.offsets = vec![0; self.run.tapes().len()];
        self.carry = 0.0;
        self.last = get_time();
    }

    /// Run the machine until it halts or reaches `max_steps` (without drawing) and report the
    /// result.
    fn finish(&mut self) {
        self.run.run(self.max_steps - self.run.steps());
        self.report();
    }

    /// Call `on_stop` for the current run, once.
    fn report(&mut self) {
        if !self.reported {
            (self.on_stop)(&self.run);
            self.reported = true;
        }
    }

    /// Once the machine has halted, start it again on the same input.
    fn restart(&mut self) {
        if self.run.is_halted() {
            self.run = self.initial.clone();
            self.start();
        }
    }

    /// Width and height of the drawing, in pixels.
    fn size(&self) -> (usize, usize) {
        let (k, m) = (self.scale, self.run.tapes().len());
        (self.cells * k, m * k + GAP * (m - 1))
    }

    /// Draw the view of each tape as `scale` rows of pixels, tape 0 on top, padded with ∅ to
    /// `cells` cells.
    fn draw(&mut self) {
        let (k, run) = (self.scale, &self.run);
        let mut rows = Vec::with_capacity(run.tapes().len());
        for (j, (tape, &i)) in run.tapes().iter().zip(run.heads()).enumerate() {
            let o = self.offsets[j].max((i + 1).saturating_sub(self.cells)).min(i);
            self.offsets[j] = o;
            let mut view = tape[o..tape.len().min(o + self.cells)].to_vec();
            view.resize(self.cells, Symbol::Blank);
            rows.push(row_colors(&view, i - o, run.state()));
        }
        let w = usize::from(self.image.width);
        for (j, colors) in rows.iter().enumerate() {
            for dy in 0..k {
                let y = j * (k + GAP) + dy;
                for (x, color) in colors.iter().enumerate() {
                    let rgba: [u8; 4] = (*color).into();
                    for dx in 0..k {
                        let p = 4 * (y * w + x * k + dx);
                        self.image.bytes[p..p + 4].copy_from_slice(&rgba);
                    }
                }
            }
        }
        self.texture.update(&self.image);
    }

    /// Grow the window to fit the drawing (up to `MAX_SIZE`) and draw it in the center.
    fn fit(&mut self) {
        let (w, h) = self.size();
        let want_w = (w as f32).clamp(MIN_SIZE.0, MAX_SIZE.0);
        let want_h = (h as f32).clamp(MIN_SIZE.1, MAX_SIZE.1);
        if want_w > self.asked.0 || want_h > self.asked.1 {
            self.asked = (want_w.max(self.asked.0), want_h.max(self.asked.1));
            request_new_screen_size(self.asked.0, self.asked.1);
        }
        // Like a Tk image anchored at the center of the canvas, in whole pixels.
        let x = screen_width() as i32 / 2 - i32::from(self.image.width) / 2;
        let y = screen_height() as i32 / 2 - i32::from(self.image.height) / 2;
        draw_texture(&self.texture, x as f32, y as f32, WHITE);
    }

    /// Draw `lines` as a Tk label with its top left corner at (x, y), or its top right corner if
    /// `right` (and each line justified to the right).
    fn label(&self, lines: &str, x: f32, y: f32, right: bool) {
        let width = |line: &str| ADVANCE * line.chars().count() as f32;
        let w = lines.lines().map(width).fold(0.0, f32::max) + 2.0 * LABEL_INSET;
        let h = LINESPACE * lines.lines().count() as f32 + 2.0 * LABEL_INSET;
        draw_rectangle(if right { x - w } else { x }, y, w, h, BACKGROUND);
        let params = TextParams {
            font: self.font.as_ref(),
            font_size: FONT_SIZE,
            color: COLOR_TEXT,
            ..TextParams::default()
        };
        for (n, line) in lines.lines().enumerate() {
            let left = if right { x - LABEL_INSET - width(line) } else { x + LABEL_INSET };
            let baseline = y + LABEL_INSET + ASCENT + LINESPACE * n as f32;
            for (c, sigma) in line.chars().enumerate() {
                draw_text_ex(
                    sigma.encode_utf8(&mut [0; 4]),
                    left + ADVANCE * c as f32,
                    baseline,
                    params.clone(),
                );
            }
        }
    }

    /// Handle the keys, run as many steps as `speed` allows since the last tick, and draw the
    /// result and the labels. Return false once the window should close.
    fn tick(&mut self) -> bool {
        if is_quit_requested() || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) {
            return false;
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
            self.finish();
            return false;
        }
        if is_key_pressed(KeyCode::R) {
            self.restart();
        }
        self.speed = self.base_speed * f64::from(shift_digit().unwrap_or(1));

        let now = get_time();
        let want = self.speed * (now - self.last) + self.carry;
        let mut n = want as u64;
        (self.carry, self.last) = (want % 1.0, now);
        let deadline = now + FRAME_BUDGET;
        while n > 0 && !self.run.is_halted() && self.run.steps() < self.max_steps {
            self.run.run(1);
            n -= 1;
            if get_time() > deadline {
                self.carry = 0.0;
                break;
            }
        }
        clear_background(BACKGROUND);
        self.draw();
        self.fit();

        // Tape 0 read as a binary number: T_1[1] T_1[2] ... up to the first blank.
        let run = &self.run;
        let y = run.output();
        let value = match y.len() {
            0 => "-".to_owned(),
            1..=64 => u64::from_str_radix(&y, 2).map_or(y.clone(), |v| v.to_string()),
            len => format!("({len} bits)"),
        };
        let halted = if run.is_halted() { " H" } else { "" };
        let info = format!(
            "tape 0: {value}\nstep: {}{halted}\nstate: {}",
            thousands(run.steps()),
            run.state()
        );
        self.label(&info, 6.0, 4.0, false);
        let restart = if run.is_halted() { "\n[R]" } else { "" };
        self.label(
            &format!("[ESC]\n[{RETURN_KEY}]\n[SHIFT]+[1-9]{restart}"),
            screen_width() - 6.0,
            4.0,
            true,
        );

        if run.is_halted() || run.steps() >= self.max_steps {
            self.report();
        }
        true
    }
}

/// Open the window on M running on x and keep it open until it is closed, calling
/// `on_stop(run)` once per run. Exit the process with status 0 if the last run halted, else 1.
pub fn show(
    machine: TuringMachine,
    x: String,
    settings: Settings,
    on_stop: impl Fn(&Configuration) + 'static,
) -> ! {
    let conf = Conf {
        window_title: "Turing Machine".to_owned(),
        window_width: MIN_SIZE.0 as i32,
        window_height: MIN_SIZE.1 as i32,
        window_resizable: true,
        high_dpi: true,
        ..Conf::default()
    };
    macroquad::Window::from_config(conf, async move {
        prevent_quit();
        // Fall back to macroquad's own font where Menlo is missing (outside macOS).
        let font =
            std::fs::read(FONT_PATH).ok().and_then(|bytes| load_ttf_font_from_bytes(&bytes).ok());
        let run = Configuration::new(&machine, &x).expect("encode gives a binary string");
        let mut window = Window::new(run, &settings, font, on_stop);
        while window.tick() {
            next_frame().await;
        }
        std::process::exit(i32::from(!window.run.is_halted()));
    });
    // The window closed without going through `tick`.
    std::process::exit(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_colors_mark_the_head() {
        let tape = [Symbol::Start, Symbol::One, Symbol::Zero, Symbol::Blank];
        assert_eq!(row_colors(&tape, 1, 0), [COLOR_START, COLOR_HEADS[0], COLOR_0, COLOR_EMPTY]);
        // Each state gets its own shade of red, cycling after COLOR_HEADS.len() states.
        let shades: Vec<Color> =
            (0..=COLOR_HEADS.len()).map(|s| row_colors(&[Symbol::Start], 0, s)[0]).collect();
        assert_eq!(shades[..COLOR_HEADS.len()], COLOR_HEADS);
        assert_eq!(shades[COLOR_HEADS.len()], shades[0]);
    }

    #[test]
    fn thousands_groups_digits() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_234_567), "1,234,567");
    }
}
