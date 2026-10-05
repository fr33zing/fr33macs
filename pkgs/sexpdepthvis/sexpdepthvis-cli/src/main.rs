use std::io::{stderr, stdout, Write};

use sexpdepthvis::{
    color::{Color, ColorSet},
    config::{Config, OverlayStyle},
    debug::debug_spans,
    input::Input,
    output,
    parse::parse,
};

const PALETTE: catppuccin::FlavorColors = catppuccin::PALETTE.macchiato.colors;
const FG_COLORS: [catppuccin::Rgb; 7] = [
    PALETTE.red.rgb,
    PALETTE.peach.rgb,
    PALETTE.yellow.rgb,
    PALETTE.green.rgb,
    PALETTE.sapphire.rgb,
    PALETTE.mauve.rgb,
    PALETTE.pink.rgb,
];

fn main() -> anyhow::Result<()> {
    let input = Input::new_from_args()?;
    let mut result = parse(&input)?;

    let colors = FG_COLORS
        .map(|catppuccin::Rgb { r, g, b }: _| Color(r, g, b))
        .to_vec();
    let background_colors = ColorSet::new(colors.clone(), false)?;
    let configuration = Config {
        foreground_colors: ColorSet::new(colors, false)?,
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
