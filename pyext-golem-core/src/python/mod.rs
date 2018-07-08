use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use cpython::*;
use net::socket_address;
use net::event::Event;

use error::*;
use logging::*;

pub type PyShared = Arc<Mutex<Option<PyObject>>>;

//
// Helper macros
//

/// Convert a Python value to a native type
#[macro_export]
macro_rules! py_extract {
    ($input:expr) => {{
        use cpython::Python;

        let gil = Python::acquire_gil();
        let py = gil.python();

        py_extract!(py, $input)
    }};
    ($py:expr, $input:expr) => {{
        use cpython::PythonObject;

        $input.into_object().extract($py)
    }};
    ($py:expr, $input:expr, $to:ty) => {{
        use cpython::{PyErr, PythonObject};

        let result: Result<$to, PyErr> = $input.into_object().extract($py);
        result
    }};
}

/// Convert a native type value to Python type
#[macro_export]
macro_rules! py_wrap {
    ($py:expr, $input:expr) => {{
        $input.to_py_object($py)
    }};
    ($py:expr, $input:expr, $to:ty) => {{
        let result: $to = $input.to_py_object($py);
        result
    }};
}

//
// Helper functions
//


// SocketAddr

struct SocketAddrWrapper<'a> {
    address: &'a SocketAddr,
}

impl<'a> Into<(String, u16)> for SocketAddrWrapper<'a> {
    fn into(self) -> (String, u16) {
        let host = format!("{}", self.address.ip());
        let port: u16 = self.address.port();
        (host, port)
    }
}

pub fn host_port(address: &SocketAddr) -> (String, u16) {
    SocketAddrWrapper { address: &address }.into()
}

pub fn to_socket_address(
    py: Python,
    py_host: PyString,
    py_port: PyLong,
) -> Result<SocketAddr, ModuleError> {
    let host: String = py_extract!(py, py_host)?;
    let port: u16 = py_extract!(py, py_port)?;
    let address: SocketAddr = socket_address(&host, port)?;

    Ok(address)
}

pub fn from_socket_address(py: Python, address: SocketAddr) -> (PyString, PyInt) {
    let (host, port) = host_port(&address);
    let py_host: PyString = py_wrap!(py, host);
    let py_port: PyInt = py_wrap!(py, port);

    (py_host, py_port)
}

// Event

struct EventWrapper {
    event: Event,
}

impl ToPyObject for EventWrapper {
    type ObjectType = PyTuple;

    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        unimplemented!()
    }

    fn into_py_object(self, py: Python) -> Self::ObjectType {
        match self.event {
            Event::Exiting => {
                py_wrap!(py, (0,))
            },
            Event::Started(transport, address) => {
                py_wrap!(py, (1, transport as u16, host_port(&address)))
            }
            Event::Stopped(transport, address) => {
                py_wrap!(py, (2, transport as u16, host_port(&address)))
            }
            Event::Connected(transport, address, initiator) => {
                py_wrap!(py, (100, transport as u16, host_port(&address), initiator))
            }
            Event::Disconnected(transport, address) => {
                py_wrap!(py, (101, transport as u16, host_port(&address)))
            }
            Event::Message(transport, address, encapsulated) => {
                let bytes: PyBytes = PyBytes::new(py, &encapsulated.message[..]);
                let message = py_wrap!(py, (encapsulated.protocol_id, bytes));
                py_wrap!(py, (102, transport as u16, host_port(&address), message))
            }
            Event::Error(e) => {
                let level = LogLevel::from(e.severity);
                let message = e.description();
                py_wrap!(py, (200, level, message))
            },
        }
    }
}

pub fn event_into(py: Python, event: Event) -> PyTuple {
    EventWrapper{ event }.into_py_object(py)
}

//
// Tests
//

#[cfg(test)]
mod tests {
    use cpython::{Python, ToPyObject};

    #[test]
    fn extract() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let value: u16 = py_extract!(5_i32.to_py_object(py)).unwrap();

        assert_eq!(5_u16, value);
    }
}
