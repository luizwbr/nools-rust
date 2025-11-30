//! Error types for the nools rules engine

use thiserror::Error;

/// Result type alias for nools operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in the rules engine
#[derive(Error, Debug)]
pub enum Error {
    /// Error during rule compilation
    #[error("Compilation error: {0}")]
    Compilation(String),

    /// Error during rule execution
    #[error("Execution error: {0}")]
    Execution(String),

    /// Error in pattern matching
    #[error("Pattern matching error: {0}")]
    PatternMatch(String),

    /// Fact not found in working memory
    #[error("Fact not found: {0}")]
    FactNotFound(String),

    /// Rule not found
    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    /// Invalid constraint
    #[error("Invalid constraint: {0}")]
    InvalidConstraint(String),

    /// Agenda group not found
    #[error("Agenda group not found: {0}")]
    AgendaGroupNotFound(String),

    /// Generic error with custom message
    #[error("{0}")]
    Custom(String),
}

impl Error {
    /// Create a custom error
    pub fn custom(msg: impl Into<String>) -> Self {
        Error::Custom(msg.into())
    }
}
