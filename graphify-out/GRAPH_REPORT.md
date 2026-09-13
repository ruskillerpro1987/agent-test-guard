# Graph Report - agent-test-guard  (2026-09-13)

## Corpus Check
- 28 files · ~4,796 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 115 nodes · 210 edges · 27 communities (26 shown, 1 thin omitted)
- Extraction: 75% EXTRACTED · 25% INFERRED · 0% AMBIGUOUS · INFERRED: 52 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `141edcb6`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- main.rs
- agent-test-guard
- canonicalize_value
- crypto.rs
- 🛡️ Agent Test Guard (`agent-test-guard`)
- 4-Phase Phased Implementation Roadmap
- AuditRecord
- verify
- hex_encode
- verify_hex
- crypto_test.rs

## God Nodes (most connected - your core abstractions)
1. `canonicalize_value()` - 11 edges
2. `CanonicalError` - 10 edges
3. `to_canonical_string()` - 10 edges
4. `verify()` - 10 edges
5. `verify_hex()` - 10 edges
6. `CryptoError` - 9 edges
7. `generate_keypair()` - 9 edges
8. `sign_hex()` - 9 edges
9. `verifying_key_to_hex()` - 9 edges
10. `to_canonical_vec()` - 8 edges

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

## Communities (27 total, 1 thin omitted)

### Community 0 - "main.rs"
Cohesion: 0.40
Nodes (4): Cli, Commands, Option, String

### Community 1 - "agent-test-guard"
Cohesion: 1.00
Nodes (4): agent-test-guard, agent-test-guard-ast, agent-test-guard-core, agent-test-guard-rules

### Community 7 - "canonicalize_value"
Cohesion: 0.18
Nodes (22): CanonicalError, canonicalize_json_str(), canonicalize_value(), Result, String, Vec, to_canonical_string(), to_canonical_vec() (+14 more)

### Community 8 - "crypto.rs"
Cohesion: 0.39
Nodes (7): CryptoError, decode_fixed_hex(), fill_random_bytes(), hex_decode(), hex_val(), Option, Vec

### Community 20 - "🛡️ Agent Test Guard (`agent-test-guard`)"
Cohesion: 0.17
Nodes (11): 1. Dual-Tier Verification Protocol, 2. Cryptographic Auto-Ratchet, 3. 3-Tier Mock Boundary Enforcement, 4. Dual Boundary Threat Model, 🛡️ Agent Test Guard (`agent-test-guard`), 🚀 CLI Commands, 🏛️ Core Architecture & Invariants, 📄 Diagnostic Codes (+3 more)

### Community 21 - "4-Phase Phased Implementation Roadmap"
Cohesion: 0.20
Nodes (9): 4-Phase Phased Implementation Roadmap, Executive Summary & Architecture Overview, Hard Invariants & Technical Specifications, Implementation Plan: Agent Test Guard (`agent-test-guard`), Phase 1: Core Engine & AST Foundations, Phase 2: Rules Engine & Git Index Blob Integration, Phase 3: Polyglot Expansion, CLI Commands & Dual Output, Phase 4: Distribution, Packaging & CI Integration (+1 more)

### Community 22 - "AuditRecord"
Cohesion: 1.00
Nodes (3): AuditRecord, LogMeta, String

### Community 23 - "verify"
Cohesion: 0.42
Nodes (13): generate_keypair(), sign(), sign_hex(), verify(), verify_raw(), verifying_key_to_hex(), test_ed25519_sign_and_verify_when_valid_keypair_and_payload(), test_ed25519_verify_fails_when_message_tampered() (+5 more)

### Community 24 - "hex_encode"
Cohesion: 0.36
Nodes (9): blake3_hash(), blake3_hash_hex(), blake3_keyed_hash(), blake3_keyed_hash_hex(), hex_encode(), String, signing_key_to_hex(), test_blake3_hash_when_empty_string_and_arbitrary_data() (+1 more)

### Community 25 - "verify_hex"
Cohesion: 0.60
Nodes (6): Result, signing_key_from_hex(), verify_hex(), verifying_key_from_hex(), test_hex_decode_fails_when_invalid_hex_or_wrong_length(), test_hex_roundtrip_when_signing_and_verifying_keys()

## Knowledge Gaps
- **15 isolated node(s):** `⚡ The Problem: Specification Gaming & Reward Hacking`, `1. Dual-Tier Verification Protocol`, `2. Cryptographic Auto-Ratchet`, `3. 3-Tier Mock Boundary Enforcement`, `4. Dual Boundary Threat Model` (+10 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Are the 4 inferred relationships involving `canonicalize_value()` (e.g. with `test_array_ordering_when_elements_unordered()` and `test_idempotence_when_canonicalized_repeatedly()`) actually correct?**
  _`canonicalize_value()` has 4 INFERRED edges - model-reasoned connections that need verification._
- **Are the 3 inferred relationships involving `to_canonical_string()` (e.g. with `test_error_handling_when_invalid_json_or_non_finite_float()` and `test_number_formatting_when_floats_and_integers_processed()`) actually correct?**
  _`to_canonical_string()` has 3 INFERRED edges - model-reasoned connections that need verification._
- **Are the 3 inferred relationships involving `verify()` (e.g. with `test_ed25519_sign_and_verify_when_valid_keypair_and_payload()` and `test_ed25519_verify_fails_when_message_tampered()`) actually correct?**
  _`verify()` has 3 INFERRED edges - model-reasoned connections that need verification._
- **Are the 5 inferred relationships involving `verify_hex()` (e.g. with `test_ed25519_sign_and_verify_when_valid_keypair_and_payload()` and `test_ed25519_verify_fails_when_message_tampered()`) actually correct?**
  _`verify_hex()` has 5 INFERRED edges - model-reasoned connections that need verification._
- **What connects `⚡ The Problem: Specification Gaming & Reward Hacking`, `1. Dual-Tier Verification Protocol`, `2. Cryptographic Auto-Ratchet` to the rest of the system?**
  _15 weakly-connected nodes found - possible documentation gaps or missing edges._