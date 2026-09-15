//! Anti-reward-hacking rules engine for Agent Test Guard

pub mod anti_hollowing;
pub mod anti_mock;
pub mod anti_skip;
pub mod anti_tautology;
pub mod diagnostic;
pub mod engine;
pub mod import_validator;

pub use anti_hollowing::AntiHollowingRule;
pub use anti_mock::{AntiMockRule, TestTier};
pub use anti_skip::{AntiSkipRule, RuleError};
pub use anti_tautology::AntiTautologyRule;
pub use diagnostic::{Diagnostic, RuleCode, Severity, Span};
pub use import_validator::ImportValidatorRule;
pub use engine::{EngineConfig, RuleEngine};