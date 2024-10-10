use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct DeserializeError {
    pub message: String,
}

impl Error for DeserializeError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to deserialize message: {}", self.message)
    }
}