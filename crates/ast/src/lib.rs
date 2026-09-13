//! AST and Tree-sitter parsing module for Agent Test Guard

pub mod fingerprint;
pub mod grammar;
pub mod parser;
pub mod queries;

pub use fingerprint::compute_structural_fingerprint;
pub use grammar::{detect_language, GrammarLanguage};
pub use parser::{parse, ParseError};
pub use queries::{discover_tests, DiscoveredTest, QueryError};
