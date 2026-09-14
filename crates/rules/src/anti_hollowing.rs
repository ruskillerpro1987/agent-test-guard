//! E003: Anti-hollowing and assertion floor rule

use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_ast::parser::parse;
use tree_sitter::{Node, Tree};

use crate::anti_skip::RuleError;
use crate::diagnostic::{Diagnostic, RuleCode, Severity, Span};

/// E003 rule: Detects hollowed test bodies and tests with assertion counts below the floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AntiHollowingRule {
    min_assertions: usize,
}

impl Default for AntiHollowingRule {
    fn default() -> Self {
        Self { min_assertions: 1 }
    }
}

impl AntiHollowingRule {
    pub const DEFAULT_ASSERTION_FLOOR: usize = 1;

    pub const fn new(min_assertions: usize) -> Self {
        Self { min_assertions }
    }

    pub const fn min_assertions(&self) -> usize {
        self.min_assertions
    }

    pub fn check_source(file: &str, src: &str, lang: GrammarLanguage) -> Result<Vec<Diagnostic>, RuleError> {
        Self::default().evaluate_source(file, src, lang)
    }

    pub fn check_tree(file: &str, tree: &Tree, src: &str, lang: GrammarLanguage) -> Vec<Diagnostic> {
        Self::default().evaluate_tree(file, tree, src, lang)
    }

    pub fn evaluate_source(&self, file: &str, src: &str, lang: GrammarLanguage) -> Result<Vec<Diagnostic>, RuleError> {
        Ok(self.evaluate_tree(file, &parse(lang, src)?, src, lang))
    }

    pub fn evaluate_tree(&self, file: &str, tree: &Tree, src: &str, lang: GrammarLanguage) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let root = tree.root_node();
        match lang {
            GrammarLanguage::Rust => check_rust(file, &root, src, self.min_assertions, &mut diags),
            GrammarLanguage::TypeScript | GrammarLanguage::Tsx | GrammarLanguage::JavaScript => {
                check_js(file, &root, src, self.min_assertions, &mut diags);
            }
        }
        diags
    }
}

fn walk_descendants<F: FnMut(&Node)>(node: &Node, f: &mut F) {
    f(node);
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_descendants(&child, f);
    }
}

fn check_rust(file: &str, root: &Node, src: &str, floor: usize, diags: &mut Vec<Diagnostic>) {
    walk_descendants(root, &mut |node| {
        if node.kind() == "function_item" {
            if let Some((name, span, assertions)) = inspect_rust_test(node, src) {
                if assertions < floor {
                    diags.push(create_diagnostic(file, span, &name, assertions, floor));
                }
            }
        }
    });
}

fn inspect_rust_test(node: &Node, source: &str) -> Option<(String, Span, usize)> {
    let mut is_test = false;
    let mut check_attr = |attr_item: &Node| {
        for i in 0..attr_item.child_count() {
            if attr_item.child(i).filter(|a| a.kind() == "attribute").is_some_and(|a| is_rust_test_attr(&a, source)) {
                is_test = true;
            }
        }
    };
    let mut prev = node.prev_sibling();
    while let Some(sibling) = prev {
        match sibling.kind() {
            "attribute_item" => check_attr(&sibling),
            "line_comment" | "block_comment" | "comment" => {}
            _ => break,
        }
        prev = sibling.prev_sibling();
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i).filter(|c| c.kind() == "attribute_item") {
            check_attr(&child);
        }
    }
    if !is_test {
        return None;
    }
    let name = node.child_by_field_name("name").map(|n| source[n.byte_range()].to_string()).unwrap_or_else(|| "anonymous".to_string());
    let assertions = node.child_by_field_name("body").map(|b| count_rust_assertions(&b, source)).unwrap_or(0);
    Some((name, Span::from_node(node), assertions))
}

fn is_rust_test_attr(attr: &Node, source: &str) -> bool {
    let Some(first) = attr.named_child(0) else {
        return false;
    };
    let target = match first.kind() {
        "call_expression" => first.child_by_field_name("function").unwrap_or(first),
        _ => first,
    };
    let name = &source[target.byte_range()];
    name == "test" || name == "rstest" || name.ends_with("::test") || name == "test_case"
}

