use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash)]
pub enum HTTPHeader {
    UserAgent,
    Other(String),
}

impl HTTPHeader {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            HTTPHeader::UserAgent => b"User-Agent".as_slice(),
            HTTPHeader::Other(name) => name.as_bytes(),
        }
    }
}

pub enum HTTPMethod {
    GET,
    POST,
}

pub struct HTTPRequest {
    method: HTTPMethod,
    path: String,
    headers: HashMap<HTTPHeader, String>,
}

impl HTTPRequest {
    pub fn get(url: &str) -> Self {
        todo!();
        // Self {
        //     method: HTTPMethod::GET,
        //     path: path,
        //     headers: HashMap::new(),
        // }
    }

    pub fn write_to<W>(self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let method_name = match self.method {
            HTTPMethod::GET => b"GET".as_slice(),
            HTTPMethod::POST => b"POST".as_slice(),
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

    pub fn set_header(&mut self, header: HTTPHeader, value: &str) {
        self.headers.insert(header, value.to_string());
    }
}

mod tests {
    use super::*;

    #[test]
    fn basic_get_request() {
        let mut tcpstream: Vec<u8> = vec![];
        let mut request = HTTPRequest::get("www.example.com");
        request.set_header(HTTPHeader::UserAgent, "test");
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
