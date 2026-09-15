//! Multi-file AST rule evaluation coordinator

use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_ast::parser::parse;

use crate::anti_hollowing::AntiHollowingRule;
use crate::anti_mock::{AntiMockRule, TestTier};
use crate::anti_skip::{AntiSkipRule, RuleError};
use crate::anti_tautology::AntiTautologyRule;
use crate::diagnostic::Diagnostic;
use crate::import_validator::ImportValidatorRule;

/// Configuration options for the AST rule engine.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub min_assertions: usize,
    pub enforce_mock_boundary: bool,
    pub enforce_import_integrity: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            min_assertions: AntiHollowingRule::DEFAULT_ASSERTION_FLOOR,
            enforce_mock_boundary: true,
            enforce_import_integrity: true,
        }
    }
}

/// Unified coordinator executing all safety rules (E001..E005) against source files.
pub struct RuleEngine {
    config: EngineConfig,
    hollowing_rule: AntiHollowingRule,
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

impl RuleEngine {
    pub fn new(config: EngineConfig) -> Self {
        let hollowing_rule = AntiHollowingRule::new(config.min_assertions);
        Self {
            config,
            hollowing_rule,
        }
    }

    /// Evaluates all enabled rules against a single source code string.
    pub fn evaluate_source(
        &self,
        file_path: &str,
        source: &str,
        language: GrammarLanguage,
    ) -> Result<Vec<Diagnostic>, RuleError> {
        let tree = parse(language, source)?;
        let mut diagnostics = Vec::new();

        // E001: Anti-skip
        diagnostics.extend(AntiSkipRule::check_tree(file_path, &tree, source, language));

        // E002: Anti-tautology
        diagnostics.extend(AntiTautologyRule::check_tree(file_path, &tree, source, language));

        // E003: Anti-hollowing
        diagnostics.extend(self.hollowing_rule.evaluate_tree(file_path, &tree, source, language));

        // E004: Anti-mock boundary
        if self.config.enforce_mock_boundary {
            let tier = TestTier::from_path(file_path);
            diagnostics.extend(AntiMockRule::new(tier).evaluate_tree(file_path, &tree, source, language));
        }

        // E005: Import integrity
        if self.config.enforce_import_integrity {
            diagnostics.extend(ImportValidatorRule::check_tree(file_path, &tree, source, language));
        }

        Ok(diagnostics)
    }
}