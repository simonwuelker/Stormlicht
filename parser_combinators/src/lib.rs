pub type ParseResult<In, Out> = Result<(In, Out), usize>;

pub trait Parser {
    type In: ?Sized;
    type Out;

    fn parse<'a>(&self, data: &'a Self::In) -> ParseResult<&'a Self::In, Self::Out>;
}

pub trait ParserCombinator: Parser + Sized {
    fn then<P: Parser<In=Self::In>>(self, other: P) -> ChainedParser<Self, P> {
        ChainedParser{
            first: self,
            second: other,
        }
    }
}
impl<T: Parser + Sized> ParserCombinator for T {}
pub struct ChainedParser<A, B> {
    first: A,
    second: B,
}

impl<T: ?Sized, A: Parser<In=T>, B: Parser<In=T>> Parser for ChainedParser<A, B> {
    type In = T;
    type Out = (A::Out, B::Out);

    fn parse<'a>(&self, data: &'a Self::In) -> ParseResult<&'a Self::In, Self::Out> {
        match self.first.parse(&data) {
            Ok((remaining_input, out_first)) => {
                match self.second.parse(remaining_input) {
                    Ok((remaining_input, out_second)) => {
                        Ok((remaining_input, (out_first, out_second)))
                    }
                    Err(parsed_until) => Err(parsed_until),
                }
            }
            Err(parsed_until) => Err(parsed_until)
        }
    }
}

pub struct Literal<T: 'static> {
    want: &'static [T],
}

impl<T: Eq> Parser for Literal<T> {
    type In = [T];
    type Out = ();

    fn parse<'a>(&self, data: &'a Self::In) -> ParseResult<&'a Self::In, Self::Out> {
        if data.len() < self.want.len() {
            return Err(0);
        }
        if self.want == &data[0..self.want.len()] {
            return Ok((&data[self.want.len()..], ()))
        } else {
            return Err(0);
        }
    }
}

pub fn literal<T: 'static>(want: &'static [T]) -> Literal<T> {
    Literal {
        want
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        let abc = literal(b"abc");
		
		assert_eq!(abc.parse(b"abc"), Ok((b"".as_slice(), ())));      
        assert_eq!(abc.parse(b"def"), Err(0));
    }

    #[test]
    fn test_chained_parser() {
        let p = literal(b"abc").then(literal(b"def"));

        assert_eq!(p.parse(b"def"), Err(0));
        assert_eq!(p.parse(b"abc"), Err(0));
        assert_eq!(p.parse(b"abcdef"), Ok((b"".as_slice(), ((), ()))));
    }
}
