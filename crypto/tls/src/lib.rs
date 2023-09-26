//! TLS 1.2 [RFC 5246](https://www.rfc-editor.org/rfc/rfc5246) implementation.

#![feature(cursor_remaining, array_chunks)]

pub mod alert;
pub mod certificate;
mod connection;
pub mod der;
pub mod handshake;
pub mod random;
pub mod record_layer;
mod server_name;

mod cipher_suite;
pub use cipher_suite::CipherSuite;
pub use connection::{TLSConnection, TLSError};
pub use server_name::ServerName;
