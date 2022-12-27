use std::collections::HashMap;
use url::{URL, Host};
use std::net::{TcpStream, SocketAddr};
use std::io::{Read, Write};

use crate::response::{Response, parse_response};

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

    pub fn send(self) -> Result<Response, ()> {
        // resolve the hostname
        let ip = match &self.host {
            Host::Domain(host_str) | Host::OpaqueHost(host_str) => dns::resolve(&dns::Domain::new(host_str)).unwrap(),
            Host::IP(ip) => todo!(),
            Host::EmptyHost => todo!(),
        };

        // Establish a tcp connection
        let mut stream = TcpStream::connect(SocketAddr::new(ip, 80)).unwrap();
        self.write_to(&mut stream);

        let mut response_bytes = vec![];
        let mut response_body_bytes = vec![];
        let mut buffer = [0; 0x100];

        let needle = b"\r\n\r\n";
        loop {
            stream.read_exact(&mut buffer).map_err(|_| ())?;

            match buffer.windows(needle.len()).position(|w| w == needle) {
                Some(i) => {
                    response_bytes.extend(&buffer[..i  + needle.len()]);
                    response_body_bytes.extend(&buffer[i  + needle.len()..]);
                    break;
                }
                None => {
                    response_bytes.extend(&buffer);
                }
            }
        }

        println!("{:?}", String::from_utf8(response_body_bytes));

        let response = parse_response(&response_bytes).unwrap().1;

        if let Some(transfer_encoding) = response.get_header("Transfer-encoding") {
            println!("chunked!");
        }

        Ok(response)
    }
}

mod tests {
    use super::*;

    #[test]
    fn basic_get_request() {
        let mut tcpstream: Vec<u8> = vec![];
        let mut request = Request::get(URL::from("www.example.com")).unwrap();
        request.set_header(Header::UserAgent, "test");
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
