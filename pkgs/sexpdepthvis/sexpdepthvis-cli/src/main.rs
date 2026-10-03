use std::io::{stderr, stdout, Write};

use sexpdepthvis::{debug::debug_spans, input::Input, output, parse::parse};

fn main() -> anyhow::Result<()> {
    let input = Input::new_from_args()?;
    let mut result = parse(&input)?;

    if cfg!(debug_assertions) {
        debug_spans(&input.contents, &result)?;
        eprint!("\n");
        stderr().flush()?;
    }

    result.apply_offset(input.offset);
    let output = output(&result, input.offset)?;
    println!("{output}");

    if cfg!(debug_assertions) {
        stdout().flush()?;
        eprint!("\n\n");
    }

    Ok(())
}
