use std::ops::Range;

use crate::{color::*, parse::ParseResult};

pub fn debug_spans(contents: &str, result: &ParseResult) -> anyhow::Result<()> {
    eprint!("\n----- SPAN DEBUG START ------\n");

    let max_depth = result.max_depth();

    for (i, c) in contents.chars().enumerate() {
        let mut enclosing_span = None::<&Range<usize>>;
        for span in &result.spans {
            if span.contains(&i) {
                enclosing_span = Some(span);
            }
        }

        let color = if c == '\n' {
            RESET
        } else if enclosing_span.is_some() {
            &span_color_term(i, max_depth, false)
        } else {
            RESET
        };

        eprint!("{BLACK}{color}{c}{RESET}");
    }

    eprint!("------ SPAN DEBUG END -------\n");

    Ok(())
}
