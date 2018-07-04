use cpython::{PyBytes, PyTuple, Python, ToPyObject};
use std::net::SocketAddr;

use net::codec::message::Encapsulated;
use net::transport::TransportProtocol;

use logging::LogLevel;
use python::*;

#[derive(Debug)]
pub enum CoreEvent {
    Exiting,
    Started(TransportProtocol, SocketAddr),
    Stopped(TransportProtocol, SocketAddr),
    Connected(TransportProtocol, SocketAddr, bool),
    Disconnected(TransportProtocol, SocketAddr),
    Message(TransportProtocol, SocketAddr, Encapsulated),
    Log(LogLevel, String),
}

impl CoreEvent {
    pub fn make_py_tuple(self, py: Python) -> PyTuple {
        match self {
            CoreEvent::Exiting              => py_wrap!(py, (0,)),
            CoreEvent::Started(t, a)        => py_wrap!(py, (1, t as u16, host_port(&a))),
            CoreEvent::Stopped(t, a)        => py_wrap!(py, (2, t as u16, host_port(&a))),
            CoreEvent::Connected(t, a, i)   => py_wrap!(py, (100, t as u16, host_port(&a), i)),
            CoreEvent::Disconnected(t, a)   => py_wrap!(py, (101, t as u16, host_port(&a))),
            CoreEvent::Message(t, a, e)     => {
                let bytes = PyBytes::new(py, &e.message[..]);
                let message = py_wrap!(py, (e.protocol_id, bytes));
                                               py_wrap!(py, (102, t as u16, host_port(&a), message))
            },
            CoreEvent::Log(l, m)            => py_wrap!(py, (200, l, m)),
        }
    }
}
