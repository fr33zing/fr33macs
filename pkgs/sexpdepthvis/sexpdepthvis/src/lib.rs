pub mod color;
pub mod debug;
pub mod input;
pub mod parse;

use std::{fmt::Write, ops::Range};

use emacs::{defun, Env, Result, Value};

use crate::{
    color::span_color_hex,
    input::Input,
    parse::{parse, ParseResult},
};

emacs::plugin_is_GPL_compatible!();

#[emacs::module]
fn init(env: &Env) -> Result<Value<'_>> {
    env.message("Loaded sexpdepthvis.")
}

#[defun]
fn generate(
    _env: &Env,
    point: usize,
    offset: usize,
    major_mode: String,
    contents: String,
) -> Result<String> {
    let input = Input::new(point, offset, &major_mode, contents)?;
    let mut result = parse(&input)?;
    result.apply_offset(input.offset);
    let output = output(&result, input.offset)?;

    Ok(output)
}

pub fn output(result: &ParseResult, offset: usize) -> Result<String> {
    let mut s = String::new();

    // LIST OPEN
    write!(&mut s, "(")?;

    // OFFSET
    write!(&mut s, ":offset {offset}")?;

    // FACES
    write!(&mut s, " :faces [")?;
    let max_depth = result.max_depth();
    for i in 0..=max_depth {
        let fg_color = span_color_hex(i, max_depth, false);
        let bg_color = span_color_hex(i, max_depth, true);
        write!(
            &mut s,
            r#"(:foreground "{fg_color}" :background "{bg_color}" :extend t)"#
        )?;
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
