use cpython::{PyTuple, Python, ToPyObject};
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

impl CoreEvent {
    pub fn make_py_tuple(self, py: Python) -> PyTuple {
        match self {
            CoreEvent::Exiting => py_wrap!(py, (0,)),
            CoreEvent::Started(t) => py_wrap!(py, (1, t as u16,)),
            CoreEvent::Stopped(t) => py_wrap!(py, (2, t as u16,)),
            CoreEvent::Connected(t, a) => py_wrap!(py, (100, t as u16, host_port(&a),)),
            CoreEvent::Disconnected(t, a) => py_wrap!(py, (101, t as u16, host_port(&a),)),
            CoreEvent::Message(t, a, e) => py_wrap!(
                py,
                (102, t as u16, host_port(&a), (e.protocol_id, e.message),)
            ),
            CoreEvent::Log(l, m) => py_wrap!(py, (200, l, m,)),
        }
    }
}
