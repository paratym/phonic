#[derive(Debug)]
pub enum PhonicError {
    /// The operation is supported by the receivers interface, but is unsupported due to
    /// its implementation.
    Unsupported,

    /// A provided argument is invalid.
    InvalidInput,

    /// The receiver is in an invalid state.
    InvalidState,

    /// The operation requires data that was determined to be invalid.
    InvalidData,

    /// The receiver is missing data required to complete the operation.
    MissingData,

    /// The parameters of the input did not match the parameters of the receiver.
    ParamMismatch,

    /// The requested entity could not be found.
    NotFound,

    /// The operation would require the receiver to access data outside of its defined
    /// bounds.
    OutOfBounds,

    /// The receiver is not ready to perform the requested operation.
    NotReady,

    /// The operation was interrupted and can be retried.
    Interrupted,

    /// The receivers underlying source or destination was disconnected.
    Terminated,

    /// An io operation returned an error that could not be represented by another variant of
    /// this enum
    #[cfg(feature = "io")]
    Io(std::io::Error),
}

pub type PhonicResult<T> = Result<T, PhonicError>;

impl std::error::Error for PhonicError {}

impl std::fmt::Display for PhonicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => write!(f, "unsupported"),
            Self::InvalidInput => write!(f, "invalid input"),
            Self::InvalidState => write!(f, "invalid state"),
            Self::InvalidData => write!(f, "invalid data"),
            Self::MissingData => write!(f, "missing data"),
            Self::ParamMismatch => write!(f, "parameter mismatch"),
            Self::NotFound => write!(f, "not found"),
            Self::OutOfBounds => write!(f, "out of bounds"),
            Self::NotReady => write!(f, "not ready"),
            Self::Interrupted => write!(f, "interrupted"),
            Self::Terminated => write!(f, "terminated"),

            #[cfg(feature = "io")]
            Self::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl From<std::convert::Infallible> for PhonicError {
    fn from(error: std::convert::Infallible) -> Self {
        match error {}
    }
}

#[cfg(feature = "io")]
impl From<std::io::Error> for PhonicError {
    fn from(error: std::io::Error) -> Self {
        use std::io::ErrorKind;

        match error.kind() {
            ErrorKind::Unsupported => Self::Unsupported,
            ErrorKind::InvalidInput => Self::InvalidInput,
            ErrorKind::NotFound => Self::NotFound,
            ErrorKind::Interrupted => Self::Interrupted,
            ErrorKind::WriteZero | ErrorKind::WouldBlock => Self::NotReady,
            _ => Self::Io(error),
        }
    }
}

impl<T> From<std::sync::TryLockError<T>> for PhonicError {
    fn from(error: std::sync::TryLockError<T>) -> Self {
        use std::sync::TryLockError;

        match error {
            TryLockError::WouldBlock => PhonicError::NotReady,
            TryLockError::Poisoned(e) => e.into(),
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for PhonicError {
    fn from(_error: std::sync::PoisonError<T>) -> Self {
        Self::InvalidState
    }
}
