use std::{
    io::{self, BufReader},
    net::{SocketAddr, TcpStream},
};

use dns::DNSError;
use tls::TLSConnection;
use url::{Host, URL};

use crate::{response::Response, Headers, StatusCode};

const USER_AGENT: &str = "Stormlicht";
pub(crate) const HTTP_NEWLINE: &str = "\r\n";

const MAX_REDIRECTS: usize = 32;

#[derive(Debug)]
pub enum HTTPError {
    InvalidResponse,
    Status(StatusCode),
    IO(io::Error),
    DNS(DNSError),
    TLS(tls::TLSError),
    RedirectLoop,
    NonHTTPRedirect,
    NonHTTPURl,
}

#[derive(Clone, Debug)]
pub struct Context {
    /// The number of times we were redirected while completing
    /// the original request
    pub num_redirections: usize,

    /// The [URL] that is currently being loaded
    pub url: URL,
}

impl Context {
    #[must_use]
    pub fn new(url: URL) -> Self {
        Self {
            num_redirections: 0,
            url,
        }
    }
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
    headers: Headers,
    context: Context,
}

trait Stream: io::Read + io::Write {}

impl<S> Stream for S where S: io::Read + io::Write {}

impl Request {
    /// Create a `GET` request for the specified URL
    ///
    /// # Panics
    /// This function panics if the url scheme is not `http`
    /// or the url does not have a `host`.
    #[must_use]
    pub fn get(url: &URL) -> Self {
        assert!(
            matches!(url.scheme().as_str(), "http" | "https"),
            "URL is not http(s)"
        );

        let mut headers = Headers::with_capacity(3);
        headers.set("User-Agent", USER_AGENT.to_string());
        headers.set("Accept", "*/*".to_string());
        headers.set(
            "Host",
            url.host().expect("URL does not have a host").to_string(),
        );

        Self {
            method: Method::GET,
            headers,
            context: Context::new(url.clone()),
        }
    }

    #[must_use]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    #[must_use]
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Serialize the request to the given [Writer](std::io::Write)
    fn write_to<W>(&self, mut writer: W) -> Result<(), io::Error>
    where
        W: std::io::Write,
    {
        write!(
            writer,
            "{method} /{path} HTTP/1.1{HTTP_NEWLINE}",
            method = self.method.as_str(),
            path = self.context.url.path().join("/")
        )?;

        for (header, value) in self.headers.iter() {
            write!(writer, "{header}: {value}{HTTP_NEWLINE}")?;
        }
        write!(writer, "{HTTP_NEWLINE}")?;

        Ok(())
    }

    pub fn send(&mut self) -> Result<Response, HTTPError> {
        // Establish a connection with the host
        let host = self.context.url.host().expect("url does not have a host");
        match self.context.url.scheme().as_str() {
            "http" => {
                // Resolve the hostname
                let ip = match &host {
                    Host::Domain(host) | Host::OpaqueHost(host) => dns::Domain::new(host.as_str())
                        .lookup()
                        .map_err(HTTPError::DNS)?,
                    Host::IPv4(_ip) => todo!(),
                    Host::IPv6(_ip) => todo!(),
                    Host::EmptyHost => todo!(),
                };

                let stream = TcpStream::connect(SocketAddr::new(ip, 80)).map_err(HTTPError::IO)?;
                self.send_on_stream(stream)
            },
            "https" => {
                let server_name = match host {
                    Host::Domain(host) | Host::OpaqueHost(host) => {
                        tls::ServerName::Domain(host.to_string())
                    },
                    _ => todo!(),
                };
                let stream = TLSConnection::establish(server_name).map_err(HTTPError::TLS)?;
                self.send_on_stream(stream)
            },
            _ => Err(HTTPError::NonHTTPURl),
        }
    }

    fn send_on_stream<S: Stream>(&mut self, mut stream: S) -> Result<Response, HTTPError> {
        // Send our request
        self.write_to(&mut stream).map_err(HTTPError::IO)?;

        // Parse the response
        let mut reader = BufReader::new(stream);
        let response = Response::receive(&mut reader, self.context.clone())?;

        if response.status().is_error() {
            log::warn!("HTTP Request failed: {:?}", response.status());
            return Err(HTTPError::Status(response.status()));
        }

        if response.status().is_redirection() {
            if let Some(relocation) = response
                .headers()
                .get("Location")
                .and_then(|v| URL::try_from(v).ok())
            {
                log::info!(
                    "{current_url} redirects to {redirect_url} ({status_code:?})",
                    current_url = self.context.url.serialize(url::ExcludeFragment::No),
                    redirect_url = relocation.serialize(url::ExcludeFragment::No),
                    status_code = response.status()
                );

                if relocation.scheme().as_str() != "http" {
                    log::error!(
                        "Cannot load non-http redirect url: {redirect_url}",
                        redirect_url = relocation.serialize(url::ExcludeFragment::Yes)
                    );
                    return Err(HTTPError::NonHTTPRedirect);
                }

                self.context.num_redirections += 1;

                if self.context.num_redirections >= MAX_REDIRECTS {
                    log::warn!("Too many HTTP redirections ({MAX_REDIRECTS}), stopping");
                    return Err(HTTPError::RedirectLoop);
                }

                self.headers.set(
                    "Host",
                    relocation
                        .host()
                        .expect("relocation url does not have a host")
                        .to_string(),
                );
                self.context.url = relocation;
                return self.send();
            } else {
                log::warn!("HTTP response indicates redirection, but no new URL could be found");
            }
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::Request;

    #[test]
    fn basic_get_request() {
        let mut tcpstream: Vec<u8> = vec![];

        let mut request = Request::get(&url::URL::try_from("http://www.example.com").unwrap());
        request.headers_mut().clear(); // No default headers

        request.headers_mut().set("User-Agent", "test".to_string());
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
