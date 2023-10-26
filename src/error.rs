use std::{error::Error, fmt::Display, io};

#[derive(Clone, Debug)]
pub enum SyphonError {
    InvalidInput,
    InvalidData,
    Unsupported,
    SignalMismatch,
    NotFound,
    NotReady,
    EndOfStream,
    Interrupted,
    IoError,
    Other,
}

impl Error for SyphonError {}

impl Display for SyphonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput => write!(f, "invalid input"),
            Self::InvalidData => write!(f, "invalid data"),
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

impl From<io::Error> for SyphonError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::InvalidInput => Self::InvalidInput,
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

impl From<SyphonError> for io::Error {
    fn from(e: SyphonError) -> Self {
        let kind = match e {
            SyphonError::InvalidInput => io::ErrorKind::InvalidInput,
            SyphonError::InvalidData => io::ErrorKind::InvalidData,
            SyphonError::Unsupported => io::ErrorKind::Unsupported,
            SyphonError::SignalMismatch => io::ErrorKind::InvalidData,
            SyphonError::NotFound => io::ErrorKind::NotFound,
            SyphonError::NotReady => io::ErrorKind::WouldBlock,
            SyphonError::EndOfStream => io::ErrorKind::UnexpectedEof,
            SyphonError::Interrupted => io::ErrorKind::Interrupted,
            SyphonError::IoError => io::ErrorKind::Other,
            SyphonError::Other => io::ErrorKind::Other,
        };

        Self::new(kind, e.to_string())
    }
}
