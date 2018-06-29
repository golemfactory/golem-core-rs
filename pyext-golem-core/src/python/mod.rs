use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use cpython::{PyInt, PyLong, PyObject, PyString, Python, ToPyObject};

use error::*;
use net::socket_address;

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

/// Calls a method of a RefCell'ed Option of PyObject
macro_rules! py_call_method {
    ($py:expr, $py_obj:expr, $method:expr) => {{
        py_call_method!($py, $py_obj, $method, (), None)
    }};
    ($py:expr, $py_obj:expr, $method:expr, $args:expr) => {{
        py_call_method!($py, $py_obj, $method, $args, None)
    }};
    ($py:expr, $py_obj:expr, $method:expr, $args:expr, $kwargs:expr) => {{
        if let Some(ref obj) = *$py_obj.lock().unwrap() {
            match obj.getattr($py, $method) {
                Ok(method) => {
                    if let Err(_) = method.call($py, $args, $kwargs) {
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
    let gil = Python::acquire_gil();
    let py = gil.python();

    let host: String = py_extract!(py, py_host)?;
    let port: u16 = py_extract!(py, py_port)?;
    let address: SocketAddr = socket_address(&host, port)?;

    Ok(address)
}

pub fn from_socket_address(address: SocketAddr) -> (PyString, PyInt) {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let (host, port) = host_port(&address);
    let py_host: PyString = py_wrap!(py, host);
    let py_port: PyInt = py_wrap!(py, port);

    (py_host, py_port)
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
