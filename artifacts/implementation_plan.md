# Implementation Plan: Agent Test Guard (`agent-test-guard`)

> **Provenance:** Generated from Adversarial Multi-Agent Hyperplan Consensus (`ast-engineer`, `security-hunter`, `dx-cli-architect`, `polyglot-specialist`, `pragmatic-implementer`) and synthesized by `plan` agent.

---

## Executive Summary & Architecture Overview

**Agent Test Guard** is a standalone, ultra-low-latency (<25ms), polyglot anti-reward-hacking and test authenticity verification engine built in Rust 2021. It serves as a deterministic verification gate in pre-commit hooks and CI/CD pipelines to prevent AI coding agents from specification gaming—such as skipping tests, deleting assertions, hollowing test bodies, or injecting illegal mocks into E2E suites.

### Workspace & Crate Hierarchy
```
agent-test-guard/
├── Cargo.toml                      # Root workspace configuration
├── crates/
│   ├── core/                       # Canonical baseline schema, RFC 8785 JCS, BLAKE3 hashing, Ed25519 verification
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── canonical.rs        # RFC 8785 JSON Canonicalization Scheme (JCS)
│   │       ├── crypto.rs           # BLAKE3 keyed hashing & Ed25519 signing/verification
│   │       ├── schema.rs           # .test-guard.json schema definition & validation
│   │       └── ratchet.rs          # Monotonic ratchet & subset invariants (A ⊆ B, count >= baseline)
│   │
│   ├── ast/                        # Tree-sitter parsers, vendored grammars, S-expression query catalog
│   │   ├── Cargo.toml
│   │   ├── build.rs                # Offline cc-rs compilation for C/C++ grammars (MSVC / GCC / Clang)
│   │   ├── vendor/                 # Vendored Tree-sitter C parsers (TS, TSX, Rust, Python, Go)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── grammar.rs          # Language enum and grammar runtime initialization
│   │       ├── parser.rs           # Fault-tolerant CST parser & tree cache
│   │       ├── queries.rs          # S-expression pattern catalog for test constructs & assertions
│   │       └── fingerprint.rs      # AST node kind sequence serializer & BLAKE3 structural hasher
│   │
│   ├── rules/                      # Anti-cheat evaluation engine & lint rules
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs           # Rule registry, diagnostic collector, and evaluation pipeline
│   │       ├── anti_skip.rs        # Rule: Detects .skip, .todo, xdescribe, #[ignore], @pytest.mark.skip
│   │       ├── anti_tautology.rs   # Rule: Detects expect(true).toBe(true), assert 1 == 1, assert!(true)
│   │       ├── anti_hollowing.rs   # Rule: Enforces assertion count floor (>=1) & structural AST delta
│   │       ├── anti_mock.rs        # Rule: 3-Tier mock boundary (quarantine internal stubs from Tier-3 E2E)
│   │       └── import_validator.rs # Rule: Validates source import integrity (no dummy module mocks)
│   │
│   └── cli/                        # Dual-channel CLI, Git blob index reader, parallel file scanner
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── commands/
│           │   ├── init.rs         # Initialize baseline .test-guard.json
│           │   ├── check.rs        # Tier A (<25ms static staged) & Tier B (deep CI) checks
│           │   ├── ratchet.rs      # Advance baseline count & re-sign via Ed25519
│           │   ├── protect.rs      # Apply cooperative OS locks (attrib +r / chmod 444) & git hooks
│           │   └── explain.rs      # Detailed remediation guidance for violations
│           ├── git/
│           │   ├── mod.rs
│           │   └── blob_reader.rs  # Zero-copy Git index blob reader (Anti-TOCTOU)
│           ├── output/
│           │   ├── human.rs        # Biome-style ANSI formatted terminal reporter
│           │   └── agent.rs        # Machine-readable JSON diagnostic card for AI agent auto-correction
│           └── runner/
│               └── introspection.rs# Tier B dry-run runner (vitest list, pytest --collect-only, cargo test --list)
│
└── packages/                       # Universal npm distribution with platform-specific native binaries
    ├── agent-test-guard/           # Trampoline runner (<8ms)
    ├── agent-test-guard-win32-x64/ # Windows x64 binary
    ├── agent-test-guard-linux-x64/ # Linux x64 binary
    └── agent-test-guard-darwin-arm64/ # macOS Apple Silicon binary
```

