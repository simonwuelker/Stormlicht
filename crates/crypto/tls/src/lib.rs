//! TLS 1.2 [RFC 5246](https://www.rfc-editor.org/rfc/rfc5246) implementation.

#![feature(
    cursor_remaining,
    array_chunks,
    result_flattening,
    exclusive_range_pattern,
    ascii_char,
    ascii_char_variants
)]

pub mod alert;
pub mod certificate;
mod connection;
pub mod der;
mod encoding;
pub mod handshake;
pub mod random;
pub mod record_layer;
mod server_name;
mod session;

use encoding::Encoding;
use session::SessionId;
mod cipher_suite;
pub use cipher_suite::CipherSuite;
pub use connection::{TLSConnection, TLSError};
pub use server_name::ServerName;
