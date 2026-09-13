//! Core module for Agent Test Guard
//!
//! Provides baseline schema, RFC 8785 JSON canonicalization, BLAKE3 hashing,
//! Ed25519 signature verification, and monotonic ratchet invariants.

pub mod canonical;
pub mod crypto;
pub mod ratchet;
pub mod schema;
