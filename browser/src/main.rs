use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

use log::{info, error};
use simple_logger::SimpleLogger;

use http::request::*;
use http::response::*;

fn main() {
    SimpleLogger::new().init().unwrap();

    let request = HTTPRequest::new(HTTPMethod::GET, "/".to_string());
    let url = "google.com:80";
    match TcpStream::connect(url) {
        Ok(mut stream) => {
            info!(target: "network", "Connected to {url}");

            request.write_to(&mut stream).unwrap();

            let mut buffer: Vec<u8> = vec![];
            let mut fixed_size_buffer = [0; 0x100];
            loop {
                match stream.read(&mut fixed_size_buffer) {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            break;
                        } else {
                            buffer.extend(&fixed_size_buffer[..bytes_read]);
                            if buffer.ends_with(b"\r\n\r\n") {
                                break;
                            }
                        }

                    },
                    Err(e) => error!("Read failed:  {e}"),
                }
            }

            info!("Finished reading response");
            let response = HTTPResponse::parse(&buffer).unwrap().1;
            info!("Reponse code: {:?}", response.status);
        }
        Err(e) => {
            error!("Failed to connect to {url}: {e}");
        }
    }
}
