use std::io::{stderr, stdout, Write};

use sexpdepthvis::{
    color::{ColorConfig, FG_COLORS, PALETTE},
    config::{Config, OverlayStyle},
    debug::debug_spans,
    input::Input,
    output,
    parse::parse,
};

fn main() -> anyhow::Result<()> {
    let input = Input::new_from_args()?;
    let mut result = parse(&input)?;
    let background_colors = ColorConfig::new(
        FG_COLORS.to_vec(),
        Some((PALETTE.base.hex.to_string(), 0.12)),
    )?;
    let configuration = Config {
        foreground_colors: ColorConfig::new(FG_COLORS.to_vec(), None)?,
        background_colors,
        overlay_style: OverlayStyle::Both,
    };

    if cfg!(debug_assertions) {
        debug_spans(&configuration, &input.contents, &result)?;
        eprint!("\n");
        stderr().flush()?;
    }

    result.apply_offset(input.offset);
    let output = output(&configuration, &input, &result)?;
    println!("{output}");

    if cfg!(debug_assertions) {
        stdout().flush()?;
        eprint!("\n\n");
    }

    Ok(())
}
