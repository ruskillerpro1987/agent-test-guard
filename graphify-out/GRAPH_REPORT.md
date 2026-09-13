# Graph Report - agent-test-guard  (2026-09-13)

## Corpus Check
- 30 files · ~8,854 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 186 nodes · 396 edges · 23 communities
- Extraction: 83% EXTRACTED · 17% INFERRED · 0% AMBIGUOUS · INFERRED: 69 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `0f873082`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- main.rs
- agent-test-guard
- CanonicalError
- ratchet_test.rs
- schema_test.rs
- ratchet.rs
- schema.rs
- 🛡️ Agent Test Guard (`agent-test-guard`)
- 4-Phase Phased Implementation Roadmap
- crypto.rs

## God Nodes (most connected - your core abstractions)
1. `verify_ratchet()` - 20 edges
2. `sample_baseline_manifest()` - 19 edges
3. `BaselineManifest` - 17 edges
4. `generate_keypair()` - 15 edges
5. `sample_valid_manifest()` - 13 edges
6. `CryptoError` - 12 edges
7. `CanonicalError` - 11 edges
8. `canonicalize_value()` - 11 edges
9. `SchemaError` - 11 edges
10. `to_canonical_string()` - 10 edges

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

## Communities (23 total, 0 thin omitted)

### Community 0 - "main.rs"
Cohesion: 0.40
Nodes (4): Cli, Commands, Option, String

### Community 1 - "agent-test-guard"
Cohesion: 1.00
Nodes (4): agent-test-guard, agent-test-guard-ast, agent-test-guard-core, agent-test-guard-rules

### Community 7 - "CanonicalError"
Cohesion: 0.15
Nodes (25): CanonicalError, canonicalize_json_str(), canonicalize_value(), Error, Result, Self, String, Vec (+17 more)

### Community 8 - "ratchet_test.rs"
Cohesion: 0.25
Nodes (21): verify_ratchet(), make_manifest(), make_test_file(), make_test_item(), Vec, sample_baseline_manifest(), test_advance_baseline_fails_when_unauthorized_shrink_attempted(), test_advance_baseline_succeeds_when_adding_tests_and_signs_if_key_provided() (+13 more)

### Community 9 - "schema_test.rs"
Cohesion: 0.26
Nodes (12): sample_valid_manifest(), test_baseline_roundtrip_when_valid_json_serialized_and_deserialized(), test_canonical_digest_when_manifest_formatted_or_keys_reordered(), test_signing_and_verification_when_valid_ed25519_key(), test_validate_fails_when_assertion_floor_violated(), test_validate_fails_when_duplicate_test_id_present(), test_validate_fails_when_files_count_mismatches_actual_count(), test_validate_fails_when_hash_or_fingerprint_hex_is_invalid() (+4 more)

### Community 10 - "ratchet.rs"
Cohesion: 0.33
Nodes (10): advance_baseline(), RatchetDiff, RatchetError, RatchetOptions, Option, Result, SigningKey, String (+2 more)

### Community 11 - "schema.rs"
Cohesion: 0.16
Nodes (19): BaselineConfig, BaselineManifest, is_hex_len(), is_valid_path(), RatchetState, Error, Option, Result (+11 more)

### Community 20 - "🛡️ Agent Test Guard (`agent-test-guard`)"
Cohesion: 0.17
Nodes (11): 1. Dual-Tier Verification Protocol, 2. Cryptographic Auto-Ratchet, 3. 3-Tier Mock Boundary Enforcement, 4. Dual Boundary Threat Model, 🛡️ Agent Test Guard (`agent-test-guard`), 🚀 CLI Commands, 🏛️ Core Architecture & Invariants, 📄 Diagnostic Codes (+3 more)

### Community 21 - "4-Phase Phased Implementation Roadmap"
Cohesion: 0.20
Nodes (9): 4-Phase Phased Implementation Roadmap, Executive Summary & Architecture Overview, Hard Invariants & Technical Specifications, Implementation Plan: Agent Test Guard (`agent-test-guard`), Phase 1: Core Engine & AST Foundations, Phase 2: Rules Engine & Git Index Blob Integration, Phase 3: Polyglot Expansion, CLI Commands & Dual Output, Phase 4: Distribution, Packaging & CI Integration (+1 more)

### Community 23 - "crypto.rs"
Cohesion: 0.16
Nodes (37): blake3_hash(), blake3_hash_hex(), blake3_keyed_hash(), blake3_keyed_hash_hex(), CryptoError, decode_fixed_hex(), derive_key(), fill_random_bytes() (+29 more)

## Knowledge Gaps
- **15 isolated node(s):** `⚡ The Problem: Specification Gaming & Reward Hacking`, `1. Dual-Tier Verification Protocol`, `2. Cryptographic Auto-Ratchet`, `3. 3-Tier Mock Boundary Enforcement`, `4. Dual Boundary Threat Model` (+10 more)
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `SchemaError` connect `schema.rs` to `ratchet.rs`, `crypto.rs`, `CanonicalError`?**
  _High betweenness centrality (0.111) - this node is a cross-community bridge._
- **Why does `CanonicalError` connect `CanonicalError` to `schema.rs`?**
  _High betweenness centrality (0.085) - this node is a cross-community bridge._
- **Why does `CryptoError` connect `crypto.rs` to `ratchet.rs`, `schema.rs`?**
  _High betweenness centrality (0.066) - this node is a cross-community bridge._
- **Are the 13 inferred relationships involving `verify_ratchet()` (e.g. with `test_verify_ratchet_fails_when_allow_shrink_true_with_invalid_signature()` and `test_verify_ratchet_fails_when_allow_shrink_true_with_untrusted_public_key()`) actually correct?**
  _`verify_ratchet()` has 13 INFERRED edges - model-reasoned connections that need verification._
- **Are the 11 inferred relationships involving `generate_keypair()` (e.g. with `test_ed25519_sign_and_verify_when_valid_keypair_and_payload()` and `test_ed25519_verify_fails_when_message_tampered()`) actually correct?**
  _`generate_keypair()` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `⚡ The Problem: Specification Gaming & Reward Hacking`, `1. Dual-Tier Verification Protocol`, `2. Cryptographic Auto-Ratchet` to the rest of the system?**
  _15 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `CanonicalError` be split into smaller, more focused modules?**
  _Cohesion score 0.14532019704433496 - nodes in this community are weakly interconnected._