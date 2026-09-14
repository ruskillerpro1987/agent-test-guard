//! Anti-reward-hacking rules engine for Agent Test Guard

pub mod anti_hollowing;
pub mod anti_mock;
pub mod anti_skip;
pub mod anti_tautology;
pub mod diagnostic;
pub mod engine;
pub mod import_validator;

pub use anti_skip::{AntiSkipRule, RuleError};
pub use diagnostic::{Diagnostic, RuleCode, Severity, Span};
