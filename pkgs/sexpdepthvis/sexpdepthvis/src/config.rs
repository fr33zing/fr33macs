use emacs::{defun, FromLisp, Result, Value, Vector};

use crate::color::{interpolate_colors, Color, ColorSet};

emacs::use_symbols! {
    foreground background
}

#[derive(Debug)]
pub enum OverlayStyle {
    Both,
    Background,
    Foreground,
}

impl OverlayStyle {
    pub fn has_background(&self) -> bool {
        matches!(self, OverlayStyle::Both | OverlayStyle::Background)
    }
}

impl<'e> FromLisp<'e> for OverlayStyle {
    fn from_lisp(value: emacs::Value<'e>) -> emacs::Result<Self> {
        if value == *foreground {
            Ok(OverlayStyle::Foreground)
        } else if value == *background {
            Ok(OverlayStyle::Background)
        } else {
            Ok(OverlayStyle::Both)
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub foreground_colors: ColorSet,
    pub background_colors: ColorSet,
    pub overlay_style: OverlayStyle,
}

#[defun(mod_in_name = false, user_ptr)]
fn configure(
    overlay_colors: Vector,
    buffer_background_color: Color,
    background_opacity: f64,
    cycle_colors: Value<'_>,
    overlay_style: OverlayStyle,
) -> Result<Config> {
    let mut colors = overlay_colors
        .into_iter()
        .map(|val| val.into_rust().unwrap_or_default())
        .collect::<Vec<Color>>();

    let cycle_colors = cycle_colors.is_not_nil();
    let foreground_colors = ColorSet::new(colors.clone(), cycle_colors)?;
    interpolate_colors(&mut colors, buffer_background_color, background_opacity);
    let background_colors = ColorSet::new(colors, cycle_colors)?;

    let config = Config {
        foreground_colors,
        background_colors,
        overlay_style,
    };

    Ok(config)
}
