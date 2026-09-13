//! .test-guard.json baseline schema definitions

use std::collections::HashSet;

use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Programming languages supported for test extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestLanguage {
    #[serde(rename = "typescript")]
    TypeScript,
    #[serde(rename = "javascript")]
    JavaScript,
    Rust,
    Python,
    Go,
}

/// Verification tier for test execution scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestTier {
    Tier1Unit,
    Tier2Integration,
    Tier3E2e,
}

/// Baseline metadata for a single test item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestItemBaseline {
    pub id: String,
    pub name: String,
    pub fingerprint: String,
    pub assertion_count: usize,
}

/// Baseline metadata for a test file containing tests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestFileBaseline {
    pub path: String,
    pub language: TestLanguage,
    pub tier: TestTier,
    pub blake3_hash: String,
    pub tests: Vec<TestItemBaseline>,
}

/// Configuration settings for baseline assertions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineConfig {
    pub min_assertions_per_test: usize,
}

impl Default for BaselineConfig {
    fn default() -> Self {
        Self {
            min_assertions_per_test: 1,
        }
    }
}

/// Monotonic counters tracking test suites and files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RatchetState {
    pub total_tests: usize,
    pub files_count: usize,
}

/// Cryptographic signature block authorizing baseline state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureBlock {
    pub public_key: String,
    pub signature: String,
    pub algorithm: String,
}

/// Root manifest for `.test-guard.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineManifest {
    pub schema_version: u32,
    pub config: BaselineConfig,
    pub ratchet: RatchetState,
    pub files: Vec<TestFileBaseline>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<SignatureBlock>,
}

/// Errors occurring during schema validation or cryptographic operations.
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("Count mismatch: expected {expected}, actual {actual}")]
    CountMismatch { expected: usize, actual: usize },

    #[error("Assertion floor violation for {test_id}: {count} < {min}")]
    AssertionFloorViolation {
        test_id: String,
        count: usize,
        min: usize,
    },

    #[error("Duplicate test ID: {0}")]
    DuplicateTestId(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Invalid hash in {field}: {details}")]
    InvalidHash { field: String, details: String },

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Missing signature")]
    MissingSignature,

    #[error("Canonicalization error: {0}")]
    Canonicalization(#[from] crate::canonical::CanonicalError),

    #[error("Crypto error: {0}")]
    Crypto(#[from] crate::crypto::CryptoError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

fn is_hex_len(s: &str, expected_len: usize) -> bool {
    s.len() == expected_len && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_valid_path(p: &str) -> bool {
    !p.is_empty()
        && !p.starts_with('/')
        && !p.contains('\\')
        && !p.contains("..")
        && !p.split('/').any(|seg| seg == "." || seg == "..")
}

impl BaselineManifest {
    /// Validates internal consistency and integrity invariants of the manifest.
    pub fn validate(&self) -> Result<(), SchemaError> {
        if self.ratchet.files_count != self.files.len() {
            return Err(SchemaError::CountMismatch {
                expected: self.files.len(),
                actual: self.ratchet.files_count,
            });
        }
        let actual_tests: usize = self.files.iter().map(|f| f.tests.len()).sum();
        if self.ratchet.total_tests != actual_tests {
            return Err(SchemaError::CountMismatch {
                expected: actual_tests,
                actual: self.ratchet.total_tests,
            });
        }
        let mut seen_ids = HashSet::new();
        for (i, file) in self.files.iter().enumerate() {
            if !is_valid_path(&file.path) {
                return Err(SchemaError::InvalidPath(file.path.clone()));
            }
            if !is_hex_len(&file.blake3_hash, 64) {
                return Err(SchemaError::InvalidHash {
                    field: format!("files[{i}].blake3_hash"),
                    details: "must be 64-char hex".to_string(),
                });
            }
            for test in &file.tests {
                if !seen_ids.insert(test.id.as_str()) {
                    return Err(SchemaError::DuplicateTestId(test.id.clone()));
                }
                if test.assertion_count < self.config.min_assertions_per_test {
                    return Err(SchemaError::AssertionFloorViolation {
                        test_id: test.id.clone(),
                        count: test.assertion_count,
                        min: self.config.min_assertions_per_test,
                    });
                }
                if !is_hex_len(&test.fingerprint, 64) {
                    return Err(SchemaError::InvalidHash {
                        field: format!("tests[{}].fingerprint", test.id),
                        details: "must be 64-char hex".to_string(),
                    });
                }
            }
        }
        if let Some(sig) = &self.signature {
            if !is_hex_len(&sig.public_key, 64) {
                return Err(SchemaError::InvalidSignature(
                    "public_key must be 64-char hex".to_string(),
                ));
            }
            if !is_hex_len(&sig.signature, 128) {
                return Err(SchemaError::InvalidSignature(
                    "signature must be 128-char hex".to_string(),
                ));
            }
            if sig.algorithm != "ed25519" {
                return Err(SchemaError::InvalidSignature(
                    "algorithm must be ed25519".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Computes RFC 8785 canonical bytes for signing with signature block omitted.
    pub fn signing_bytes(&self) -> Result<Vec<u8>, SchemaError> {
        let mut unsigned = self.clone();
        unsigned.signature = None;
        crate::canonical::to_canonical_vec(&unsigned).map_err(SchemaError::from)
    }

    /// Computes deterministic BLAKE3 digest of the canonicalized manifest.
    pub fn digest(&self) -> Result<String, SchemaError> {
        let bytes = crate::canonical::to_canonical_vec(self)?;
        Ok(crate::crypto::blake3_hash_hex(&bytes))
    }

    /// Signs manifest with Ed25519 key and attaches the SignatureBlock.
    pub fn sign(&mut self, signing_key: &SigningKey) -> Result<(), SchemaError> {
        let bytes = self.signing_bytes()?;
        let sig_hex = crate::crypto::sign_hex(signing_key, &bytes);
        let pub_hex = crate::crypto::verifying_key_to_hex(&signing_key.verifying_key());
        self.signature = Some(SignatureBlock {
            public_key: pub_hex,
            signature: sig_hex,
            algorithm: "ed25519".to_string(),
        });
        Ok(())
    }

    /// Verifies the embedded Ed25519 signature over unsigned canonical bytes.
    pub fn verify_signature(&self) -> Result<bool, SchemaError> {
        let sig = match &self.signature {
            Some(s) => s,
            None => return Ok(false),
        };
        let bytes = self.signing_bytes()?;
        Ok(crate::crypto::verify_hex(&sig.public_key, &bytes, &sig.signature).is_ok())
    }
}
