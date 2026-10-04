pub mod color;
pub mod config;
pub mod debug;
pub mod input;
pub mod parse;

use std::{fmt::Write, ops::Range};

use emacs::{defun, Env, Result};

use crate::{
    color::span_color_hex,
    config::Config,
    config::OverlayStyle,
    input::Input,
    parse::{parse, ParseResult},
};

emacs::plugin_is_GPL_compatible!();

#[emacs::module(separator = "--")]
fn init(_: &Env) -> Result<()> {
    Ok(())
}

#[defun]
fn generate(
    configuration: &Config,
    point: usize,
    offset: usize,
    major_mode: String,
    contents: String,
) -> Result<String> {
    //env.call("sexpdepthvis--init", &[])?;

    let input = Input::new(point, offset, &major_mode, contents)?;
    let mut result = parse(&input)?;
    result.apply_offset(input.offset);
    let output = output(&configuration, &input, &result)?;

    Ok(output)
}

pub fn output(configuration: &Config, input: &Input, result: &ParseResult) -> Result<String> {
    let mut s = String::new();

    // LIST OPEN
    write!(&mut s, "(")?;

    // OFFSET
    write!(&mut s, ":offset {}", input.offset)?;

    // FACES
    write!(&mut s, " :faces [")?;
    let max_depth = result.max_depth();
    for i in 0..=max_depth {
        let fg = span_color_hex(i, max_depth, &configuration.foreground_colors);
        let bg = span_color_hex(i, max_depth, &configuration.background_colors);
        match configuration.overlay_style {
            OverlayStyle::Both => {
                write!(&mut s, r#"(:foreground "{fg}" :background "{bg}" "#)?;
            }
            OverlayStyle::Foreground => {
                write!(&mut s, r#"(:foreground "{fg}" "#)?;
            }
            OverlayStyle::Background => {
                write!(&mut s, r#"(:background "{bg}" "#)?;
            }
        };
        write!(&mut s, ":extend t)")?;
    }
    write!(&mut s, "]")?;

    // SPANS
    write!(&mut s, " :spans [")?;
    for Range { start, end } in &result.spans {
        write!(&mut s, "({start} . {end})")?;
    }
    write!(&mut s, "]")?;

    // COMMENT RANGES
    write!(&mut s, " :comments [")?;
    for Range { start, end } in &result.comments {
        write!(&mut s, "({start} . {end})")?;
    }
    write!(&mut s, "]")?;

    // LIST CLOSE
    write!(&mut s, ")")?;

    Ok(s)
}
