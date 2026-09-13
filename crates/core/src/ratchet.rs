//! Monotonic ratchet logic and subset invariant validation (Baseline ⊆ Current)

use std::collections::HashMap;

use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::crypto::CryptoError;
use crate::schema::{BaselineManifest, RatchetState, SchemaError, TestItemBaseline};

/// Options configuring monotonic ratchet evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RatchetOptions {
    pub allow_shrink: bool,
    pub trusted_keys: Vec<String>,
}

/// Delta representing changes in a modified test item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestItemDelta {
    pub id: String,
    pub old_fingerprint: String,
    pub new_fingerprint: String,
    pub old_assertion_count: usize,
    pub new_assertion_count: usize,
}

/// Diff summary between baseline and current test suites.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RatchetDiff {
    pub added: Vec<TestItemBaseline>,
    pub removed: Vec<TestItemBaseline>,
    pub modified: Vec<TestItemDelta>,
    pub unchanged: Vec<TestItemBaseline>,
    pub baseline_total: usize,
    pub current_total: usize,
}

/// Errors occurring during ratchet verification or baseline advancement.
#[derive(Debug, Error)]
pub enum RatchetError {
    #[error("Missing tests: {0:?}")]
    MissingTests(Vec<String>),

    #[error("Test count decreased: baseline {baseline}, current {current}")]
    TestCountDecreased { baseline: usize, current: usize },

    #[error("Shrink requires signature")]
    ShrinkRequiresSignature,

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Untrusted key: {0}")]
    UntrustedKey(String),

    #[error("Schema error: {0}")]
    Schema(#[from] SchemaError),

    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),
}

/// Verifies that the current manifest satisfies monotonic ratchet invariants relative to baseline.
pub fn verify_ratchet(
    baseline: &BaselineManifest,
    current: &BaselineManifest,
    options: &RatchetOptions,
) -> Result<RatchetDiff, RatchetError> {
    let mut baseline_map = HashMap::new();
    for file in &baseline.files {
        for test in &file.tests {
            baseline_map.insert(test.id.as_str(), test);
        }
    }

    let mut current_map = HashMap::new();
    for file in &current.files {
        for test in &file.tests {
            current_map.insert(test.id.as_str(), test);
        }
    }

    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut unchanged = Vec::new();

    for file in &current.files {
        for test in &file.tests {
            match baseline_map.get(test.id.as_str()) {
                None => added.push(test.clone()),
                Some(&old) => {
                    if old.fingerprint != test.fingerprint
                        || old.assertion_count != test.assertion_count
                    {
                        modified.push(TestItemDelta {
                            id: test.id.clone(),
                            old_fingerprint: old.fingerprint.clone(),
                            new_fingerprint: test.fingerprint.clone(),
                            old_assertion_count: old.assertion_count,
                            new_assertion_count: test.assertion_count,
                        });
                    } else {
                        unchanged.push(test.clone());
                    }
                }
            }
        }
    }

    let mut removed = Vec::new();
    for file in &baseline.files {
        for test in &file.tests {
            if !current_map.contains_key(test.id.as_str()) {
                removed.push(test.clone());
            }
        }
    }

    if !removed.is_empty() && !options.allow_shrink {
        return Err(RatchetError::MissingTests(
            removed.iter().map(|t| t.id.clone()).collect(),
        ));
    }

    if current.ratchet.total_tests < baseline.ratchet.total_tests && !options.allow_shrink {
        return Err(RatchetError::TestCountDecreased {
            baseline: baseline.ratchet.total_tests,
            current: current.ratchet.total_tests,
        });
    }

    let shrink_occurred =
        !removed.is_empty() || current.ratchet.total_tests < baseline.ratchet.total_tests;
    if shrink_occurred {
        let sig = current
            .signature
            .as_ref()
            .ok_or(RatchetError::ShrinkRequiresSignature)?;

        if !options.trusted_keys.is_empty() && !options.trusted_keys.contains(&sig.public_key) {
            return Err(RatchetError::UntrustedKey(sig.public_key.clone()));
        }

        match current.verify_signature() {
            Ok(true) => {}
            Ok(false) => {
                return Err(RatchetError::InvalidSignature(
                    "signature verification failed".to_string(),
                ));
            }
            Err(err) => {
                return Err(RatchetError::InvalidSignature(err.to_string()));
            }
        }
    }

    Ok(RatchetDiff {
        added,
        removed,
        modified,
        unchanged,
        baseline_total: baseline.ratchet.total_tests,
        current_total: current.ratchet.total_tests,
    })
}

/// Advances baseline to current manifest state if ratchet invariants pass, optionally signing it.
pub fn advance_baseline(
    baseline: &BaselineManifest,
    current: &BaselineManifest,
    options: &RatchetOptions,
    signing_key: Option<&SigningKey>,
) -> Result<BaselineManifest, RatchetError> {
    verify_ratchet(baseline, current, options)?;

    let mut advanced = current.clone();
    let total_tests = advanced.files.iter().map(|f| f.tests.len()).sum();
    let files_count = advanced.files.len();
    advanced.ratchet = RatchetState {
        total_tests,
        files_count,
    };
    advanced.signature = None;

    if let Some(key) = signing_key {
        advanced.sign(key)?;
    }

    advanced.validate()?;

    Ok(advanced)
}
