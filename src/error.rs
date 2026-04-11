use std::fmt;

pub enum AppError {
    /// Pre-formatted error message. Exit code: 1
    Message(String),
    /// User cancelled. Exit code: 0
    UserAborted,
    /// Non-zero exit without additional message. Exit code: 1
    SilentFailure,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Message(msg) => write!(f, "{}", msg),
            AppError::UserAborted | AppError::SilentFailure => Ok(()),
        }
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}
