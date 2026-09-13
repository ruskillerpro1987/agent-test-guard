# Graph Report - agent-test-guard  (2026-09-13)

## Corpus Check
- 29 files · ~6,543 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 154 nodes · 291 edges · 26 communities
- Extraction: 83% EXTRACTED · 17% INFERRED · 0% AMBIGUOUS · INFERRED: 50 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `3d55248d`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- main.rs
- agent-test-guard
- CanonicalError
- CryptoError
- schema_test.rs
- schema.rs
- 🛡️ Agent Test Guard (`agent-test-guard`)
- 4-Phase Phased Implementation Roadmap
- sign_hex
- generate_keypair
- crypto.rs
- crypto_test.rs

## God Nodes (most connected - your core abstractions)
1. `BaselineManifest` - 13 edges
2. `sample_valid_manifest()` - 13 edges
3. `CanonicalError` - 11 edges
4. `canonicalize_value()` - 11 edges
5. `generate_keypair()` - 11 edges
6. `to_canonical_string()` - 10 edges
7. `CryptoError` - 10 edges
8. `verify()` - 10 edges
9. `verify_hex()` - 10 edges
10. `SchemaError` - 10 edges

## Surprising Connections (you probably didn't know these)
- `test_number_formatting_when_floats_and_integers_processed()` --calls--> `to_canonical_string()`  [INFERRED]
  crates/core/tests/canonical_test.rs → crates/core/src/canonical.rs
- `test_whitespace_removal_when_multiline_and_spaced_json()` --calls--> `canonicalize_json_str()`  [INFERRED]
  crates/core/tests/canonical_test.rs → crates/core/src/canonical.rs
- `test_array_ordering_when_elements_unordered()` --calls--> `canonicalize_value()`  [INFERRED]
  crates/core/tests/canonical_test.rs → crates/core/src/canonical.rs
- `test_lexicographical_key_sorting_when_nested_and_utf16_code_units()` --calls--> `canonicalize_value()`  [INFERRED]
  crates/core/tests/canonical_test.rs → crates/core/src/canonical.rs
- `test_minimal_escaping_when_control_chars_and_utf8_present()` --calls--> `canonicalize_value()`  [INFERRED]
  crates/core/tests/canonical_test.rs → crates/core/src/canonical.rs

## Import Cycles
- None detected.

## Communities (26 total, 0 thin omitted)

### Community 0 - "main.rs"
Cohesion: 0.40
Nodes (4): Cli, Commands, Option, String

### Community 1 - "agent-test-guard"
Cohesion: 1.00
Nodes (4): agent-test-guard, agent-test-guard-ast, agent-test-guard-core, agent-test-guard-rules

### Community 7 - "CanonicalError"
Cohesion: 0.15
Nodes (25): CanonicalError, canonicalize_json_str(), canonicalize_value(), Error, Result, Self, String, Vec (+17 more)

### Community 8 - "CryptoError"
Cohesion: 0.40
Nodes (6): CryptoError, decode_fixed_hex(), hex_decode(), hex_val(), Option, Vec

### Community 9 - "schema_test.rs"
Cohesion: 0.26
Nodes (12): sample_valid_manifest(), test_baseline_roundtrip_when_valid_json_serialized_and_deserialized(), test_canonical_digest_when_manifest_formatted_or_keys_reordered(), test_signing_and_verification_when_valid_ed25519_key(), test_validate_fails_when_assertion_floor_violated(), test_validate_fails_when_duplicate_test_id_present(), test_validate_fails_when_files_count_mismatches_actual_count(), test_validate_fails_when_hash_or_fingerprint_hex_is_invalid() (+4 more)

### Community 11 - "schema.rs"
Cohesion: 0.16
Nodes (19): BaselineConfig, BaselineManifest, is_hex_len(), is_valid_path(), RatchetState, Error, Option, Result (+11 more)

