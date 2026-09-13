# Graph Report - agent-test-guard  (2026-09-13)

## Corpus Check
- 31 files · ~11,809 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 241 nodes · 515 edges · 19 communities
- Extraction: 84% EXTRACTED · 16% INFERRED · 0% AMBIGUOUS · INFERRED: 83 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `df344570`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- main.rs
- agent-test-guard
- ast_test.rs
- queries.rs
- CanonicalError
- ratchet_test.rs
- schema_test.rs
- schema.rs
- 🛡️ Agent Test Guard (`agent-test-guard`)
- 4-Phase Phased Implementation Roadmap
- crypto.rs

## God Nodes (most connected - your core abstractions)
1. `verify_ratchet()` - 20 edges
2. `sample_baseline_manifest()` - 19 edges
3. `discover_tests()` - 18 edges
4. `BaselineManifest` - 17 edges
5. `generate_keypair()` - 15 edges
6. `sample_valid_manifest()` - 13 edges
7. `CryptoError` - 12 edges
8. `CanonicalError` - 11 edges
9. `canonicalize_value()` - 11 edges
10. `SchemaError` - 11 edges

## Surprising Connections (you probably didn't know these)
- `test_structural_fingerprint_determinism_when_same_code_hashed()` --calls--> `compute_structural_fingerprint()`  [INFERRED]
  crates/ast/tests/ast_test.rs → crates/ast/src/fingerprint.rs
- `test_structural_fingerprint_invariance_when_formatting_and_whitespace_altered()` --calls--> `compute_structural_fingerprint()`  [INFERRED]
  crates/ast/tests/ast_test.rs → crates/ast/src/fingerprint.rs
- `test_structural_fingerprint_invariance_when_variable_identifiers_renamed()` --calls--> `compute_structural_fingerprint()`  [INFERRED]
  crates/ast/tests/ast_test.rs → crates/ast/src/fingerprint.rs
- `test_structural_fingerprint_sensitivity_when_test_body_mutated_or_hollowed()` --calls--> `compute_structural_fingerprint()`  [INFERRED]
  crates/ast/tests/ast_test.rs → crates/ast/src/fingerprint.rs
- `test_language_detection_when_file_paths_have_standard_extensions()` --calls--> `detect_language()`  [INFERRED]
  crates/ast/tests/ast_test.rs → crates/ast/src/grammar.rs

## Import Cycles
- None detected.

## Communities (19 total, 0 thin omitted)

### Community 0 - "main.rs"
Cohesion: 0.40
Nodes (4): Cli, Commands, Option, String

### Community 1 - "agent-test-guard"
Cohesion: 1.00
Nodes (4): agent-test-guard, agent-test-guard-ast, agent-test-guard-core, agent-test-guard-rules

### Community 2 - "ast_test.rs"
Cohesion: 0.07
Nodes (34): AsRef, compute_structural_fingerprint(), hash_node_structure(), Node, String, detect_language(), GrammarLanguage, Option (+26 more)

### Community 6 - "queries.rs"
Cohesion: 0.32
Nodes (16): classify_js_call(), collect_rust_tests(), count_js_assertions(), count_rust_assertions(), DiscoveredTest, extract_string_literal(), inspect_rust_function(), is_js_assertion_call() (+8 more)

### Community 7 - "CanonicalError"
Cohesion: 0.15
Nodes (25): CanonicalError, canonicalize_json_str(), canonicalize_value(), Error, Result, Self, String, Vec (+17 more)

### Community 8 - "ratchet_test.rs"
Cohesion: 0.16
Nodes (31): advance_baseline(), RatchetDiff, RatchetError, RatchetOptions, Option, Result, SigningKey, String (+23 more)

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

### Community 23 - "crypto.rs"
Cohesion: 0.16
Nodes (37): blake3_hash(), blake3_hash_hex(), blake3_keyed_hash(), blake3_keyed_hash_hex(), CryptoError, decode_fixed_hex(), derive_key(), fill_random_bytes() (+29 more)

## Knowledge Gaps
- **15 isolated node(s):** `⚡ The Problem: Specification Gaming & Reward Hacking`, `1. Dual-Tier Verification Protocol`, `2. Cryptographic Auto-Ratchet`, `3. 3-Tier Mock Boundary Enforcement`, `4. Dual Boundary Threat Model` (+10 more)
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `SchemaError` connect `schema.rs` to `ratchet_test.rs`, `crypto.rs`, `CanonicalError`?**
  _High betweenness centrality (0.066) - this node is a cross-community bridge._
- **Why does `CanonicalError` connect `CanonicalError` to `schema.rs`?**
  _High betweenness centrality (0.051) - this node is a cross-community bridge._
- **Why does `CryptoError` connect `crypto.rs` to `ratchet_test.rs`, `schema.rs`?**
  _High betweenness centrality (0.039) - this node is a cross-community bridge._
- **Are the 13 inferred relationships involving `verify_ratchet()` (e.g. with `test_verify_ratchet_fails_when_allow_shrink_true_with_invalid_signature()` and `test_verify_ratchet_fails_when_allow_shrink_true_with_untrusted_public_key()`) actually correct?**
  _`verify_ratchet()` has 13 INFERRED edges - model-reasoned connections that need verification._
- **Are the 9 inferred relationships involving `discover_tests()` (e.g. with `test_rust_discovery_when_async_tokio_test_present()` and `test_rust_discovery_when_non_test_functions_and_multiple_assertions_present()`) actually correct?**
  _`discover_tests()` has 9 INFERRED edges - model-reasoned connections that need verification._
- **Are the 11 inferred relationships involving `generate_keypair()` (e.g. with `test_ed25519_sign_and_verify_when_valid_keypair_and_payload()` and `test_ed25519_verify_fails_when_message_tampered()`) actually correct?**
  _`generate_keypair()` has 11 INFERRED edges - model-reasoned connections that need verification._
- **What connects `⚡ The Problem: Specification Gaming & Reward Hacking`, `1. Dual-Tier Verification Protocol`, `2. Cryptographic Auto-Ratchet` to the rest of the system?**
  _15 weakly-connected nodes found - possible documentation gaps or missing edges._