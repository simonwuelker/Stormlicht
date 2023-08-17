use dns::DNSError;
use std::collections::HashMap;
use std::io::{self, BufReader};
use std::net::{SocketAddr, TcpStream};
use url::{Host, URL};

use crate::response::Response;

const USER_AGENT: &str = "Stormlicht";
pub(crate) const HTTP_NEWLINE: &str = "\r\n";

#[derive(Debug)]
pub enum HTTPError {
    InvalidResponse,
    IO(io::Error),
    DNS(DNSError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    GET,
    POST,
}

impl Method {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GET => "GET",
            Self::POST => "POST",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Request {
    method: Method,
    path: String,
    headers: HashMap<String, String>,
    host: Host,
}

impl Request {
    /// Create a `GET` request for the specified URL
    ///
    /// # Panics
    /// This function panics if the url scheme is not `http`
    /// or the url does not have a `host`.
    pub fn get(url: URL) -> Self {
        assert_eq!(url.scheme, "http", "URL is not http");
        let host = url.host.expect("URL does not have a host");

        let mut headers = HashMap::with_capacity(2);
        headers.insert("User-Agent".to_string(), USER_AGENT.to_string());
        headers.insert("Host".to_string(), host.to_string());

        Self {
            method: Method::GET,
            path: format!("/{}", url.path.join("/")),
            headers,
            host,
        }
    }

    /// Serialize the request to the given [Writer](std::io::Write)
    fn write_to<W>(self, mut writer: W) -> Result<(), io::Error>
    where
        W: std::io::Write,
    {
        write!(
            writer,
            "{method} {path} HTTP/1.1{HTTP_NEWLINE}",
            method = self.method.as_str(),
            path = self.path
        )?;

        for (header, value) in &self.headers {
            write!(writer, "{}: {value}{HTTP_NEWLINE}", header.as_str())?;
        }
        write!(writer, "{HTTP_NEWLINE}")?;

        Ok(())
    }

    pub fn set_header(&mut self, header: &str, value: &str) {
        self.headers.insert(header.to_string(), value.to_string());
    }

    pub fn send(self) -> Result<Response, HTTPError> {
        // Resolve the hostname
        let ip = match &self.host {
            Host::Domain(host_str) | Host::OpaqueHost(host_str) => dns::Domain::new(host_str)
                .lookup()
                .map_err(HTTPError::DNS)?,
            Host::IPv4(_ip) => todo!(),
            Host::IPv6(_ip) => todo!(),
            Host::EmptyHost => todo!(),
        };

        // Establish a TCP connection with the host
        let mut stream = TcpStream::connect(SocketAddr::new(ip, 80)).map_err(HTTPError::IO)?;

        // Send our request
        self.write_to(&mut stream).map_err(HTTPError::IO)?;

        // Parse the response
        let mut reader = BufReader::new(stream);
        let response = Response::receive(&mut reader)?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::Request;

    #[test]
    fn basic_get_request() {
        let mut tcpstream: Vec<u8> = vec![];

        let mut request = Request::get(url::URL::try_from("http://www.example.com").unwrap());
        request.headers.clear();

        request.set_header("User-Agent", "test");
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
