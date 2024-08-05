use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct ControlValueError {
    pub message: String,
}

impl ControlValueError {
    pub fn new(message: String) -> ControlValueError {
        return ControlValueError { message };
    }
}

impl Display for ControlValueError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

