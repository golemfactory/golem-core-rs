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
    Unsupported = 0,
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

_impl_from!(u16, u32, u64, i16, i32, i64);


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

#[cfg(test)]
mod tests {

    #[cfg(test)]
    mod protocol {

        use transport::TransportProtocol;

        #[test]
        fn test_as() {
            assert_eq!(6, TransportProtocol::Tcp as u16);
            assert_eq!(17, TransportProtocol::Udp as u16);
            assert_eq!(0, TransportProtocol::Unsupported as u16);
        }

        #[test]
        fn test_from() {
            assert_eq!(TransportProtocol::Tcp, TransportProtocol::from(6));
            assert_eq!(TransportProtocol::Udp, TransportProtocol::from(17));
            assert_eq!(TransportProtocol::Unsupported, TransportProtocol::from(0));
        }
    }
}
