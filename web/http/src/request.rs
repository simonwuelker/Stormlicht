use std::collections::HashMap;
use url::{URL, Host};
use std::net::{TcpStream, SocketAddr};
use std::io::{Read, Write};

use crate::response::Response;

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
            Host::Domain(domain) | Host::OpaqueHost(domain) => dns::resolve(b"www.ecosia.com").unwrap(),
            Host::IP(ip) => todo!(),
            Host::EmptyHost => todo!(),
        };

        // Establish a tcp connection
        let mut stream = TcpStream::connect(SocketAddr::new(ip, 80)).unwrap();

        todo!()
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
