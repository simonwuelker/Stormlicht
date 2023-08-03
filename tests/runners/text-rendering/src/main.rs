//! A Tool used for [Unicode Rendering Tests](https://github.com/unicode-org/text-rendering-tests)
//!
use cli::CommandLineArgumentParser;
use font::ttf::{Font, TTFParseError};

use std::{fs, io};

#[derive(Debug, Default, CommandLineArgumentParser)]
struct ArgumentParser {
    #[argument(
        may_be_omitted,
        short_name = 'f',
        long_name = "font",
        description = "Specify the font to be used"
    )]
    font_path: Option<String>,

    #[argument(
        may_be_omitted,
        short_name = 'r',
        long_name = "render",
        description = "The characters to be rendered"
    )]
    text: Option<String>,

    #[argument(
        may_be_omitted,
        short_name = 't',
        long_name = "testcase",
        description = "The name of the current test case"
    )]
    testcase: Option<String>,

    #[argument(
        flag,
        short_name = 'v',
        long_name = "version",
        description = "Show package version"
    )]
    version: bool,
}

#[derive(Debug)]
enum Error {
    IO(io::Error),
    Font(TTFParseError),
}

fn main() -> Result<(), Error> {
    let arguments = match ArgumentParser::parse() {
        Ok(arguments) => arguments,
        Err(_) => {
            println!("{}", ArgumentParser::help());
            return Ok(());
        },
    };

    if arguments.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
    } else {
        let font_bytes =
            fs::read(arguments.font_path.expect("No font path provided")).map_err(Error::IO)?;
        let font = Font::new(&font_bytes).map_err(Error::Font)?;
        let svg = font.render_as_svg(
            &arguments.text.expect("No text provided"),
            &arguments.testcase.expect("No testcase name provided"),
        );
        println!("{svg}");
    }
    Ok(())
}
