use std::{
    io::{stderr, stdout, Write},
    ops::Range,
};

use sexpdepthvis::{
    color::span_color_hex,
    debug::debug_spans,
    input::Input,
    parse::{parse, ParseResult, Span},
};

fn main() -> anyhow::Result<()> {
    let input = Input::new_from_args()?;
    let mut result = parse(&input)?;

    if cfg!(debug_assertions) {
        debug_spans(&input.contents, &result)?;
        eprint!("\n");
        stderr().flush()?;
    }

    result.apply_offset(input.offset);
    print_output(&result, input.contents.len(), input.offset)?;

    if cfg!(debug_assertions) {
        stdout().flush()?;
        eprint!("\n\n");
    }

    Ok(())
}

fn print_output(result: &ParseResult, len: usize, offset: usize) -> anyhow::Result<()> {
    // LIST OPEN
    print!("(");

    print!(":offset {offset} :len {len}");

    // FACES
    print!(" :faces [");
    let max_depth = result.max_depth();
    for i in 0..=max_depth {
        let fg_color = span_color_hex(i, max_depth, false);
        let bg_color = span_color_hex(i, max_depth, true);
        print!(r#"(:foreground "{fg_color}" :background "{bg_color}" :extend t)"#);
    }
    print!("]");

    // SPANS
    print!(" :spans [");
    for Span { range, depth } in &result.spans {
        let Range { start, end } = range;
        print!("({start} {end} {depth})");
    }
    print!("]");

    // COMMENT RANGES
    print!(" :commented-face (:inherit font-lock-comment-face :slant normal :weight normal)");
    print!(" :commented-ranges [");
    for Range { start, end } in &result.commented_ranges {
        print!("({start} {end})");
    }
    print!("]");

    // LIST CLOSE
    print!(")");

    Ok(())
}
