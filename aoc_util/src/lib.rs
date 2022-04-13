use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct AocError {
    err: String,
}
pub type AocResult<T> = std::result::Result<T, Box<dyn error::Error>>;

impl fmt::Display for AocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl error::Error for AocError {}

pub fn failure<T>(err: &str) -> AocResult<T> {
    Err(Box::new(AocError {
        err: err.to_string(),
    }))
}
