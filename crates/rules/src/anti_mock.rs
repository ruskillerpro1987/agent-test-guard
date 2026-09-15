//! E004: 3-Tier mock boundary rule (Tier 3 E2E quarantine)

use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_ast::parser::parse;
use tree_sitter::{Node, Tree};

use crate::anti_skip::RuleError;
use crate::diagnostic::{Diagnostic, RuleCode, Severity, Span};

/// Tier classification for test files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestTier {
    /// Unit tests: mocking allowed.
    Tier1Unit,
    /// Integration tests: partial mocking allowed.
    Tier2Integration,
    /// End-to-end / quarantine tests: mocking strictly prohibited.
    Tier3E2E,
}

impl TestTier {
    /// Infers test tier from file path semantics.
    pub fn from_path(path: &str) -> Self {
        let normalized = path.replace('\\', "/").to_lowercase();
        if normalized.starts_with("e2e/")
            || normalized.contains("/e2e/")
            || normalized.contains(".e2e.")
            || normalized.starts_with("tests/e2e")
            || normalized.contains("/tests/e2e")
            || normalized.contains("quarantine")
        {
            Self::Tier3E2E
        } else if normalized.starts_with("integration/")
            || normalized.contains("/integration/")
            || normalized.contains(".integration.")
            || normalized.starts_with("tests/integration")
            || normalized.contains("/tests/integration")
        {
            Self::Tier2Integration
        } else {
            Self::Tier1Unit
        }
    }
}

/// E004 rule: Detects forbidden mocking patterns inside Tier 3 (E2E) test files.
pub struct AntiMockRule {
    tier: TestTier,
}

impl Default for AntiMockRule {
    fn default() -> Self {
        Self {
            tier: TestTier::Tier3E2E,
        }
    }
}

impl AntiMockRule {
    pub const fn new(tier: TestTier) -> Self {
        Self { tier }
    }

    /// Checks raw source code, automatically deriving the tier from file path.
    pub fn check_source(
        file_path: &str,
        source: &str,
        language: GrammarLanguage,
    ) -> Result<Vec<Diagnostic>, RuleError> {
        let tier = TestTier::from_path(file_path);
        Self::new(tier).evaluate_source(file_path, source, language)
    }

    /// Evaluates raw source code under explicit tier configuration.
    pub fn evaluate_source(
        &self,
        file_path: &str,
        source: &str,
        language: GrammarLanguage,
    ) -> Result<Vec<Diagnostic>, RuleError> {
        let tree = parse(language, source)?;
        Ok(self.evaluate_tree(file_path, &tree, source, language))
    }

    /// Evaluates AST tree against mocking invariants.
    pub fn evaluate_tree(
        &self,
        file_path: &str,
        tree: &Tree,
        source: &str,
        language: GrammarLanguage,
    ) -> Vec<Diagnostic> {
        // Mocking restrictions are strictly enforced in Tier 3 (E2E)
        if self.tier != TestTier::Tier3E2E {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        let root = tree.root_node();
        match language {
            GrammarLanguage::Rust => check_rust(file_path, &root, source, &mut diagnostics),
            GrammarLanguage::TypeScript | GrammarLanguage::Tsx | GrammarLanguage::JavaScript => {
                check_js(file_path, &root, source, &mut diagnostics);
            }
        }
        diagnostics
    }
}

fn walk_descendants<F: FnMut(&Node)>(node: &Node, f: &mut F) {
    f(node);
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_descendants(&child, f);
    }
}

fn check_rust(file_path: &str, root: &Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    walk_descendants(root, &mut |node| {
        match node.kind() {
            // Rust: #[automock] / #[mockall::automock] attributes
            "attribute_item" => {
                for i in 0..node.child_count() {
                    if let Some(attr) = node.child(i).filter(|a| a.kind() == "attribute") {
                        if let Some(name_node) = attr.named_child(0) {
                            let text = &source[name_node.byte_range()];
                            if text == "automock"
                                || text.ends_with("::automock")
                                || text.starts_with("mock")
                            {
                                diagnostics.push(Diagnostic::new(
                                    RuleCode::E004,
                                    Severity::Error,
                                    file_path,
                                    Span::from_node(node),
                                    format!("Prohibited mock attribute `#[{text}]` in Tier 3 E2E test"),
                                    "E2E tests must verify real service integration. Remove mock attribute or move test to Tier 1.",
                                ));
                            }
                        }
                    }
                }
            }
            // Rust: mockall::mock! / mock! macro invocations
            "macro_invocation" => {
                if let Some(macro_node) = node.child_by_field_name("macro") {
                    let text = &source[macro_node.byte_range()];
                    let base = text.rsplit("::").next().unwrap_or(text);
                    if base == "mock" || base.starts_with("mock_") {
                        diagnostics.push(Diagnostic::new(
                            RuleCode::E004,
                            Severity::Error,
                            file_path,
                            Span::from_node(node),
                            format!("Prohibited mock macro invocation `{text}!` in Tier 3 E2E test"),
                            "Replace synthetic mock macro with live test container or move test to Tier 1/2.",
                        ));
                    }
                }
            }
            _ => {}
        }
    });
}

fn check_js(file_path: &str, root: &Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    walk_descendants(root, &mut |node| {
        // JS/TS: Import from mocking frameworks (sinon, nock, msw, wiremock)
        if node.kind() == "import_statement" {
            if let Some(source_node) = node.child_by_field_name("source") {
                let raw_text = &source[source_node.byte_range()].trim();
                let text = raw_text.trim_matches(|c: char| c == '\'' || c == '"' || c == '`');
                if matches!(text, "sinon" | "nock" | "supertest" | "msw" | "msw/node") {
                    diagnostics.push(Diagnostic::new(
                        RuleCode::E004,
                        Severity::Error,
                        file_path,
                        Span::from_node(node),
                        format!("Prohibited mock library import `{text}` in Tier 3 E2E test"),
                        "End-to-end tests must execute against real services without mock interception.",
                    ));
                }
            }
        }

        // JS/TS: jest.mock(), vi.mock(), cy.intercept(), page.route()
        if node.kind() == "call_expression" {
            if let Some(func) = node.child_by_field_name("function") {
                if func.kind() == "member_expression" {
                    if let (Some(obj), Some(prop)) = (
                        func.child_by_field_name("object"),
                        func.child_by_field_name("property"),
                    ) {
                        let obj_name = &source[obj.byte_range()];
                        let prop_name = &source[prop.byte_range()];

                        let is_mock_call = match (obj_name, prop_name) {
                            ("jest" | "vi", "mock" | "spyOn" | "fn" | "doMock") => true,
                            ("cy", "intercept") => true,
                            ("page", "route" | "routeFromHAR") => true,
                            _ => false,
                        };

                        if is_mock_call {
                            diagnostics.push(Diagnostic::new(
                                RuleCode::E004,
                                Severity::Error,
                                file_path,
                                Span::from_node(node),
                                format!("Prohibited mock call `{obj_name}.{prop_name}()` in Tier 3 E2E test"),
                                "Remove mocking call in Tier 3 suite or relocate unit test into Tier 1 suite.",
                            ));
                        }
                    }
                }
            }
        }
    });
}