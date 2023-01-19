use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::{SocketAddr, TcpStream};
use url::{Host, URL};

use crate::response::{parse_response, Response};

use anyhow::{anyhow, Context, Result};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Unknown transfer encoding: {:?}", .0)]
    UnknownTransferEncoding(String),
}

#[derive(PartialEq, Eq, Hash)]
pub enum Header {
    UserAgent,
    Other(String),
}

impl Header {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::UserAgent => b"User-Agent".as_slice(),
            Self::Other(name) => name.as_bytes(),
        }
    }
}

pub enum Method {
    GET,
    POST,
}

pub struct Request {
    method: Method,
    path: String,
    headers: HashMap<Header, String>,
    host: Host,
}

#[derive(Debug)]
pub enum CreateRequestError {
    NotHTTP,
    MissingHost,
}

/// Like [BufReader::read_until], except the needle may have arbitrary length
fn read_until<R: Read>(reader: &mut BufReader<R>, needle: &[u8]) -> Result<Vec<u8>> {
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

impl Request {
    pub fn get(url: URL) -> Result<Self, CreateRequestError> {
        if url.scheme != "http" {
            return Err(CreateRequestError::NotHTTP);
        }

        Ok(Self {
            method: Method::GET,
            path: "/".to_string(),
            headers: HashMap::new(),
            host: url.host.ok_or(CreateRequestError::MissingHost)?,
        })
    }

    pub fn write_to<W>(self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let method_name = match self.method {
            Method::GET => b"GET".as_slice(),
            Method::POST => b"POST".as_slice(),
        };

        writer.write_all(method_name)?;
        writer.write_all(b" ")?;
        writer.write_all(self.path.as_bytes())?;
        writer.write_all(b" HTTP/1.1\r\n")?;

        for (header, value) in &self.headers {
            writer.write_all(header.as_bytes())?;
            writer.write_all(b": ")?;
            writer.write_all(value.as_bytes())?;
            writer.write_all(b"\r\n")?;
        }
        writer.write_all(b"\r\n")?;

        Ok(())
    }

    pub fn set_header(&mut self, header: Header, value: &str) {
        self.headers.insert(header, value.to_string());
    }

    pub fn send(self) -> Result<Response> {
        // resolve the hostname
        let ip = match &self.host {
            Host::Domain(host_str) | Host::OpaqueHost(host_str) => {
                dns::resolve(&dns::Domain::new(host_str))?
            },
            Host::IP(_ip) => todo!(),
            Host::EmptyHost => todo!(),
        };

        // Establish a tcp connection
        let mut stream = TcpStream::connect(SocketAddr::new(ip, 80))?;

        // Send our request
        self.write_to(&mut stream)?;

        // Parse the response
        // TODO all of this is very insecure - we blindly trust the size in Transfer-Encoding: chunked,
        // no timeouts, stuff like that.
        let mut reader = BufReader::new(stream);
        let needle = b"\r\n\r\n";
        let header_bytes = read_until(&mut reader, needle)?;

        let mut response = parse_response(&header_bytes)
            .map_err(|_| anyhow!("Invalid response"))?
            .1;

        if let Some(transfer_encoding) = response.get_header("Transfer-encoding") {
            match transfer_encoding {
                "chunked" => {
                    let needle = b"\r\n";
                    loop {
                        let size_bytes_with_newline = read_until(&mut reader, needle)?;
                        let size_bytes = &size_bytes_with_newline
                            [..size_bytes_with_newline.len() - needle.len()];
                        let size = size_bytes
                            .iter()
                            .fold(0, |acc, x| acc * 16 + hex_digit(*x) as usize);

                        if size == 0 {
                            break;
                        }

                        let mut buffer = vec![0; size];
                        reader.read_exact(&mut buffer)?;
                        response.body.extend(&buffer)
                    }
                },
                _ => {
                    return Err(RequestError::UnknownTransferEncoding(
                        transfer_encoding.to_string(),
                    )
                    .into())
                },
            }
        } else if let Some(content_length) = response.get_header("Content-Length") {
            let mut buffer = vec![
                0;
                str::parse(content_length)
                    .context("Failed to parse Content-Length value")?
            ];
            reader.read_exact(&mut buffer)?;
            response.body.extend(&buffer)
        }

        Ok(response)
    }
}

fn hex_digit(byte: u8) -> u8 {
    match byte {
        65..=90 => byte - 55,  // ascii lowercase
        97..=122 => byte - 87, // ascii uppercase
        48..=57 => byte - 48,  // ascii digit
        _ => panic!("invalid hex digit"),
    }
}

mod tests {

    #[test]
    fn basic_get_request() {
        let mut tcpstream: Vec<u8> = vec![];

        let mut request = super::Request::get(url::URL::from("http://www.example.com")).unwrap();
        request.headers.clear();

        request.set_header(super::Header::UserAgent, "test");
        request.write_to(&mut tcpstream).unwrap();
        assert_eq!(
            String::from_utf8(tcpstream).unwrap(),
            "\
        GET / HTTP/1.1\r\n\
        User-Agent: test\r\n\
        \r\n"
        );
    }
}
