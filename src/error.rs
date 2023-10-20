use std::{error::Error, fmt::Display};

#[derive(Clone, Debug)]
pub enum SyphonError {
    Empty,
    NotReady,
    BadRequest,
    MalformedData,
    Unsupported,
    StreamMismatch,
    Other(String),
}

impl Error for SyphonError {}

impl Display for SyphonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "empty"),
            Self::NotReady => write!(f, "not ready"),
            Self::BadRequest => write!(f, "bad request"),
            Self::MalformedData => write!(f, "malformed data"),
            Self::Unsupported => write!(f, "unsupported"),
            Self::StreamMismatch => write!(f, "stream specs do not match"),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<std::io::Error> for SyphonError {
    fn from(e: std::io::Error) -> Self {
        Self::Other(e.to_string())
    }
}

impl From<SyphonError> for std::io::Error {
    fn from(e: SyphonError) -> Self {
        Self::new(std::io::ErrorKind::Other, e.to_string())
    }
}