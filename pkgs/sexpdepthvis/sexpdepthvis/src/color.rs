use color_gradient::RgbGradient;
use emacs::{FromLisp, Result, Value};

pub(crate) const RESET: &str = "\x1b[0m";
pub(crate) const BLACK: &str = "\x1b[38;2;0;0;0m";
pub(crate) const BG_RED: &str = "\x1b[48;2;237;135;150m";
pub(crate) const BG_GREEN: &str = "\x1b[48;2;166;218;149m";
pub(crate) const BG_BLUE: &str = "\x1b[48;2;125;196;228m";
pub(crate) const BG_YELLOW: &str = "\x1b[48;2;238;212;159m";

#[derive(Default, Clone, Copy, Debug)]
pub struct Color(pub u8, pub u8, pub u8);
pub type Colors = Vec<Color>;

impl<'e> FromLisp<'e> for Color {
    fn from_lisp(value: emacs::Value<'e>) -> Result<Self> {
        let r: u32 = value.car()?;
        let gb: Value<'_> = value.cdr()?;
        let g: u32 = gb.car()?;
        let b: Value<'_> = gb.cdr()?;
        let b: u32 = b.car()?;

        Ok(Self(
            ((r * 255 + 32767) / 65535) as u8,
            ((g * 255 + 32767) / 65535) as u8,
            ((b * 255 + 32767) / 65535) as u8,
        ))
    }
}

#[derive(Debug)]
pub struct ColorSet {
    colors: Colors,
    gradient: RgbGradient,
    cycle: bool,
}

impl ColorSet {
    pub fn new(colors: Colors, cycle: bool) -> Result<Self> {
        let mut gradient = RgbGradient::new(0.0, 1.0);
        for (i, color) in colors.iter().enumerate() {
            let key = i as f32 / (colors.len() - 1) as f32;
            gradient.insert_red(key, color.0 as f32 / 255.0);
            gradient.insert_green(key, color.1 as f32 / 255.0);
            gradient.insert_blue(key, color.2 as f32 / 255.0);
        }

        Ok(Self {
            colors,
            gradient,
            cycle,
        })
    }

    pub fn for_span(&self, depth: usize, max_depth: usize) -> Color {
        let ColorSet {
            colors,
            gradient,
            cycle,
        } = self;
        if *cycle || max_depth < colors.len() {
            colors[depth % colors.len()]
        } else {
            let key = (depth as f32 / max_depth as f32).clamp(0.0, 1.0);
            let color = gradient.get_linear(key);
            Color(
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
            )
        }
    }

    pub fn for_span_ansi(&self, depth: usize, max_depth: usize) -> String {
        rgb_to_ansi(self.for_span(depth, max_depth))
    }

    pub fn for_span_hex(&self, depth: usize, max_depth: usize) -> String {
        rgb_to_hex(self.for_span(depth, max_depth))
    }
}

pub fn interpolate_colors(from: &mut Colors, to: Color, factor: f64) {
    from.iter_mut().for_each(|from| {
        lerp(&mut from.0, to.0, factor);
        lerp(&mut from.1, to.1, factor);
        lerp(&mut from.2, to.2, factor);
    });
}

fn rgb_to_ansi(Color(r, g, b): Color) -> String {
    format!("\x1b[48;2;{};{};{}m", r, g, b)
}

fn rgb_to_hex(Color(r, g, b): Color) -> String {
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

fn lerp(a: &mut u8, b: u8, f: f64) {
    let aa = *a as f64;
    let bb = b as f64;
    *a = (aa + (bb - aa) * (1.0 - f)) as u8;
}
