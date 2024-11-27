use core::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct BasicError {
    message: String,
}

impl BasicError {
    pub fn new(message: String) -> Box<dyn Error> {
        return Box::new(BasicError { message });
    }
}

impl Display for BasicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for BasicError {}

#[macro_export]
macro_rules! basic_error {
    ($($token:tt)+) => {
        $crate::rules::infrastructure::BasicError::new(format!($($token)+))
    };
}
