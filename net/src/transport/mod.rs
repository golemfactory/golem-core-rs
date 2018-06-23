pub mod error;
pub mod message;
pub mod tcp;
pub mod udp;

use std::clone::Clone;
use std::fmt;

use actix::AsyncContext;

use self::tcp::{TcpActorAddr, TcpSessionAddr};
use self::udp::UdpActorAddr;
use super::network::Network;

/// Available transport protocols
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum TransportProtocol {
    Tcp = 6,
    Udp = 17,
    Unsupported,
}

impl From<u16> for TransportProtocol {
    fn from(value: u16) -> Self {
        match value {
            6 => TransportProtocol::Tcp,
            17 => TransportProtocol::Udp,
            _ => TransportProtocol::Unsupported,
        }
    }
}

impl fmt::Display for TransportProtocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

/// Transport router addresses
pub enum Transport<N>
where
    N: Network,
    N::Context: AsyncContext<N>,
{
    Tcp(TcpActorAddr<N>),
    Udp(UdpActorAddr<N>),
}

/// Transport session addresses
pub enum TransportSession<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    Tcp(TcpSessionAddr<N>),
    Udp(UdpActorAddr<N>),
}

impl<N> Clone for TransportSession<N>
where
    N: Network + 'static,
    N::Context: AsyncContext<N>,
{
    fn clone(&self) -> Self {
        match *self {
            TransportSession::Tcp(ref a) => TransportSession::Tcp(a.clone()),
            TransportSession::Udp(ref a) => TransportSession::Udp(a.clone()),
        }
    }
}
