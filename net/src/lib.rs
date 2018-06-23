extern crate bincode;
extern crate byteorder;
extern crate bytes;
extern crate futures;
extern crate serde;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_io;
extern crate tokio_tcp;
extern crate tokio_udp;

#[macro_use]
extern crate actix;
#[macro_use]
extern crate serde_derive;

pub mod codec;
pub mod network;
pub mod transport;

use std::net::{AddrParseError, IpAddr, SocketAddr};

pub fn socket_address(host: &String, port: u16) -> Result<SocketAddr, AddrParseError> {
    let ip: IpAddr = host.parse()?;
    Ok(SocketAddr::new(ip, port))
}
