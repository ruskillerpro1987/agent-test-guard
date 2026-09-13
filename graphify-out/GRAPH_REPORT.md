# Graph Report - agent-test-guard  (2026-09-13)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 28 nodes · 12 edges · 20 communities
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `4538055c`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- main.rs
- agent-test-guard

## God Nodes (most connected - your core abstractions)
1. `Cli` - 3 edges
2. `Commands` - 3 edges
3. `agent-test-guard` - 3 edges
4. `agent-test-guard-ast` - 3 edges
5. `agent-test-guard-core` - 3 edges
6. `agent-test-guard-rules` - 3 edges

## Surprising Connections (you probably didn't know these)
- `agent-test-guard` --depends_on--> `agent-test-guard-ast`  [EXTRACTED]
  crates/cli/Cargo.toml → crates/ast/Cargo.toml
- `agent-test-guard` --depends_on--> `agent-test-guard-core`  [EXTRACTED]
  crates/cli/Cargo.toml → crates/core/Cargo.toml
- `agent-test-guard` --depends_on--> `agent-test-guard-rules`  [EXTRACTED]
  crates/cli/Cargo.toml → crates/rules/Cargo.toml
- `agent-test-guard-ast` --depends_on--> `agent-test-guard-core`  [EXTRACTED]
  crates/ast/Cargo.toml → crates/core/Cargo.toml
- `agent-test-guard-rules` --depends_on--> `agent-test-guard-ast`  [EXTRACTED]
  crates/rules/Cargo.toml → crates/ast/Cargo.toml

## Import Cycles
- None detected.

## Communities (20 total, 0 thin omitted)

### Community 0 - "main.rs"
Cohesion: 0.40
Nodes (4): Cli, Commands, Option, String

### Community 1 - "agent-test-guard"
Cohesion: 1.00
Nodes (4): agent-test-guard, agent-test-guard-ast, agent-test-guard-core, agent-test-guard-rules

## Suggested Questions
_Not enough signal to generate questions. This usually means the corpus has no AMBIGUOUS edges, no bridge nodes, no INFERRED relationships, and all communities are tightly cohesive. Add more files or run with --mode deep to extract richer edges._