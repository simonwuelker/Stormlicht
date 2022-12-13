#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken,
}

pub type ParseResult<'a, T> = Result<&'a[T], ParseError>;
pub type Parser<T> = Box<dyn for<'a> Fn(&'a [T]) -> ParseResult<T>>;

pub fn literal<T: 'static + PartialEq>(to_match: &'static [T]) -> Parser<T> {
    Box::new(
        move |data: &[T]| {
            if &data[..to_match.len()] == to_match {
                Ok(&data[..to_match.len()])
            } else {
            	Err(ParseError::UnexpectedToken)
            }
        }
    )
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        let parser = literal(b"abc");
		
		assert_eq!(parser(b"abc"), Ok(b"abc".as_slice()));      
        assert_eq!(parser(b"def"), Err(ParseError::UnexpectedToken));
    }
}
