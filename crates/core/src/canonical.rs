//! RFC 8785 JSON Canonicalization Scheme (JCS)
//!
//! Provides deterministic JSON canonicalization compliant with RFC 8785,
//! ensuring consistent representation for cryptographic hashing and verification.

use serde::Serialize;
use thiserror::Error;

/// Errors that can occur during RFC 8785 canonicalization.
#[derive(Debug, Error)]
pub enum CanonicalError {
    /// JSON serialization, deserialization, or syntax error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Canonicalization error.
    #[error("Canonicalization error: {0}")]
    Canonicalization(String),
}

impl From<std::string::FromUtf8Error> for CanonicalError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        CanonicalError::Canonicalization(err.to_string())
    }
}

impl From<String> for CanonicalError {
    fn from(err: String) -> Self {
        CanonicalError::Canonicalization(err)
    }
}

impl From<&str> for CanonicalError {
    fn from(err: &str) -> Self {
        CanonicalError::Canonicalization(err.to_string())
    }
}

/// Serializes a value implementing `Serialize` into an RFC 8785 canonical JSON string.
pub fn to_canonical_string<T: Serialize>(value: &T) -> Result<String, CanonicalError> {
    let bytes = to_canonical_vec(value)?;
    String::from_utf8(bytes).map_err(CanonicalError::from)
}

/// Serializes a value implementing `Serialize` into an RFC 8785 canonical JSON byte vector.
pub fn to_canonical_vec<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError> {
    serde_json_canonicalizer::to_vec(value).map_err(CanonicalError::from)
}

/// Parses a JSON string and converts it to an RFC 8785 canonical JSON string.
pub fn canonicalize_json_str(json_str: &str) -> Result<String, CanonicalError> {
    let value: serde_json::Value = serde_json::from_str(json_str)?;
    canonicalize_value(&value)
}

/// Converts a `serde_json::Value` to an RFC 8785 canonical JSON string.
pub fn canonicalize_value(value: &serde_json::Value) -> Result<String, CanonicalError> {
    to_canonical_string(value)
}
