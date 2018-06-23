use std::fmt;
use std::io::{Error, ErrorKind};
use std::net;

use transport::TransportProtocol;

pub fn mailbox_io_error() -> Error {
    Error::new(ErrorKind::Other, "mailbox error")
}

pub fn unsupported_error(name: &str) -> Error {
    Error::new(ErrorKind::Other, format!("{} is not supported", name))
}

pub fn cannot_listen_error(name: &str, address: &net::SocketAddr) -> Error {
    Error::new(
        ErrorKind::AddrInUse,
        format!("{} cannot listen on address {}", name, address),
    )
}

pub fn not_listening_error(name: &str) -> Error {
    Error::new(
        ErrorKind::NotConnected,
        format!("{} is not listening", name),
    )
}

pub fn not_connected_error(transport: &TransportProtocol, address: &net::SocketAddr) -> Error {
    Error::new(
        ErrorKind::NotConnected,
        format!("{} is not connected to {}", transport, address),
    )
}

pub fn invalid_address(host: &String, port: i64) -> Error {
    Error::new(
        ErrorKind::Other,
        format!("Invalid address: {}:{}", host, port),
    )
}

pub fn log_error<T, E>(result: Result<T, E>)
    where
        E: fmt::Display,
{
    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }
}
