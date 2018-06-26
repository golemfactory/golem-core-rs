use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use cpython::{PyInt, PyLong, PyObject, PyString, ToPyObject};

use error::*;
use net::socket_address;

pub type PyShared = Arc<Mutex<Option<PyObject>>>;

//
// Helper macros
//

/// Convert a Python value to a native type
#[macro_export]
macro_rules! py_extract {
    ($input:expr, $to:tt) => {{
        use cpython::{PyErr, Python, PythonObject};

        let gil = Python::acquire_gil();
        let py = gil.python();
        let result: Result<$to, PyErr> = $input.into_object().extract(py);

        result
    }};
}

/// Convert a native type value to Python type
#[macro_export]
macro_rules! py_from {
    ($input:expr) => {{
        use cpython::Python;

        let gil = Python::acquire_gil();
        let py = gil.python();
        $input.to_py_object(py)
    }};
    ($input:expr, $to:ty) => {{
        use cpython::{ObjectProtocol, Python};

        let gil = Python::acquire_gil();
        let py = gil.python();
        let result: $to = $input.to_py_object(py);

        result
    }};
}

/// Creates a RefCell'ed Option of PyObject
#[macro_export]
macro_rules! py_to_shared {
    ($py_obj:expr) => {{
        use std::cell::RefCell;
        RefCell::new(Some($py_obj))
    }};
}

/// Calls a RefCell'ed Option of PyObject
#[macro_export]
macro_rules! py_call {
    ($py_obj:expr) => {{
        py_call!($py_obj, (), None)
    }};
    ($py_obj:expr, $args:expr) => {{
        py_call!($py_obj, $args, None)
    }};
    ($py_obj:expr, $args:expr, $kwargs:expr) => {{
        use cpython::Python;

        if let Some(ref callable) = *$py_obj.borrow() {
            let gil = Python::acquire_gil();
            let py = gil.python();

            try!(callable.call(py, $args, $kwargs))
        }
    }};
}

/// Calls a method of a RefCell'ed Option of PyObject
macro_rules! py_call_method {
    ($py_obj:expr, $method:expr) => {{
        py_call_method!($py_obj, $method, (), None)
    }};
    ($py_obj:expr, $method:expr, $args:expr) => {{
        py_call_method!($py_obj, $method, $args, None)
    }};
    ($py_obj:expr, $method:expr, $args:expr, $kwargs:expr) => {{
        use cpython::Python;

        if let Some(ref obj) = *$py_obj.lock().unwrap() {
            let gil = Python::acquire_gil();
            let py = gil.python();

            match obj.getattr(py, $method) {
                Ok(method) => {
                    if let Err(_) = method.call(py, $args, $kwargs) {
                        eprintln!("Core: cannot call a Python method");
                    }
                }
                Err(_) => {}
            };
        }
    }};
}

//
// Helper functions
//

pub fn host_port(address: &SocketAddr) -> (String, u16) {
    match address {
        SocketAddr::V4(a) => {
            let host = format!("{}", a.ip());
            let port: u16 = a.port();
            (host, port)
        }
        SocketAddr::V6(a) => {
            let host = format!("{}", a.ip());
            let port: u16 = a.port();
            (host, port)
        }
    }
}

pub fn to_socket_address(py_host: PyString, py_port: PyLong) -> Result<SocketAddr, ModuleError> {
    let host = py_extract!(py_host, String)?;
    let port = py_extract!(py_port, u16)?;
    let address: SocketAddr = socket_address(&host, port)?;

    Ok(address)
}

pub fn from_socket_address(address: SocketAddr) -> (PyString, PyInt) {
    let (host, port) = host_port(&address);
    let py_host: PyString = py_from!(host);
    let py_port: PyInt = py_from!(port);

    (py_host, py_port)
}

//
// Tests
//

#[cfg(test)]
mod tests {
    use cpython::{PyLong, Python, ToPyObject};

    #[test]
    fn extract() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_long: PyLong = 5_i32.to_py_object(py);

        assert_eq!(5_u16, py_extract!(py_long, u16).unwrap());
    }
}