fn count_rust_assertions(body: &Node, source: &str) -> usize {
    let mut count = 0;
    walk_descendants(body, &mut |node| {
        if node.kind() == "macro_invocation" {
            if let Some(macro_node) = node.child_by_field_name("macro") {
                let name = &source[macro_node.byte_range()];
                let base = name.rsplit("::").next().unwrap_or(name);
                if base.starts_with("assert") || base.starts_with("debug_assert") {
                    count += 1;
                }
            }
        }
    });
    count
}

fn check_js(file: &str, root: &Node, src: &str, floor: usize, diags: &mut Vec<Diagnostic>) {
    walk_descendants(root, &mut |node| {
        if node.kind() == "call_expression" {
            if let Some((name, span, assertions)) = inspect_js_test(node, src) {
                if assertions < floor {
                    diags.push(create_diagnostic(file, span, &name, assertions, floor));
                }
            }
        }
    });
}

fn inspect_js_test(call: &Node, source: &str) -> Option<(String, Span, usize)> {
    let func = call.child_by_field_name("function")?;
    let args = call.child_by_field_name("arguments")?;
    let is_test = match func.kind() {
        "identifier" => matches!(&source[func.byte_range()], "test" | "it" | "xtest" | "xit" | "fit"),
        "member_expression" => func
            .child_by_field_name("object")
            .filter(|o| o.kind() == "identifier")
            .is_some_and(|o| matches!(&source[o.byte_range()], "test" | "it")),
        _ => false,
    };
    if !is_test {
        return None;
    }
    let name = args
        .named_child(0)
        .and_then(|n| extract_string_literal(&n, source))
        .unwrap_or_else(|| "unnamed_test".to_string());
    Some((name, Span::from_node(call), count_js_assertions(call, source)))
}

fn count_js_assertions(test_node: &Node, source: &str) -> usize {
    let mut count = 0;
    if let Some(args) = test_node.child_by_field_name("arguments") {
        for i in 0..args.named_child_count() {
            if let Some(child) = args.named_child(i).filter(|c| {
                matches!(c.kind(), "arrow_function" | "function_expression" | "function")
            }) {
                walk_descendants(&child, &mut |n| {
                    if is_js_assertion(n, source) {
                        count += 1;
                    }
                });
            }
        }
    }
    count
}

fn is_js_assertion(node: &Node, source: &str) -> bool {
    let Some(func) = (node.kind() == "call_expression")
        .then(|| node.child_by_field_name("function"))
        .flatten()
    else {
        return false;
    };
    match func.kind() {
        "identifier" => matches!(&source[func.byte_range()], "expect" | "assert"),
        "member_expression" => func
            .child_by_field_name("object")
            .filter(|o| o.kind() == "identifier")
            .is_some_and(|o| matches!(&source[o.byte_range()], "assert" | "t")),
        _ => false,
    }
}

fn extract_string_literal(node: &Node, source: &str) -> Option<String> {
    let text = source[node.byte_range()].trim();
    if text.len() >= 2 && matches!(text.as_bytes()[0], b'"' | b'\'' | b'`') {
        return Some(text[1..text.len() - 1].to_string());
    }
    (0..node.child_count())
        .find_map(|i| node.child(i).filter(|c| c.kind() == "string_fragment"))
        .map(|c| source[c.byte_range()].to_string())
        .or_else(|| Some(text.to_string()))
}

fn create_diagnostic(file: &str, span: Span, name: &str, assertions: usize, floor: usize) -> Diagnostic {
    let (c_lbl, m_lbl) = (
        if assertions == 1 { "assertion" } else { "assertions" },
        if floor == 1 { "assertion" } else { "assertions" },
    );
    Diagnostic::new(
        RuleCode::E003,
        Severity::Error,
        file,
        span,
        format!("Test `{name}` is hollowed: contains {assertions} {c_lbl} (minimum required: {floor} {m_lbl})"),
        format!(
            "Add at least {} verified assertion(s) (e.g. `expect()`, `assert!`) verifying expected behavior.",
            floor.saturating_sub(assertions).max(1)
        ),
    )
}
