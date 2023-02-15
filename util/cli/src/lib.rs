pub use cli_derive::CommandLineArgumentParser;

#[derive(Clone, Debug)]
pub enum CommandLineParseError {
    InvalidArguments,
    MissingRequiredArgument(&'static str),
    NotAFlag(&'static str),
}

pub trait CommandLineArgumentParser: Sized {
    /// Parses command line arguments more or less as defined in <https://www.gnu.org/software/libc/manual/html_node/Argument-Syntax.html>
    ///
    /// Arguments can be passed either as an option or by position.
    /// Options are specified in the link above and can be placed in any order.
    /// Positional arguments can be interspersed with options but their ordering matters.
    /// For example, the arguments
    /// ```text, ignore
    /// abc -d --ef=g hi -jk
    /// ``
    /// would parse as
    /// * 2 positional arguments, `abc` and `hi`
    /// * 3 short options `d`, `j``and `k` (assuming none of these need values)
    /// * 1 long option `ef` with value `1`
    fn parse() -> Result<Self, CommandLineParseError>;
    fn help() -> &'static str;
}