---

## Hard Invariants & Technical Specifications

| Invariant | Specification | Enforcement Mechanism |
| :--- | :--- | :--- |
| **1. Pre-Commit Latency Gate** | Execution budget **strictly < 25ms** for typical commits (<50 test files). | Pure static Tree-sitter CST traversal in compiled Rust; blobs read straight from in-memory Git index via `git2` without checking out to disk. Zero subprocess invocation (`node`, `python`, `cargo`). |
| **2. Anti-TOCTOU Git Blob Reader** | Inspects staged blobs (`git index`), never the dirty working directory. | `repo.index()?.get_path(path, 0)?.id` followed by `repo.find_blob(oid)?.content()`. Prevents agent from committing one version while running tests against another. |
| **3. Sybil Test Substitution & Anti-Deletion** | Tests cannot be deleted, commented out, or renamed by an automated agent. | Invariant: $\text{BaselineTestIDs} \subseteq \text{CurrentTestIDs} \land |\text{Current}| \ge |\text{Baseline}|$. Shrinking requires `--allow-shrink` with valid Ed25519 signature verification. |
| **4. Anti-Hollowing Fingerprint** | Test bodies cannot be emptied or stripped of assertions. | Each test requires $\ge 1$ verified assertion node kind (`expect()`, `assert!`, `assert`, `t.True()`), plus a BLAKE3 structural hash of the sequence of Tree-sitter node kinds. |
| **5. 3-Tier Mock Boundary** | E2E tests cannot mock internal domain logic or transport. | Tier 1 (Unit): In-memory mocks permitted.<br>Tier 2 (Integration): Contract wire mocks permitted.<br>Tier 3 (E2E): Internal IPC, process mocking, and route stubbing trigger hard failure. |
| **6. Threat Model Realism** | Honest demarcation of security boundaries. | Documented dual boundary: Model A (Cooperative Specification Gamer) is constrained by local hooks & read-only locks; Model B (Hostile Local Actor) is strictly bounded by remote GitHub Actions CI with Ed25519 branch protection. |

---

## 4-Phase Phased Implementation Roadmap

### Phase 1: Core Engine & AST Foundations
1. Root Workspace & `crates/core` Scaffold (RFC 8785 JCS, BLAKE3, Ed25519).
2. Cryptographic Primitives & Baseline Schema (`.test-guard.json`).
3. Tree-sitter C Grammar Vendoring & `build.rs` (TS, TSX, Rust).
4. AST S-Expression Queries & Structural Fingerprinting.

### Phase 2: Rules Engine & Git Index Blob Integration
1. Anti-Skip Rule (E001) for TS/Rust.
2. Anti-Tautology Rule (E002).
3. Anti-Hollowing & Assertion Floor Rule (E003).
4. Anti-TOCTOU Git Index Zero-Copy Blob Reader.
5. Ratchet Monotonic Engine.

### Phase 3: Polyglot Expansion, CLI Commands & Dual Output
1. Python & Go Grammars & S-Expression Catalog.
2. 3-Tier Mock Boundary Rule (E004) for E2E suites.
3. Dual-Channel Output (Human ANSI + Agent JSON Diagnostic Cards).
4. CLI Subcommands (`init`, `check`, `ratchet`, `protect`, `explain`).
5. Tier B Hermetic Introspection Runner (`vitest list`, `pytest --collect-only`, `cargo test -- --list`).

### Phase 4: Distribution, Packaging & CI Integration
1. npm Platform Binary Distribution & Trampoline (<8ms).
2. Pre-Commit Hook Installer (`protect` command).
3. Turnkey GitHub Actions CI Starter Workflow.
4. Adversarial Red-Team Test Suite (6 Attack Vectors).
