use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct AocError {
    err: String,
}

impl AocError {
    pub fn new<S: AsRef<str>>(err: S) -> Self {
        AocError {
            err: err.as_ref().to_string(),
        }
    }
}

impl fmt::Display for AocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl error::Error for AocError {}

pub type AocResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub fn failure<T, S: AsRef<str>>(err: S) -> AocResult<T> {
    Err(Box::new(AocError::new(err.as_ref())))
}
