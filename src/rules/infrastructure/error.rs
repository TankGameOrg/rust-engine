use std::fmt::Display;
use core::error::Error;

#[derive(Debug)]
pub enum RuleError {
    /// The requested attribute was not found on this Entity
    AttributeNotFound {
        name: &'static str,
    },
    /// A catch all error that has a string error message
    Generic(String),
}

impl Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AttributeNotFound { name } => {
                f.write_fmt(format_args!("Could not find attribute '{}'", name))?;
            },
            Self::Generic(message) => {
                f.write_str(&message)?;
            },
        }

        Ok(())
    }
}

impl Error for RuleError {}
