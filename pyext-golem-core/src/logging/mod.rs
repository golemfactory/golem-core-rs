use cpython::{PyInt, Python, ToPyObject};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}

impl ToPyObject for LogLevel {
    type ObjectType = PyInt;

    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        (*self as u8).to_py_object(py)
    }
}
