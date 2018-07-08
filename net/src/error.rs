use std::net::AddrParseError;
use std::sync::mpsc::{RecvError, RecvTimeoutError};
use std::{convert, error, fmt, io};

use actix::MailboxError;
use codec::error::CodecError;

#[derive(Debug, Copy, Clone)]
pub enum ErrorKind {
    Io,
    Network,
    Mailbox,
    Other,
}

#[derive(Debug, Copy, Clone)]
pub enum ErrorSeverity {
    Low,
    MediumLow,
    Medium,
    MediumHigh,
    High,
    Other,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub severity: ErrorSeverity,
    pub message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, severity: ErrorSeverity, message: &str) -> Self {
        let message = String::from(message);
        Error {
            kind,
            severity,
            message,
        }
    }
}

unsafe impl Send for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error ({:?}, severity: {:?}): {}", self.kind, self.severity, &self.message[..])
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.message[..]
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl convert::From<AddrParseError> for Error {
    fn from(e: AddrParseError) -> Self {
        Error::new(
            ErrorKind::Other,
            ErrorSeverity::Low,
            &format!("address error: {}", e),
        )
    }
}

impl convert::From<CodecError> for Error {
    fn from(e: CodecError) -> Self {
        Error::new(
            ErrorKind::Other,
            ErrorSeverity::Medium,
            &format!("codec error: {}", e),
        )
    }
}

impl convert::From<MailboxError> for Error {
    fn from(e: MailboxError) -> Self {
        Error::new(
            ErrorKind::Mailbox,
            ErrorSeverity::MediumHigh,
            &format!("actor error: {:?}", e),
        )
    }
}

impl convert::From<RecvError> for Error {
    fn from(e: RecvError) -> Self {
        Error::new(
            ErrorKind::Io,
            ErrorSeverity::MediumLow,
            &format!("rx error: {}", e),
        )
    }
}

impl convert::From<RecvTimeoutError> for Error {
    fn from(e: RecvTimeoutError) -> Self {
        Error::new(
            ErrorKind::Io,
            ErrorSeverity::Low,
            &format!("rx timeout: {}", e),
        )
    }
}

impl convert::From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::new(
            ErrorKind::Io,
            ErrorSeverity::MediumLow,
            &format!("io error: {}", e),
        )
    }
}

impl convert::From<Box<error::Error>> for Error {
    fn from(e: Box<error::Error>) -> Self {
        Error::new(
            ErrorKind::Other,
            ErrorSeverity::Other,
            &format!("error: {}", e),
        )
    }
}

impl convert::From<()> for Error {
    fn from(_: ()) -> Self {
        Error::new(ErrorKind::Other, ErrorSeverity::Other, "unknown error")
    }
}
