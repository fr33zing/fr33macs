use emacs::{defun, FromLisp, Result, Value};

use crate::color::{ColorConfig, FG_COLORS};

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
    pub foreground_colors: ColorConfig,
    pub background_colors: ColorConfig,
    pub overlay_style: OverlayStyle,
}

#[defun(mod_in_name = false, user_ptr)]
fn configure(
    overlay_style: Value<'_>,
    buffer_background: String,
    background_opacity: Value<'_>,
) -> Result<Config> {
    let background_opacity: f64 = background_opacity.into_rust()?;
    let foreground_colors = ColorConfig::new(FG_COLORS.to_vec(), None)?;
    let background_colors = ColorConfig::new(
        FG_COLORS.to_vec(),
        Some((buffer_background, background_opacity)),
    )?;
    let overlay_style: OverlayStyle = overlay_style.into_rust()?;

    let config = Config {
        foreground_colors,
        background_colors,
        overlay_style,
    };

    Ok(config)
}
