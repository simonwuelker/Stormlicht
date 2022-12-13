use std::io::{Cursor, Read, Seek};

pub type SourceText<'a> = &'a [u8];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub index: u64,
}

impl Position {
    pub fn new(index: u64) -> Self {
        Self { index: index }
    }
}

#[derive(Debug, PartialEq)]
pub struct Span {
    start: Position,
    end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self {
            start: start,
            end: end,
        }
    }
}

pub trait Parser {
    /// Try to apply the parser to the source text. Returns the span
    /// of matched text on success and a position with a syntax error on failure
    /// After a failed parse, the cursor position is undefined, the caller
    /// is responsible for resetting it
    fn parse(&self, source: &mut Cursor<SourceText>) -> Result<Span, Position>;
}

pub struct Literal {
    value: &'static [u8],
}

impl Literal {
    pub fn new(value: &'static [u8]) -> Self {
        Self { value: value }
    }
}

impl Parser for Literal {
    fn parse(&self, source: &mut Cursor<SourceText>) -> Result<Span, Position> {
        let start = Position::new(source.stream_position().unwrap());
        let mut buffer = vec![0; self.value.len()];
        source
            .read_exact(&mut buffer)
            .map_err(|_| Position::new(source.stream_position().unwrap()))?;

        let end = Position::new(source.stream_position().unwrap());
        if buffer == self.value {
            Ok(Span::new(start, end))
        } else {
            Err(start)
        }
    }
}

/// Wraps another parser and applies it as often as possible (including not at all)
/// This implies that the [Parser::parse] operation can never fail for [Many].
pub struct Many {
    parser: Box<dyn Parser>,
}

impl Many {
    pub fn new<P: Parser + 'static>(parser: P) -> Self {
        Self {
            parser: Box::new(parser) as Box<dyn Parser>,
        }
    }
}

impl Parser for Many {
    fn parse(&self, source: &mut Cursor<SourceText>) -> Result<Span, Position> {
        let start = Position::new(source.stream_position().unwrap());
        let mut up_to = start;
        while let Ok(parsed_span) = self.parser.parse(source) {
            up_to = parsed_span.end;
        }
        Ok(Span::new(start, up_to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Seek};

    #[test]
    fn test_literal_parser() {
        let mut source_text = Cursor::new(&b"foo"[..]);
        let succeeding_parser = Literal::new(&b"fo"[..]);
        let failing_parser = Literal::new(&b"bar"[..]);
        assert_eq!(
            succeeding_parser.parse(&mut source_text),
            Ok(Span::new(Position::new(0), Position::new(2)))
        );
        source_text.rewind().unwrap();
        assert_eq!(
            failing_parser.parse(&mut source_text),
            Err(Position::new(0))
        );
    }

    #[test]
    fn test_many_parser() {
        let mut source_text = Cursor::new(&b"foofoofoobar"[..]);
        let parser = Many::new(Literal::new(&b"foo"[..]));
        assert_eq!(
            parser.parse(&mut source_text),
            Ok(Span::new(Position::new(0), Position::new(9)))
        );
    }
}
