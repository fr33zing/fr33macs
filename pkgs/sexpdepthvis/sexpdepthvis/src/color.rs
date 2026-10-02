use color_gradient::RgbGradient;
use lazy_static::lazy_static;

pub(crate) const RESET: &str = "\x1b[0m";
pub(crate) const BLACK: &str = "\x1b[38;2;0;0;0m";
pub(crate) const BG_RED: &str = "\x1b[48;2;237;135;150m";
pub(crate) const BG_GREEN: &str = "\x1b[48;2;166;218;149m";
pub(crate) const BG_BLUE: &str = "\x1b[48;2;125;196;228m";
pub(crate) const BG_YELLOW: &str = "\x1b[48;2;238;212;159m";

lazy_static! {
    static ref FG_COLORS: Vec<catppuccin::Rgb> = vec![
        catppuccin::PALETTE.macchiato.colors.red.rgb,
        catppuccin::PALETTE.macchiato.colors.peach.rgb,
        catppuccin::PALETTE.macchiato.colors.yellow.rgb,
        catppuccin::PALETTE.macchiato.colors.green.rgb,
        catppuccin::PALETTE.macchiato.colors.sapphire.rgb,
        catppuccin::PALETTE.macchiato.colors.mauve.rgb,
        catppuccin::PALETTE.macchiato.colors.pink.rgb,
    ];
    static ref BG_COLORS: Vec<catppuccin::Rgb> = {
        let lerp_fac = 0.95;
        let mut colors = FG_COLORS.clone();

        fn lerp(a: &mut u8, b: u8, f: f32) {
            let af = *a as f32;
            let bf = b as f32;
            *a = (af + (bf - af) * f) as u8;
        }

        let bg: catppuccin::Rgb = catppuccin::PALETTE.macchiato.colors.base.rgb;
        colors.iter_mut().for_each(|c| {
            lerp(&mut c.r, bg.r, lerp_fac);
            lerp(&mut c.g, bg.g, lerp_fac);
            lerp(&mut c.b, bg.b, lerp_fac);
        });

        colors
    };
    static ref TERM_BG_COLORS: Vec<String> = FG_COLORS
        .iter()
        .map(|catppuccin::Rgb { r, g, b, .. }| format!("\x1b[48;2;{r};{g};{b}m",))
        .collect();
    static ref FG_GRADIENT: RgbGradient = {
        let mut gradient = RgbGradient::new(0.0, 1.0);
        for (i, color) in FG_COLORS.iter().enumerate() {
            let key = i as f32 / (FG_COLORS.len() - 1) as f32;
            gradient.insert_red(key, color.r as f32 / 255.0);
            gradient.insert_green(key, color.g as f32 / 255.0);
            gradient.insert_blue(key, color.b as f32 / 255.0);
        }
        gradient
    };
    static ref BG_GRADIENT: RgbGradient = {
        let mut gradient = RgbGradient::new(0.0, 1.0);
        for (i, color) in BG_COLORS.iter().enumerate() {
            let key = i as f32 / (BG_COLORS.len() - 1) as f32;
            gradient.insert_red(key, color.r as f32 / 255.0);
            gradient.insert_green(key, color.g as f32 / 255.0);
            gradient.insert_blue(key, color.b as f32 / 255.0);
        }
        gradient
    };
}

pub(crate) fn span_color(depth: usize, max_depth: usize, background: bool) -> (u8, u8, u8) {
    let table: &Vec<catppuccin::Rgb> = if background { &BG_COLORS } else { &FG_COLORS };
    let gradient: &RgbGradient = if background {
        &BG_GRADIENT
    } else {
        &FG_GRADIENT
    };

    if max_depth < table.len() {
        let catppuccin::Rgb { r, g, b, .. } = table[depth];
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

pub(crate) fn span_color_term(depth: usize, max_depth: usize, background: bool) -> String {
    let (r, g, b) = span_color(depth, max_depth, background);
    format!("\x1b[48;2;{};{};{}m", r, g, b)
}

pub fn span_color_hex(depth: usize, max_depth: usize, background: bool) -> String {
    let (r, g, b) = span_color(depth, max_depth, background);
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}
