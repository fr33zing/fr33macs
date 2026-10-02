use crate::{
    color::*,
    parse::{ParseResult, Span},
};

pub fn debug_spans(contents: &str, result: &ParseResult) -> anyhow::Result<()> {
    eprint!("\n----- SPAN DEBUG START ------\n");

    let max_depth = result.max_depth();

    for (i, c) in contents.chars().enumerate() {
        let mut enclosing_span = None::<&Span>;
        for span in &result.spans {
            if span.range.contains(&i) {
                enclosing_span = Some(span);
            }
        }

        let color = if c == '\n' {
            RESET
        } else if let Some(span) = enclosing_span {
            &span_color_term(span.depth, max_depth, false)
        } else {
            RESET
        };

        eprint!("{BLACK}{color}{c}{RESET}");
    }

    eprint!("------ SPAN DEBUG END -------\n");

    Ok(())
}
