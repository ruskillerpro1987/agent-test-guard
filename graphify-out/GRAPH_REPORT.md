# Graph Report - agent-test-guard  (2026-09-13)

## Corpus Check
- 27 files · ~3,240 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 77 nodes · 90 edges · 23 communities
- Extraction: 87% EXTRACTED · 13% INFERRED · 0% AMBIGUOUS · INFERRED: 12 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f4735bc8`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- main.rs
- agent-test-guard
- canonicalize_value
- 🛡️ Agent Test Guard (`agent-test-guard`)
- 4-Phase Phased Implementation Roadmap
- canonical_test.rs

## God Nodes (most connected - your core abstractions)
1. `canonicalize_value()` - 11 edges
2. `CanonicalError` - 10 edges
3. `to_canonical_string()` - 10 edges
4. `to_canonical_vec()` - 8 edges
5. `canonicalize_json_str()` - 8 edges
6. `🛡️ Agent Test Guard (`agent-test-guard`)` - 7 edges
7. `🏛️ Core Architecture & Invariants` - 5 edges
8. `4-Phase Phased Implementation Roadmap` - 5 edges
9. `test_error_handling_when_invalid_json_or_non_finite_float()` - 4 edges
10. `Implementation Plan: Agent Test Guard (`agent-test-guard`)` - 4 edges

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
Nodes (4): Cli, Commands, String, Option

### Community 1 - "agent-test-guard"
Cohesion: 1.00
Nodes (4): agent-test-guard, agent-test-guard-ast, agent-test-guard-core, agent-test-guard-rules

### Community 7 - "canonicalize_value"
Cohesion: 0.23
Nodes (17): CanonicalError, canonicalize_json_str(), canonicalize_value(), String, to_canonical_string(), to_canonical_vec(), test_error_handling_when_invalid_json_or_non_finite_float(), test_idempotence_when_canonicalized_repeatedly() (+9 more)

### Community 20 - "🛡️ Agent Test Guard (`agent-test-guard`)"
Cohesion: 0.17
Nodes (11): 1. Dual-Tier Verification Protocol, 2. Cryptographic Auto-Ratchet, 3. 3-Tier Mock Boundary Enforcement, 4. Dual Boundary Threat Model, 🛡️ Agent Test Guard (`agent-test-guard`), 🚀 CLI Commands, 🏛️ Core Architecture & Invariants, 📄 Diagnostic Codes (+3 more)

### Community 21 - "4-Phase Phased Implementation Roadmap"
Cohesion: 0.20
Nodes (9): 4-Phase Phased Implementation Roadmap, Executive Summary & Architecture Overview, Hard Invariants & Technical Specifications, Implementation Plan: Agent Test Guard (`agent-test-guard`), Phase 1: Core Engine & AST Foundations, Phase 2: Rules Engine & Git Index Blob Integration, Phase 3: Polyglot Expansion, CLI Commands & Dual Output, Phase 4: Distribution, Packaging & CI Integration (+1 more)

### Community 22 - "canonical_test.rs"
Cohesion: 0.28
Nodes (8): AuditRecord, LogMeta, String, test_array_ordering_when_elements_unordered(), test_lexicographical_key_sorting_when_nested_and_utf16_code_units(), test_minimal_escaping_when_control_chars_and_utf8_present(), test_number_formatting_when_floats_and_integers_processed(), test_whitespace_removal_when_multiline_and_spaced_json()

## Knowledge Gaps
- **15 isolated node(s):** `⚡ The Problem: Specification Gaming & Reward Hacking`, `1. Dual-Tier Verification Protocol`, `2. Cryptographic Auto-Ratchet`, `3. 3-Tier Mock Boundary Enforcement`, `4. Dual Boundary Threat Model` (+10 more)
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `canonicalize_value()` connect `canonicalize_value` to `canonical_test.rs`?**
  _High betweenness centrality (0.028) - this node is a cross-community bridge._
- **Why does `to_canonical_string()` connect `canonicalize_value` to `canonical_test.rs`?**
  _High betweenness centrality (0.018) - this node is a cross-community bridge._
- **Are the 4 inferred relationships involving `canonicalize_value()` (e.g. with `test_array_ordering_when_elements_unordered()` and `test_idempotence_when_canonicalized_repeatedly()`) actually correct?**
  _`canonicalize_value()` has 4 INFERRED edges - model-reasoned connections that need verification._
- **Are the 3 inferred relationships involving `to_canonical_string()` (e.g. with `test_error_handling_when_invalid_json_or_non_finite_float()` and `test_number_formatting_when_floats_and_integers_processed()`) actually correct?**
  _`to_canonical_string()` has 3 INFERRED edges - model-reasoned connections that need verification._
- **Are the 2 inferred relationships involving `to_canonical_vec()` (e.g. with `test_error_handling_when_invalid_json_or_non_finite_float()` and `test_typed_struct_serialization_when_string_and_vec_invoked()`) actually correct?**
  _`to_canonical_vec()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Are the 3 inferred relationships involving `canonicalize_json_str()` (e.g. with `test_error_handling_when_invalid_json_or_non_finite_float()` and `test_idempotence_when_canonicalized_repeatedly()`) actually correct?**
  _`canonicalize_json_str()` has 3 INFERRED edges - model-reasoned connections that need verification._
- **What connects `⚡ The Problem: Specification Gaming & Reward Hacking`, `1. Dual-Tier Verification Protocol`, `2. Cryptographic Auto-Ratchet` to the rest of the system?**
  _15 weakly-connected nodes found - possible documentation gaps or missing edges._