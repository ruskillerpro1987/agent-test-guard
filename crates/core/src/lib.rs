//! Core module for Agent Test Guard
//!
//! Provides baseline schema, RFC 8785 JSON canonicalization, BLAKE3 hashing,
//! Ed25519 signature verification, and monotonic ratchet invariants.

pub mod canonical;
pub mod crypto;
pub mod ratchet;
pub mod schema;

pub use canonical::{
    canonicalize_json_str, canonicalize_value, to_canonical_string, to_canonical_vec,
    CanonicalError,
};
pub use crypto::{
    blake3_hash, blake3_hash_hex, blake3_keyed_hash, blake3_keyed_hash_hex, derive_key,
    generate_keypair, sign, sign_hex, signing_key_from_hex, signing_key_to_hex, verify, verify_hex,
    verify_raw, verifying_key_from_hex, verifying_key_to_hex, CryptoError, Signature, SigningKey,
    VerifyingKey,
};
pub use ratchet::{
    advance_baseline, verify_ratchet, RatchetDiff, RatchetError, RatchetOptions, TestItemDelta,
};
pub use schema::{
    BaselineConfig, BaselineManifest, RatchetState, SchemaError, SignatureBlock, TestFileBaseline,
    TestItemBaseline, TestLanguage, TestTier,
};
