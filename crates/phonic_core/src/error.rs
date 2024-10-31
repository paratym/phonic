use std::{convert::Infallible, error::Error, fmt::Display, io};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PhonicError {
    Unreachable,
    InvalidData,
    MissingData,
    Unsupported,
    SignalMismatch,
    NotFound,
    NotReady,
    OutOfBounds,
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
            Self::OutOfBounds => write!(f, "out of bounds"),
            Self::Interrupted => write!(f, "interrupted"),
            Self::IoError => write!(f, "io error"),
            Self::Other => write!(f, "other"),
        }
    }
}

impl From<Infallible> for PhonicError {
    fn from(error: Infallible) -> Self {
        match error {}
    }
}

impl From<io::Error> for PhonicError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::InvalidData => Self::InvalidData,
            io::ErrorKind::Unsupported => Self::Unsupported,
            io::ErrorKind::NotFound => Self::NotFound,
            io::ErrorKind::WouldBlock => Self::NotReady,
            io::ErrorKind::UnexpectedEof => Self::OutOfBounds,
            io::ErrorKind::Interrupted => Self::Interrupted,
            _ => Self::IoError,
        }
    }
}

impl From<PhonicError> for io::Error {
    fn from(error: PhonicError) -> Self {
        let kind = match error {
            PhonicError::Unreachable => io::ErrorKind::Other,
            PhonicError::InvalidData => io::ErrorKind::InvalidData,
            PhonicError::MissingData => io::ErrorKind::InvalidData,
            PhonicError::Unsupported => io::ErrorKind::Unsupported,
            PhonicError::SignalMismatch => io::ErrorKind::InvalidData,
            PhonicError::NotFound => io::ErrorKind::NotFound,
            PhonicError::NotReady => io::ErrorKind::WouldBlock,
            PhonicError::OutOfBounds => io::ErrorKind::UnexpectedEof,
            PhonicError::Interrupted => io::ErrorKind::Interrupted,
            PhonicError::IoError => io::ErrorKind::Other,
            PhonicError::Other => io::ErrorKind::Other,
        };

        Self::new(kind, error.to_string())
    }
}
