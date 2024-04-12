#[must_use = "DecodeResult may indicate an error"]
#[derive(Clone, Copy, Debug)]
pub enum DecodeResult {
    /// <https://encoding.spec.whatwg.org/#finished>
    Finished,

    Item(char),

    /// <https://encoding.spec.whatwg.org/#error>
    Error,

    /// <https://encoding.spec.whatwg.org/#continue>
    Continue,
}

#[derive(Clone, Copy, Debug)]
pub struct DecodeError {
    byte_offset: usize,
}

pub struct Context<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Context<'a> {
    pub fn go_back(&mut self) {
        self.offset -= 1;
    }
}

pub trait Decoder: Default {
    fn eat_byte(&mut self, context: &mut Context<'_>) -> DecodeResult;

    fn next_char(&mut self, context: &mut Context<'_>) -> Result<Option<char>, DecodeError> {
        loop {
            match self.eat_byte(context) {
                DecodeResult::Continue => {},
                DecodeResult::Item(c) => return Ok(Some(c)),
                DecodeResult::Finished => return Ok(None),
                DecodeResult::Error => {
                    return Err(DecodeError {
                        byte_offset: context.offset,
                    })
                },
            }
        }
    }

    fn fully_decode<P: AsRef<[u8]>>(bytes: P) -> Result<String, DecodeError> {
        let mut result = String::new();
        let mut decoder = Self::default();
        let mut context = Context {
            bytes: bytes.as_ref(),
            offset: 0,
        };

        while let Some(c) = decoder.next_char(&mut context)? {
            result.push(c);
        }

        Ok(result)
    }
}

impl<'a> Iterator for Context<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.bytes.get(self.offset)?;
        self.offset += 1;

        Some(*byte)
    }
}
