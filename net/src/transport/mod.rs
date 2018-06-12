pub mod message;
pub mod session;
pub mod tcp;
pub mod udp;

use std::{cmp, fmt, net};
use std::io::{Error, ErrorKind};

use actix::{AsyncContext, Unsync};

use super::logic::Logic;
use self::tcp::{TcpTransportAddr, TcpSessionAddr};
use self::udp::UdpTransportAddr;

pub type AddrType = Unsync;

//
// Transport definition
//

/// Available transports
#[derive(Hash, Debug, Copy, Clone, PartialEq)]
pub enum Transport
{
    Tcp,
    Udp
}

impl fmt::Display for Transport
{
    fn fmt(&self, f: &mut fmt::Formatter)
           -> Result<(), fmt::Error>
    {
        match *self {
            Transport::Tcp => write!(f, "TCP"),
            Transport::Udp => write!(f, "UDP"),
        }
    }
}

impl cmp::Eq for Transport
{}

/// Actor addresses
#[derive(Clone)]
pub enum TransportRouter<L>
where
    L: Logic,
    L::Context: AsyncContext<L>,
{
    Tcp(TcpTransportAddr<L>),
    Udp(UdpTransportAddr<L>),
}

pub enum TransportSession<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    Tcp(TcpSessionAddr<L>),
    Udp(UdpTransportAddr<L>),
}

impl<L> Clone for TransportSession<L>
where
    L: Logic + 'static,
    L::Context: AsyncContext<L>,
{
    fn clone(&self) -> Self {
        match *self {
            TransportSession::Tcp(ref a) => TransportSession::Tcp(a.clone()),
            TransportSession::Udp(ref a) => TransportSession::Udp(a.clone()),
        }
    }
}

//
// Helper functions
//

pub fn mailbox_io_error()
    -> Error
{
    Error::new(ErrorKind::Other, "mailbox error")
}

pub fn not_listening_error(name: &str)
    -> Error
{
    Error::new(
        ErrorKind::NotConnected,
        format!("{} transport is not listening", name)
    )
}

pub fn not_connected_error(transport: &Transport, address: &net::SocketAddr)
    -> Error
{
    Error::new(
        ErrorKind::NotConnected,
        format!("{} is not connected to {}", transport, address)
    )
}


pub fn log_error<T, E>(result: Result<T, E>)
where
    E: fmt::Display,
{
    match result {
        Ok(_) => (),
        Err(e) => eprintln!("Error: {}", e),
    }
}
