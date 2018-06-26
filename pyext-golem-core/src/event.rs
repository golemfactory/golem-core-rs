use cpython::{PyTuple, ToPyObject};
use std::net::SocketAddr;

use net::codec::message::Encapsulated;
use net::transport::TransportProtocol;

use logging::LogLevel;
use python::*;

#[derive(Debug)]
pub enum CoreEvent {
    Exiting,
    Started(TransportProtocol),
    Stopped(TransportProtocol),
    Connected(TransportProtocol, SocketAddr),
    Disconnected(TransportProtocol, SocketAddr),
    Message(TransportProtocol, SocketAddr, Encapsulated),
    Log(LogLevel, String),
}

impl Into<PyTuple> for CoreEvent {
    fn into(self) -> PyTuple {
        match self {
            CoreEvent::Exiting => py_from!((0,)),
            CoreEvent::Started(t) => py_from!((1, t as u16,)),
            CoreEvent::Stopped(t) => py_from!((2, t as u16,)),
            CoreEvent::Connected(t, a) => py_from!((100, t as u16, host_port(&a),)),
            CoreEvent::Disconnected(t, a) => py_from!((101, t as u16, host_port(&a),)),
            CoreEvent::Message(t, a, e) => {
                py_from!((102, t as u16, host_port(&a), (e.protocol_id, e.message),))
            }
            CoreEvent::Log(l, m) => py_from!((200, l, m,)),
        }
    }
}
