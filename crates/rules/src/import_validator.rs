//! E005: Source file import integrity validator

use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_ast::parser::parse;
use tree_sitter::{Node, Tree};

use crate::anti_skip::RuleError;
use crate::diagnostic::{Diagnostic, RuleCode, Severity, Span};

/// E005 rule: Validates that test suites import actual production source modules
/// and do not import themselves or other test fixtures circularly.
#[derive(Debug, Default, Clone, Copy)]
pub struct ImportValidatorRule;

impl ImportValidatorRule {
    pub fn check_source(
        file_path: &str,
        source: &str,
        language: GrammarLanguage,
    ) -> Result<Vec<Diagnostic>, RuleError> {
        let tree = parse(language, source)?;
        Ok(Self::check_tree(file_path, &tree, source, language))
    }

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
    // В integration тестах (tests/*.rs) обязательно должен быть импорт или ссылка на crate / super / свой модуль
    let normalized = file_path.replace('\\', "/");
    let is_external_test = normalized.starts_with("tests/") || normalized.contains("/tests/");

    let mut imports = Vec::new();

    walk_descendants(root, &mut |node| {
        if node.kind() == "use_declaration" {
            if let Some(arg) = node.child_by_field_name("argument") {
                let text = &source[arg.byte_range()];
                imports.push((text.to_string(), Span::from_node(node)));
            }
        }
    });

    let file_stem = std::path::Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    for (import_path, span) in &imports {
        // Проверка на self-import: tests/auth_test.rs -> use tests::auth_test;
        if !file_stem.is_empty() && import_path.contains(file_stem) {
            diagnostics.push(Diagnostic::new(
                RuleCode::E005,
                Severity::Error,
                file_path,
                *span,
                format!("Circular or self-import detected: `{import_path}`"),
                "Tests must import production modules, not themselves.",
            ));
        }
    }

    // Если это внешний интеграционный тест, но нет ни одного `use`, поднимаем диагностику
    if is_external_test && imports.is_empty() {
        diagnostics.push(Diagnostic::new(
            RuleCode::E005,
            Severity::Warning,
            file_path,
            Span::from_node(root),
            "Test file does not import any production crate or module",
            "Ensure the test imports and exercises actual production code.",
        ));
    }
}

fn check_js(file_path: &str, root: &Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut imports = Vec::new();
    let current_stem = std::path::Path::new(file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .split('.')
        .next()
        .unwrap_or("");

    walk_descendants(root, &mut |node| {
        match node.kind() {
            "import_statement" => {
                if let Some(src_node) = node.child_by_field_name("source") {
                    let raw = &source[src_node.byte_range()];
                    let path = raw.trim_matches(|c| c == '\'' || c == '"' || c == '`');
                    imports.push((path.to_string(), Span::from_node(node)));
                }
            }
            "call_expression" => {
                if let Some(func) = node.child_by_field_name("function") {
                    if func.kind() == "identifier" && &source[func.byte_range()] == "require" {
                        if let Some(args) = node.child_by_field_name("arguments") {
                            if let Some(first) = args.named_child(0) {
                                let raw = &source[first.byte_range()];
                                let path = raw.trim_matches(|c| c == '\'' || c == '"' || c == '`');
                                imports.push((path.to_string(), Span::from_node(node)));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    });

    let mut has_product_import = false;

    for (path, span) in &imports {
        // Проверка на импорт другого тест-файла или самого себя: import './auth.test'
        if path.contains(".test") || path.contains(".spec") || (!current_stem.is_empty() && path.ends_with(current_stem)) {
            diagnostics.push(Diagnostic::new(
                RuleCode::E005,
                Severity::Error,
                file_path,
                *span,
                format!("Prohibited test-on-test or self-import detected: `{path}`"),
                "Tests must import production modules, not test files or fixtures.",
            ));
        }

        // Проверяем, что есть хоть один импорт не из тестовых раннеров (vitest, jest, chai)
        if !matches!(
            path.as_str(),
            "vitest" | "jest" | "@jest/globals" | "chai" | "mocha" | "ava" | "supertest" | "node:test"
        ) {
            has_product_import = true;
        }
    }

    if !imports.is_empty() && !has_product_import {
        diagnostics.push(Diagnostic::new(
            RuleCode::E005,
            Severity::Warning,
            file_path,
            Span::from_node(root),
            "Test suite imports only testing frameworks without production code linkages",
            "Import and exercise production application modules.",
        ));
    }
}