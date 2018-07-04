use std::net::AddrParseError;
use std::sync::mpsc::RecvError;
use std::{convert, error, fmt, io};

use actix::MailboxError;
use cpython::{PyErr, PyString, Python};

use CoreError;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Io,
    Network,
    Mailbox,
    Python,
    Other,
}

#[derive(Debug)]
pub struct ModuleError {
    kind: ErrorKind,
    message: String,
    py_err: Option<PyErr>,
}

impl ModuleError {
    pub fn new(kind: ErrorKind, message: &str, py_err: Option<PyErr>) -> Self {
        ModuleError {
            kind,
            message: String::from(message),
            py_err,
        }
    }
}

unsafe impl Send for ModuleError {}

impl fmt::Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "module error ({:?}): {}", self.kind, &self.message[..])
    }
}

impl error::Error for ModuleError {
    fn description(&self) -> &str {
        &self.message[..]
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl convert::From<io::Error> for ModuleError {
    fn from(e: io::Error) -> Self {
        ModuleError::new(ErrorKind::Io, &format!("io error: {}", e), None)
    }
}

impl convert::From<Box<error::Error>> for ModuleError {
    fn from(e: Box<error::Error>) -> Self {
        ModuleError::new(ErrorKind::Other, &format!("error: {}", e), None)
    }
}

impl convert::From<AddrParseError> for ModuleError {
    fn from(e: AddrParseError) -> Self {
        ModuleError::new(ErrorKind::Other, &format!("address error: {}", e), None)
    }
}

impl convert::From<RecvError> for ModuleError {
    fn from(e: RecvError) -> Self {
        ModuleError::new(ErrorKind::Network, &format!("rx error: {}", e), None)
    }
}

impl convert::From<MailboxError> for ModuleError {
    fn from(e: MailboxError) -> Self {
        ModuleError::new(ErrorKind::Mailbox, &format!("actor mailbox error: {:?}", e), None)
    }
}

impl convert::From<()> for ModuleError {
    fn from(_: ()) -> Self {
        ModuleError::new(ErrorKind::Other, "unknown error", None)
    }
}

impl convert::From<PyErr> for ModuleError {
    fn from(e: PyErr) -> Self {
        ModuleError::new(ErrorKind::Python, "python error", Some(e))
    }
}

impl convert::Into<PyErr> for ModuleError {
    fn into(self) -> PyErr {
        match self.py_err {
            Some(e) => e,
            None => {
                let gil = Python::acquire_gil();
                let py = gil.python();

                let msg = format!("{:?} {}", self.kind, self.message);
                let py_msg = PyString::new(py, &msg[..]);

                PyErr::new::<CoreError, PyString>(py, py_msg)
            }
        }
    }
}
