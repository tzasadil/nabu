use std::fmt;

/// Every error carries a fix, so no message ends without a next step for the user.
#[derive(Debug)]
pub struct Error {
    cause: String,
    fix: String,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(cause: impl Into<String>, fix: impl Into<String>) -> Self {
        Self {
            cause: cause.into(),
            fix: fix.into(),
        }
    }
}

impl Error {
    /// The first line of the cause, for one-line summaries such as `list` rows.
    pub fn summary(&self) -> &str {
        self.cause.lines().next().unwrap_or_default()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {}\nfix: {}", self.cause, self.fix)
    }
}
