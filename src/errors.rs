use thiserror::Error;

/// Errors that can occur when using unificial-api.
///
/// This error type is used across all site implementations (e.g., AO3)
/// and covers common failure modes such as HTML selector issues,
/// regex compilation failures, serialization problems, I/O errors,
/// and network errors.
#[derive(Error, Debug)]
pub enum UnificialError {
    #[error("Selector error: {0}")]
    SelectorError(String),
    #[error("regex error: {0}")]
    RegexError(String),
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("{0}")]
    GenericError(String),
}
