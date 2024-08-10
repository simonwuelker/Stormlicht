//! A Tool used for [Unicode Rendering Tests](https://github.com/unicode-org/text-rendering-tests)

use std::{fs, io};

use clap::Parser;
use font::ttf::{Font, TTFParseError};

#[derive(Debug, Default, Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    /// Specify the font to be used
    #[arg(short = 'f', long = "font")]
    font_path: Option<String>,

    /// The characters to be rendered
    #[arg(short = 'r', long = "render")]
    text: Option<String>,

    /// The name of the current test case
    #[arg(short = 't', long = "testcase")]
    testcase: Option<String>,
}

#[derive(Debug)]
// Dead code analysis ignores debug impls (which are called when the error is returned from main)
#[allow(dead_code)]
enum Error {
    IO(io::Error),
    Font(TTFParseError),
}

fn main() -> Result<(), Error> {
    let args = Arguments::parse();

    let font_bytes = fs::read(args.font_path.expect("No font path provided")).map_err(Error::IO)?;
    let font = Font::new(&font_bytes).map_err(Error::Font)?;
    let svg = font.render_as_svg(
        &args.text.expect("No text provided"),
        &args.testcase.expect("No testcase name provided"),
    );
    println!("{svg}");

    Ok(())
}
