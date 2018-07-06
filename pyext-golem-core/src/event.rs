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

impl ToPyObject for CoreEvent {
    type ObjectType = PyTuple;

    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        unimplemented!()
    }

    fn into_py_object(self, py: Python) -> Self::ObjectType {
        match self {
            CoreEvent::Exiting => {
                py_wrap!(py, (0,))
            },
            CoreEvent::Started(transport, address) => {
                py_wrap!(py, (1, transport as u16, host_port(&address)))
            }
            CoreEvent::Stopped(transport, address) => {
                py_wrap!(py, (2, transport as u16, host_port(&address)))
            }
            CoreEvent::Connected(transport, address, initiator) => {
                py_wrap!(py, (100, transport as u16, host_port(&address), initiator))
            }
            CoreEvent::Disconnected(transport, address) => {
                py_wrap!(py, (101, transport as u16, host_port(&address)))
            }
            CoreEvent::Message(transport, address, encapsulated) => {
                let bytes: PyBytes = PyBytes::new(py, &encapsulated.message[..]);
                let message = py_wrap!(py, (encapsulated.protocol_id, bytes));
                py_wrap!(py, (102, transport as u16, host_port(&address), message))
            }
            CoreEvent::Log(level, message) => {
                py_wrap!(py, (200, level, message))
            },
        }
    }
}
