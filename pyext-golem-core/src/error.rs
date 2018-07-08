use std::{convert, error, fmt};
use std::sync::mpsc::{RecvError, RecvTimeoutError};
use std::net::AddrParseError;

use actix::MailboxError;
use cpython::{PyErr, PyString, Python};

use net::error::{Error, ErrorKind, ErrorSeverity};
use CoreError;

#[derive(Debug)]
pub struct ModuleError {
    error: Error,
    py_err: Option<PyErr>,
}

impl ModuleError {
    pub fn new(error: Error, py_err: Option<PyErr>) -> Self {
        ModuleError { error, py_err }
    }

    pub fn not_running() -> Self {
        let error = Error::new(ErrorKind::Network, ErrorSeverity::High, "not running");
        ModuleError::from(error)
    }
}

unsafe impl Send for ModuleError {}

impl fmt::Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl error::Error for ModuleError {
    fn description(&self) -> &str {
        self.error.description()
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl convert::From<Error> for ModuleError {
    fn from(e: Error) -> Self {
        ModuleError::new(e, None)
    }
}

impl convert::From<PyErr> for ModuleError {
    fn from(e: PyErr) -> Self {
        let error = Error::new(ErrorKind::Other, ErrorSeverity::High, "python error");
        ModuleError::new(error, Some(e))
    }
}

impl convert::Into<PyErr> for ModuleError {
    fn into(self) -> PyErr {
        match self.py_err {
            Some(e) => e,
            None => {
                let gil = Python::acquire_gil();
                let py = gil.python();

                let msg = <Self as error::Error>::description(&self);
                let py_msg = PyString::new(py, &msg[..]);

                PyErr::new::<CoreError, PyString>(py, py_msg)
            }
        }
    }
}

macro_rules! impl_from {
    ($($from:ty),+) => {
        $(impl From<$from> for ModuleError {
            fn from(e: $from) -> Self {
                Self::from(Error::from(e))
            }
        })*
    }
}

impl_from!(
    AddrParseError,
    Box<error::Error>,
    MailboxError,
    RecvError,
    RecvTimeoutError
);
