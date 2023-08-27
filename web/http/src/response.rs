//! HTTP/1.1 response parser

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

use sl_std::iter::MultiElementSplit;

use crate::{
    request::{Context, HTTPError, HTTP_NEWLINE},
    status_code::StatusCode,
};

/// Like [BufReader::read_until], except the needle may have arbitrary length
fn read_until<R: std::io::Read>(
    reader: &mut BufReader<R>,
    needle: &[u8],
) -> Result<Vec<u8>, std::io::Error> {
    let mut result = vec![];

    loop {
        match reader
            .buffer()
            .windows(needle.len())
            .position(|w| w == needle)
        {
            Some(i) => {
                let bytes_to_consume = i + needle.len();

                result.extend(&reader.buffer()[..bytes_to_consume]);
                reader.consume(bytes_to_consume);
                return Ok(result);
            },
            None => {
                result.extend(reader.buffer());
                reader.consume(reader.capacity());
                reader.fill_buf()?;
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    context: Context,
}

impl Response {
    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn get_header(&self, header: &str) -> Option<&str> {
        for (key, value) in &self.headers {
            if key.eq_ignore_ascii_case(header) {
                return Some(value);
            }
        }
        None
    }

    // FIXME: Requiring a BufReader here is kind of ugly
    /// Read a [Response] from the given [Reader](std::io::Read)
    ///
    /// This requires a [BufReader] because we make direct use of its buffer
    pub fn receive<R: std::io::Read>(
        reader: &mut BufReader<R>,
        context: Context,
    ) -> Result<Self, HTTPError> {
        // TODO all of this is very insecure - we blindly trust the size in Transfer-Encoding: chunked,
        // no timeouts, stuff like that.
        let needle = b"\r\n\r\n";
        let header_bytes = read_until(reader, needle).map_err(HTTPError::IO)?;

        let mut response_lines =
            MultiElementSplit::new(&header_bytes, |w: &[u8; 2]| w == HTTP_NEWLINE.as_bytes());

        let mut status_line_words = response_lines
            .next()
            .ok_or(HTTPError::InvalidResponse)?
            .split(|&b| b == b' ')
            .filter(|word| !word.is_empty());

        if !matches!(status_line_words.next(), Some(b"HTTP/1.1")) {
            return Err(HTTPError::InvalidResponse);
        }

        // Parse status code
        let status: StatusCode =
            std::str::from_utf8(status_line_words.next().ok_or(HTTPError::InvalidResponse)?)
                .map_err(|_| HTTPError::InvalidResponse)?
                .parse()
                .map_err(|_| HTTPError::InvalidResponse)?;

        // What follows is a textual description of the error code ("OK" for 200) - we don't care about that

        // Parse the response headers
        let mut headers = HashMap::new();
        for header_line in response_lines {
            // An empty header indicates the end of the list of headers
            if header_line.is_empty() {
                break;
            }

            let separator = header_line
                .iter()
                .position(|&elem| elem == b':')
                .ok_or(HTTPError::InvalidResponse)?;

            let key = &header_line[..separator];
            let value = &header_line[separator + 1..];
            headers.insert(
                std::str::from_utf8(key)
                    .map_err(|_| HTTPError::InvalidResponse)?
                    .trim()
                    .to_owned(),
                std::str::from_utf8(value)
                    .map_err(|_| HTTPError::InvalidResponse)?
                    .trim()
                    .to_owned(),
            );
        }

        let mut response = Self {
            status,
            headers,
            body: vec![],
            context,
        };

        // Anything after the headers is the actual response body
        // The length of the body depends on the headers that were sent
        if let Some(transfer_encoding) = response.get_header("Transfer-encoding") {
            match transfer_encoding {
                "chunked" => loop {
                    let size_bytes_with_newline =
                        read_until(reader, HTTP_NEWLINE.as_bytes()).map_err(HTTPError::IO)?;
                    let size_bytes = &size_bytes_with_newline
                        [..size_bytes_with_newline.len() - HTTP_NEWLINE.len()];

                    if size_bytes.is_empty() {
                        break;
                    }

                    let size = usize::from_str_radix(
                        std::str::from_utf8(size_bytes).map_err(|_| HTTPError::InvalidResponse)?,
                        16,
                    )
                    .map_err(|_| HTTPError::InvalidResponse)?;

                    if size == 0 {
                        break;
                    }

                    let mut buffer = vec![0; size];
                    reader.read_exact(&mut buffer).map_err(HTTPError::IO)?;
                    response.body.extend(&buffer)
                },
                _ => {
                    log::warn!("Unknown transfer encoding: {transfer_encoding}");
                    return Err(HTTPError::InvalidResponse);
                },
            }
        } else if let Some(content_length) = response.get_header("Content-Length") {
            let mut buffer =
                vec![0; str::parse(content_length).map_err(|_| HTTPError::InvalidResponse)?];
            reader.read_exact(&mut buffer).map_err(HTTPError::IO)?;
            response.body.extend(&buffer)
        } else {
            log::warn!("Neither Transfer-Encoding nor Content-Length were provided, we don't know how to decode the body!");
            return Err(HTTPError::InvalidResponse);
        }

        Ok(response)
    }
}
