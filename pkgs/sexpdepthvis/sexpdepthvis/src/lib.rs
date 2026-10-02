pub mod color;
pub mod debug;
pub mod input;
pub mod parse;

use std::{fmt::Write, ops::Range};

use color::span_color_hex;
use emacs::{defun, Env, Result, Value};
use parse::{ParseResult, Span};

use crate::{input::Input, parse::parse};

emacs::plugin_is_GPL_compatible!(); // :)

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
    let output = print_output(&result, input.contents.len(), input.offset)?;

    Ok(output)
}

fn print_output(result: &ParseResult, len: usize, offset: usize) -> Result<String> {
    let mut s = String::new();

    // LIST OPEN
    write!(&mut s, "(")?;

    write!(&mut s, ":offset {offset} :len {len}")?;

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
    for Span { range, depth } in &result.spans {
        let Range { start, end } = range;
        write!(&mut s, "({start} {end} {depth})")?;
    }
    write!(&mut s, "]")?;

    // COMMENT RANGES
    write!(
        &mut s,
        " :commented-face (:inherit font-lock-comment-face :slant normal :weight normal)"
    )?;
    write!(&mut s, " :commented-ranges [")?;
    for Range { start, end } in &result.commented_ranges {
        write!(&mut s, "({start} {end})")?;
    }
    write!(&mut s, "]")?;

    // LIST CLOSE
    write!(&mut s, ")")?;

    Ok(s)
}
