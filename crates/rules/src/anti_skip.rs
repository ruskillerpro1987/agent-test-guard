//! E001: Anti-skip and ignore rule

use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_ast::parser::{parse, ParseError};
use thiserror::Error;
use tree_sitter::{Node, Tree};

use crate::diagnostic::{Diagnostic, RuleCode, Severity, Span};

/// Errors occurring during rule evaluation.
#[derive(Debug, Error)]
pub enum RuleError {
    #[error("AST parsing failed: {0}")]
    Parse(#[from] ParseError),
}

/// E001 rule: Detects skipped, ignored, and exclusively focused tests or suites.
pub struct AntiSkipRule;

impl AntiSkipRule {
    /// Evaluates anti-skip invariants against raw source code.
    pub fn check_source(
        file_path: &str,
        source: &str,
        language: GrammarLanguage,
    ) -> Result<Vec<Diagnostic>, RuleError> {
        let tree = parse(language, source)?;
        Ok(Self::check_tree(file_path, &tree, source, language))
    }

    /// Evaluates anti-skip invariants against a pre-parsed syntax tree.
    pub fn check_tree(
        file_path: &str,
        tree: &Tree,
        source: &str,
        language: GrammarLanguage,
    ) -> Vec<Diagnostic> {
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
        if node.kind() == "function_item" {
            if let Some((test_name, ignore_span)) = inspect_rust_function(node, source) {
                let span = ignore_span.unwrap_or_else(|| Span::from_node(node));
                diagnostics.push(Diagnostic::new(
                    RuleCode::E001,
                    Severity::Error,
                    file_path,
                    span,
                    format!("Test function `{}` has `#[ignore]` attribute", test_name),
                    "Remove `#[ignore]` or resolve the underlying failure instead of skipping.",
                ));
            }
        }
    });
}

fn inspect_rust_function(node: &Node, source: &str) -> Option<(String, Option<Span>)> {
    let mut is_test = false;
    let mut ignore_span = None;

    let mut check_attribute = |attr_item: &Node| {
        for i in 0..attr_item.child_count() {
            let Some(attr_node) = attr_item.child(i).filter(|a| a.kind() == "attribute") else {
                continue;
            };

            if let Some(first) = attr_node.named_child(0) {
                match first.kind() {
                    "identifier" => {
                        let name = &source[first.byte_range()];
                        if name == "test" || name == "rstest" {
                            is_test = true;
                        } else if name == "ignore" {
                            ignore_span = Some(Span::from_node(attr_item));
                        }
                    }
                    "scoped_identifier" => {
                        let name = &source[first.byte_range()];
                        if name.ends_with("::test") {
                            is_test = true;
                        }
                    }
                    _ => {
                        let full_attr = &source[first.byte_range()];
                        if full_attr.starts_with("ignore") {
                            ignore_span = Some(Span::from_node(attr_item));
                        }
                    }
                }
            }
        }
    };

    // Проверяем смежные внешние атрибуты перед функцией с безопасным обрывом
    let mut prev = node.prev_sibling();
    while let Some(sibling) = prev {
        match sibling.kind() {
            "attribute_item" => {
                check_attribute(&sibling);
            }
            "line_comment" | "block_comment" | "comment" => {}
            _ => break,
        }
        prev = sibling.prev_sibling();
    }

    // Проверяем внутренние атрибуты внутри узла функции
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i).filter(|c| c.kind() == "attribute_item") {
            check_attribute(&child);
        }
    }

    if is_test && ignore_span.is_some() {
        let name = node
            .child_by_field_name("name")
            .map(|n| source[n.byte_range()].to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        Some((name, ignore_span))
    } else {
        None
    }
}

fn check_js(file_path: &str, root: &Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    walk_descendants(root, &mut |node| {
        if node.kind() == "call_expression" {
            if let Some(func) = node.child_by_field_name("function") {
                check_js_call(file_path, node, &func, source, diagnostics);
            }
        }
    });
}

fn check_js_call(
    file_path: &str,
    call_node: &Node,
    func_node: &Node,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match func_node.kind() {
        "identifier" => {
            let name = &source[func_node.byte_range()];
            match name {
                "xtest" | "xit" => {
                    diagnostics.push(Diagnostic::new(
                        RuleCode::E001,
                        Severity::Error,
                        file_path,
                        Span::from_node(call_node),
                        format!("Skipped test detected via `{}` prefix alias", name),
                        "Remove `x` prefix to re-enable test execution.",
                    ));
                }
                "xdescribe" => {
                    diagnostics.push(Diagnostic::new(
                        RuleCode::E001,
                        Severity::Error,
                        file_path,
                        Span::from_node(call_node),
                        "Skipped test suite detected via `xdescribe` prefix alias",
                        "Remove `x` prefix to re-enable test suite execution.",
                    ));
                }
                "fit" => {
                    diagnostics.push(Diagnostic::new(
                        RuleCode::E001,
                        Severity::Error,
                        file_path,
                        Span::from_node(call_node),
                        "Focused test detected via `fit` prefix alias",
                        "Remove `f` prefix to execute the entire test suite.",
                    ));
                }
                "fdescribe" => {
                    diagnostics.push(Diagnostic::new(
                        RuleCode::E001,
                        Severity::Error,
                        file_path,
                        Span::from_node(call_node),
                        "Focused test suite detected via `fdescribe` prefix alias",
                        "Remove `f` prefix to execute the entire test suite.",
                    ));
                }
                _ => {}
            }
        }
        "member_expression" => {
            if let Some((is_skip, caller_text)) = inspect_js_member_expression(func_node, source) {
                let (msg, hint) = if is_skip {
                    (
                        format!("Test skipping detected via `{}.skip`", caller_text),
                        "Remove `.skip` or resolve the underlying failure instead of skipping.",
                    )
                } else {
                    (
                        format!("Exclusive test focusing detected via `{}.only`", caller_text),
                        "Remove `.only` to ensure all tests are executed in CI/CD.",
                    )
                };
                diagnostics.push(Diagnostic::new(
                    RuleCode::E001,
                    Severity::Error,
                    file_path,
                    Span::from_node(call_node),
                    msg,
                    hint,
                ));
            }
        }
        _ => {}
    }
}

fn inspect_js_member_expression(node: &Node, source: &str) -> Option<(bool, String)> {
    let mut current = *node;
    let mut found_skip = false;
    let mut found_only = false;
    let mut root_obj = None;

    while current.kind() == "member_expression" {
        if let Some(prop) = current.child_by_field_name("property") {
            let prop_name = &source[prop.byte_range()];
            if prop_name == "skip" {
                found_skip = true;
            } else if prop_name == "only" {
                found_only = true;
            }
        }
        if let Some(obj) = current.child_by_field_name("object") {
            if obj.kind() == "identifier" {
                root_obj = Some(&source[obj.byte_range()]);
                break;
            }
            current = obj;
        } else {
            break;
        }
    }

    if (found_skip || found_only) && root_obj.is_some() {
        let obj_name = root_obj?;
        if matches!(obj_name, "test" | "it" | "describe" | "suite" | "context") {
            return Some((found_skip, obj_name.to_string()));
        }
    }
    None
}