macro_rules! variants {
    ($(
        $(#[$attrs:meta])*
        $constructor:ident -> $variant:ident $({
            $($field:ident : $field_ty:ty),*
        })?
    );*) => {
        struct Private;

        #[allow(private_interfaces)]
        pub enum PhonicError {$(
            $(#[$attrs])*
            $variant {
                _private: Private,

                #[cfg(debug_assertions)]
                location: &'static std::panic::Location<'static>,

                $($($field : $field_ty),*)?
            }
        ),*}

        impl PhonicError {
            $(
                $(#[$attrs])*
                #[track_caller]
                pub fn $constructor(
                    $($($field : $field_ty),*)?
                ) -> Self {
                    Self::$variant {
                        _private: Private,

                        #[cfg(debug_assertions)]
                        location: std::panic::Location::caller(),

                        $($($field),*)?
                    }
                }
            )*

            #[cfg(debug_assertions)]
            fn location(&self) -> &'static std::panic::Location<'static> {
                match self {
                    $(Self::$variant { location, .. } => location),*
                }
            }
        }
    };
}

variants! {
    /// The operation is supported by the receivers interface, but is unsupported due to
    /// its implementation.
    unsupported -> Unsupported;

    /// A provided argument is invalid.
    invalid_input -> InvalidInput;

    /// The receiver is in an invalid state.
    invalid_state -> InvalidState;

    /// The operation requires data that was determined to be invalid.
    invalid_data -> InvalidData;

    /// The receiver is missing data required to complete the operation.
    missing_data -> MissingData;

    /// The parameters of the input did not match the parameters of the receiver.
    param_mismatch -> ParamMismatch;

    /// The requested entity could not be found.
    not_found -> NotFound;

    /// The operation would require the receiver to access data outside of its defined
    /// bounds.
    out_of_bounds -> OutOfBounds;

    /// The receiver is not ready to perform the requested operation.
    not_ready -> NotReady;

    /// The operation was interrupted and can be retried.
    interrupted -> Interrupted;

    /// The receivers underlying source or destination was disconnected.
    terminated -> Terminated;

    /// An io operation returned an error that could not be represented by another variant of
    /// this enum
    io -> Io { error: std::io::Error }
}

pub type PhonicResult<T> = Result<T, PhonicError>;

impl std::error::Error for PhonicError {}

impl std::fmt::Display for PhonicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported { .. } => write!(f, "unsupported"),
            Self::InvalidInput { .. } => write!(f, "invalid input"),
            Self::InvalidState { .. } => write!(f, "invalid state"),
            Self::InvalidData { .. } => write!(f, "invalid data"),
            Self::MissingData { .. } => write!(f, "missing data"),
            Self::ParamMismatch { .. } => write!(f, "parameter mismatch"),
            Self::NotFound { .. } => write!(f, "not found"),
            Self::OutOfBounds { .. } => write!(f, "out of bounds"),
            Self::NotReady { .. } => write!(f, "not ready"),
            Self::Interrupted { .. } => write!(f, "interrupted"),
            Self::Terminated { .. } => write!(f, "terminated"),
            Self::Io { error, .. } => write!(f, "io error: {error}"),
        }
    }
}

impl std::fmt::Debug for PhonicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "phonic error: \"{self}\"")?;

        #[cfg(debug_assertions)]
        write!(f, " at {}", self.location())?;

        Ok(())
    }
}

impl From<std::convert::Infallible> for PhonicError {
    fn from(error: std::convert::Infallible) -> Self {
        match error {}
    }
}

impl From<std::io::Error> for PhonicError {
    #[track_caller]
    fn from(error: std::io::Error) -> Self {
        use std::io::ErrorKind;

        match error.kind() {
            ErrorKind::Unsupported => Self::unsupported(),
            ErrorKind::InvalidInput => Self::invalid_input(),
            ErrorKind::InvalidData => Self::invalid_data(),
            ErrorKind::NotFound => Self::not_found(),
            ErrorKind::Interrupted => Self::interrupted(),
            ErrorKind::WouldBlock => Self::not_ready(),
            _ => Self::io(error),
        }
    }
}

impl From<PhonicError> for std::io::Error {
    fn from(error: PhonicError) -> Self {
        use std::io::ErrorKind;

        let kind = match error {
            PhonicError::Unsupported { .. } => ErrorKind::Unsupported,
            PhonicError::InvalidInput { .. } | PhonicError::ParamMismatch { .. } => {
                ErrorKind::InvalidInput
            }
            PhonicError::InvalidState { .. }
            | PhonicError::InvalidData { .. }
            | PhonicError::MissingData { .. } => ErrorKind::InvalidData,
            PhonicError::NotFound { .. } => ErrorKind::NotFound,
            PhonicError::OutOfBounds { .. } => ErrorKind::UnexpectedEof,
            PhonicError::NotReady { .. } => ErrorKind::WouldBlock,
            PhonicError::Interrupted { .. } => ErrorKind::Interrupted,
            PhonicError::Terminated { .. } => ErrorKind::Other,
            PhonicError::Io { error, .. } => return error,
        };

        let msg = format!("phonic error: {}", error);
        Self::new(kind, msg)
    }
}

impl<T> From<std::sync::TryLockError<T>> for PhonicError {
    #[track_caller]
    fn from(error: std::sync::TryLockError<T>) -> Self {
        use std::sync::TryLockError;

        match error {
            TryLockError::WouldBlock => PhonicError::not_ready(),
            TryLockError::Poisoned(e) => e.into(),
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for PhonicError {
    #[track_caller]
    fn from(_error: std::sync::PoisonError<T>) -> Self {
        Self::invalid_state()
    }
}
