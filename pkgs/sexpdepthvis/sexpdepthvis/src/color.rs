use catppuccin::FlavorColors;
use color_gradient::RgbGradient;
use emacs::Result;

pub(crate) const RESET: &str = "\x1b[0m";
pub(crate) const BLACK: &str = "\x1b[38;2;0;0;0m";
pub(crate) const BG_RED: &str = "\x1b[48;2;237;135;150m";
pub(crate) const BG_GREEN: &str = "\x1b[48;2;166;218;149m";
pub(crate) const BG_BLUE: &str = "\x1b[48;2;125;196;228m";
pub(crate) const BG_YELLOW: &str = "\x1b[48;2;238;212;159m";

pub const PALETTE: FlavorColors = catppuccin::PALETTE.macchiato.colors;
pub const FG_COLORS: [catppuccin::Rgb; 7] = [
    PALETTE.red.rgb,
    PALETTE.peach.rgb,
    PALETTE.yellow.rgb,
    PALETTE.green.rgb,
    PALETTE.sapphire.rgb,
    PALETTE.mauve.rgb,
    PALETTE.pink.rgb,
];

type Colors = Vec<catppuccin::Rgb>;

#[derive(Debug)]
pub struct ColorConfig {
    pub colors: Colors,
    pub gradient: RgbGradient,
}

impl ColorConfig {
    fn lerp(a: &mut u8, b: u8, f: f64) {
        let af = *a as f64;
        let bf = b as f64;
        *a = (af + (bf - af) * f) as u8;
    }

    pub fn new(mut colors: Colors, interpolate: Option<(String, f64)>) -> Result<Self> {
        if let Some((bg, opacity)) = interpolate {
            let bg = hex_color::HexColor::parse(&bg)?;
            let opacity = 1.0 - opacity;
            colors.iter_mut().for_each(|c| {
                Self::lerp(&mut c.r, bg.r, opacity);
                Self::lerp(&mut c.g, bg.g, opacity);
                Self::lerp(&mut c.b, bg.b, opacity);
            });
        }

        let mut gradient = RgbGradient::new(0.0, 1.0);
        for (i, color) in colors.iter().enumerate() {
            let key = i as f32 / (colors.len() - 1) as f32;
            gradient.insert_red(key, color.r as f32 / 255.0);
            gradient.insert_green(key, color.g as f32 / 255.0);
            gradient.insert_blue(key, color.b as f32 / 255.0);
        }

        Ok(Self { colors, gradient })
    }
}

pub(crate) fn span_color(
    depth: usize,
    max_depth: usize,
    ColorConfig { colors, gradient }: &ColorConfig,
) -> (u8, u8, u8) {
    if max_depth < colors.len() {
        let catppuccin::Rgb { r, g, b, .. } = colors[depth];
        (r, g, b)
    } else {
        let key = (depth as f32 / max_depth as f32).clamp(0.0, 1.0);
        let color = gradient.get_linear(key);
        (
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
        )
    }
}

pub(crate) fn span_color_term(depth: usize, max_depth: usize, colors: &ColorConfig) -> String {
    let (r, g, b) = span_color(depth, max_depth, colors);
    format!("\x1b[48;2;{};{};{}m", r, g, b)
}

pub fn span_color_hex(depth: usize, max_depth: usize, colors: &ColorConfig) -> String {
    let (r, g, b) = span_color(depth, max_depth, colors);
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}
