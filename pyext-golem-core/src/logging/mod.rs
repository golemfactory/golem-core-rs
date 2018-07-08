use cpython::{PyInt, Python, ToPyObject};
use std::convert;

use net::error::ErrorSeverity;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}

impl convert::From<ErrorSeverity> for LogLevel {
    fn from(severity: ErrorSeverity) -> Self {
        match severity {
            ErrorSeverity::Other => LogLevel::Info,
            ErrorSeverity::Low => LogLevel::Debug,
            ErrorSeverity::MediumLow => LogLevel::Info,
            ErrorSeverity::Medium => LogLevel::Warning,
            ErrorSeverity::MediumHigh => LogLevel::Error,
            ErrorSeverity::High => LogLevel::Error,
        }
    }
}

impl ToPyObject for LogLevel {
    type ObjectType = PyInt;

    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        (*self as u8).to_py_object(py)
    }
}