### Community 20 - "🛡️ Agent Test Guard (`agent-test-guard`)"
Cohesion: 0.17
Nodes (11): 1. Dual-Tier Verification Protocol, 2. Cryptographic Auto-Ratchet, 3. 3-Tier Mock Boundary Enforcement, 4. Dual Boundary Threat Model, 🛡️ Agent Test Guard (`agent-test-guard`), 🚀 CLI Commands, 🏛️ Core Architecture & Invariants, 📄 Diagnostic Codes (+3 more)

### Community 21 - "4-Phase Phased Implementation Roadmap"
Cohesion: 0.20
Nodes (9): 4-Phase Phased Implementation Roadmap, Executive Summary & Architecture Overview, Hard Invariants & Technical Specifications, Implementation Plan: Agent Test Guard (`agent-test-guard`), Phase 1: Core Engine & AST Foundations, Phase 2: Rules Engine & Git Index Blob Integration, Phase 3: Polyglot Expansion, CLI Commands & Dual Output, Phase 4: Distribution, Packaging & CI Integration (+1 more)

### Community 22 - "sign_hex"
Cohesion: 0.67
Nodes (4): SigningKey, sign(), sign_hex(), Signature

### Community 23 - "generate_keypair"
Cohesion: 0.42
Nodes (12): fill_random_bytes(), generate_keypair(), Result, verify(), verify_hex(), verify_raw(), verifying_key_to_hex(), test_ed25519_sign_and_verify_when_valid_keypair_and_payload() (+4 more)

### Community 24 - "crypto.rs"
Cohesion: 0.42
Nodes (9): blake3_hash(), blake3_hash_hex(), blake3_keyed_hash(), blake3_keyed_hash_hex(), hex_encode(), String, signing_key_to_hex(), test_blake3_hash_when_empty_string_and_arbitrary_data() (+1 more)

### Community 25 - "crypto_test.rs"
Cohesion: 0.38
Nodes (6): derive_key(), signing_key_from_hex(), verifying_key_from_hex(), test_derive_key_when_different_contexts_and_same_material(), test_hex_decode_fails_when_invalid_hex_or_wrong_length(), test_hex_roundtrip_when_signing_and_verifying_keys()

## Knowledge Gaps
- **15 isolated node(s):** `⚡ The Problem: Specification Gaming & Reward Hacking`, `1. Dual-Tier Verification Protocol`, `2. Cryptographic Auto-Ratchet`, `3. 3-Tier Mock Boundary Enforcement`, `4. Dual Boundary Threat Model` (+10 more)
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `SchemaError` connect `schema.rs` to `CryptoError`, `CanonicalError`?**
  _High betweenness centrality (0.134) - this node is a cross-community bridge._
- **Why does `CanonicalError` connect `CanonicalError` to `schema.rs`?**
  _High betweenness centrality (0.094) - this node is a cross-community bridge._
- **Why does `CryptoError` connect `CryptoError` to `crypto.rs`, `crypto_test.rs`, `schema.rs`, `generate_keypair`?**
  _High betweenness centrality (0.072) - this node is a cross-community bridge._
- **Are the 4 inferred relationships involving `canonicalize_value()` (e.g. with `test_array_ordering_when_elements_unordered()` and `test_idempotence_when_canonicalized_repeatedly()`) actually correct?**
  _`canonicalize_value()` has 4 INFERRED edges - model-reasoned connections that need verification._
- **Are the 7 inferred relationships involving `generate_keypair()` (e.g. with `test_ed25519_sign_and_verify_when_valid_keypair_and_payload()` and `test_ed25519_verify_fails_when_message_tampered()`) actually correct?**
  _`generate_keypair()` has 7 INFERRED edges - model-reasoned connections that need verification._
- **What connects `⚡ The Problem: Specification Gaming & Reward Hacking`, `1. Dual-Tier Verification Protocol`, `2. Cryptographic Auto-Ratchet` to the rest of the system?**
  _15 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `CanonicalError` be split into smaller, more focused modules?**
  _Cohesion score 0.14532019704433496 - nodes in this community are weakly interconnected._