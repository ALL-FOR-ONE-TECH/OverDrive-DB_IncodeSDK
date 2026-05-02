use std::fmt;

pub type Result<T> = std::result::Result<T, SdkError>;

#[derive(Debug)]
pub struct SdkError(pub String);

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[overdrive-sdk] {}", self.0)
    }
}

impl std::error::Error for SdkError {}

impl From<String> for SdkError {
    fn from(s: String) -> Self { SdkError(s) }
}

impl From<&str> for SdkError {
    fn from(s: &str) -> Self { SdkError(s.to_string()) }
}

impl From<serde_json::Error> for SdkError {
    fn from(e: serde_json::Error) -> Self { SdkError(e.to_string()) }
}
