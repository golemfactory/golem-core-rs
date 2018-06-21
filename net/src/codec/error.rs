use std::{error, convert, fmt, io};
use bincode::ErrorKind;

#[derive(Debug, Clone)]
pub struct CodecError
{
    message: String,
}

impl CodecError
{
    pub fn new(message: &str) -> Self
    {
        CodecError{ message: String::from(message.clone()) }
    }
}


impl fmt::Display for CodecError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "message codec error: {}", &self.message[..])
    }
}


impl error::Error for CodecError
{
    fn description(&self) -> &str
    {
        &self.message[..]
    }

    fn cause(&self) -> Option<&error::Error>
    {
        None
    }
}


impl convert::From<io::Error> for CodecError
{
    fn from(e: io::Error) -> Self
    {
        CodecError::new(&format!("io error: {}", e))
    }
}

impl convert::From<Box<ErrorKind>> for CodecError {
    fn from(e: Box<ErrorKind>) -> Self
    {
        CodecError::new(&format!("{}", e))
    }
}

impl convert::From<()> for CodecError {
    fn from(_: ()) -> Self
    {
        CodecError::new("unknown error")
    }
}
