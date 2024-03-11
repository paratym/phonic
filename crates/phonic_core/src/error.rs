use std::{error::Error, fmt::Display, io};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PhonicError {
    Unreachable,
    InvalidData,
    MissingData,
    Unsupported,
    SignalMismatch,
    NotFound,
    NotReady,
    EndOfStream,
    Interrupted,
    IoError,
    Other,
}

impl Error for PhonicError {}

impl Display for PhonicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreachable => write!(f, "unreachable"),
            Self::InvalidData => write!(f, "invalid data"),
            Self::MissingData => write!(f, "missing data"),
            Self::Unsupported => write!(f, "unsupported"),
            Self::SignalMismatch => write!(f, "signal mismatch"),
            Self::NotFound => write!(f, "not found"),
            Self::NotReady => write!(f, "not ready"),
            Self::EndOfStream => write!(f, "end of stream"),
            Self::Interrupted => write!(f, "interrupted"),
            Self::IoError => write!(f, "io error"),
            Self::Other => write!(f, "other"),
        }
    }
}

impl From<io::Error> for PhonicError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::InvalidData => Self::InvalidData,
            io::ErrorKind::Unsupported => Self::Unsupported,
            io::ErrorKind::NotFound => Self::NotFound,
            io::ErrorKind::WouldBlock => Self::NotReady,
            io::ErrorKind::UnexpectedEof => Self::EndOfStream,
            io::ErrorKind::Interrupted => Self::Interrupted,
            _ => Self::IoError,
        }
    }
}

impl From<PhonicError> for io::Error {
    fn from(e: PhonicError) -> Self {
        let kind = match e {
            PhonicError::Unreachable => io::ErrorKind::Other,
            PhonicError::InvalidData => io::ErrorKind::InvalidData,
            PhonicError::MissingData => io::ErrorKind::InvalidData,
            PhonicError::Unsupported => io::ErrorKind::Unsupported,
            PhonicError::SignalMismatch => io::ErrorKind::InvalidData,
            PhonicError::NotFound => io::ErrorKind::NotFound,
            PhonicError::NotReady => io::ErrorKind::WouldBlock,
            PhonicError::EndOfStream => io::ErrorKind::UnexpectedEof,
            PhonicError::Interrupted => io::ErrorKind::Interrupted,
            PhonicError::IoError => io::ErrorKind::Other,
            PhonicError::Other => io::ErrorKind::Other,
        };

        Self::new(kind, e.to_string())
    }
}
