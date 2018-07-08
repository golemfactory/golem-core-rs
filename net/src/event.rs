use std::net::SocketAddr;

use error::Error;
use codec::message::Encapsulated;
use transport::TransportProtocol;

#[derive(Debug)]
pub enum Event {
    Exiting,
    Started(TransportProtocol, SocketAddr),
    Stopped(TransportProtocol, SocketAddr),
    Connected(TransportProtocol, SocketAddr, bool),
    Disconnected(TransportProtocol, SocketAddr),
    Message(TransportProtocol, SocketAddr, Encapsulated),
    Error(Error),
}
