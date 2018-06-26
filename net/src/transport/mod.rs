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
    Tcp,
    Udp,
    Unsupported,
}

macro_rules! _impl_into {
    ($($into:ty),+) => {
        $(impl Into<$into> for TransportProtocol {
            fn into(self) -> $into {
                match self {
                    TransportProtocol::Tcp => 6,
                    TransportProtocol::Udp => 17,
                    TransportProtocol::Unsupported => 0,
                }
            }
        })*
    }
}

macro_rules! _impl_from {
    ($($from:ty),+) => {
        $(impl From<$from> for TransportProtocol {
            fn from(value: $from) -> Self {
                match value {
                    6 => TransportProtocol::Tcp,
                    17 => TransportProtocol::Udp,
                    _ => TransportProtocol::Unsupported,
                }
            }
        })*
    }
}

_impl_into!(u8, u16, u32, i8, i16, i32);
_impl_from!(u8, u16, u32, i8, i16, i32);


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
