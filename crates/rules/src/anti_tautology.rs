//! E002: Anti-tautology assertion rule

use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_ast::parser::parse;
use tree_sitter::{Node, Tree};

use crate::anti_skip::RuleError;
use crate::diagnostic::{Diagnostic, RuleCode, Severity, Span};

/// E002 rule: Detects tautological assertions in test bodies.
pub struct AntiTautologyRule;

impl AntiTautologyRule {
    /// Evaluates anti-tautology invariants against raw source code.
    pub fn check_source(
        file_path: &str,
        source: &str,
        language: GrammarLanguage,
    ) -> Result<Vec<Diagnostic>, RuleError> {
        let tree = parse(language, source)?;
        Ok(Self::check_tree(file_path, &tree, source, language))
    }

    /// Evaluates anti-tautology invariants against a pre-parsed syntax tree.
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
            _ => check_js(file_path, &root, source, &mut diagnostics),
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

fn make_diag(file_path: &str, span: Span, expr_text: &str) -> Diagnostic {
    Diagnostic::new(
        RuleCode::E002, Severity::Error, file_path, span,
        format!("Tautological assertion detected: `{}`", expr_text.trim()),
        "Assert actual dynamic values and state changes instead of invariant tautologies.",
    )
}

fn check_rust(file_path: &str, root: &Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    walk_descendants(root, &mut |node| {
        if node.kind() == "macro_invocation" {
            if let Some(diag) = inspect_rust_macro(file_path, node, source) {
                diagnostics.push(diag);
            }
        }
    });
}

fn inspect_rust_macro(file_path: &str, node: &Node, source: &str) -> Option<Diagnostic> {
    let macro_node = node.child_by_field_name("macro")?;
    let name = &source[macro_node.byte_range()];
    let base = name.rsplit("::").next().unwrap_or(name);

    let token_tree = node.children(&mut node.walk()).find(|c| c.kind() == "token_tree")?;
    let raw = &source[token_tree.byte_range()];
    if raw.len() < 2 {
        return None;
    }
    let args = split_top_level_args(raw[1..raw.len() - 1].trim());
    let span = Span::from_node(node);
    let macro_text = &source[node.byte_range()];

    let is_taut = match base {
        "assert" | "debug_assert" if !args.is_empty() => {
            let f = args[0].trim();
            f == "true" || f == "!false" || is_binary_tautology(f)
        }
        "assert_eq" | "debug_assert_eq" if args.len() >= 2 => {
            let (l, r) = (args[0].trim(), args[1].trim());
            l == r
        }
        "assert_ne" | "debug_assert_ne" if args.len() >= 2 => {
            let (l, r) = (args[0].trim(), args[1].trim());
            is_constant_literal(l) && is_constant_literal(r) && l != r
        }
        _ => false,
    };
    is_taut.then(|| make_diag(file_path, span, macro_text))
}

fn check_js(file_path: &str, root: &Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    walk_descendants(root, &mut |node| {
        if node.kind() == "call_expression" {
            if let Some(diag) = inspect_js_call(file_path, node, source) {
                diagnostics.push(diag);
            }
        }
    });
}

fn inspect_js_call(file_path: &str, node: &Node, source: &str) -> Option<Diagnostic> {
    let func = node.child_by_field_name("function")?;
    let span = Span::from_node(node);
    let text = &source[node.byte_range()];

    if func.kind() == "identifier" && &source[func.byte_range()] == "assert" {
        let args = node.child_by_field_name("arguments")?;
        let first = source[args.named_child(0)?.byte_range()].trim();
        return (first == "true" || is_binary_tautology(first))
            .then(|| make_diag(file_path, span, text));
    }

    if func.kind() == "member_expression" {
        let obj = func.child_by_field_name("object")?;
        let prop = func.child_by_field_name("property")?;

        if obj.kind() == "identifier" && matches!(&source[obj.byte_range()], "assert" | "t") {
            let method = &source[prop.byte_range()];
            let args = node.child_by_field_name("arguments")?;
            let taut = match method {
                "ok" | "true" => {
                    let f = source[args.named_child(0)?.byte_range()].trim();
                    f == "true" || is_binary_tautology(f)
                }
                "equal" | "strictEqual" | "deepEqual" | "is" => {
                    let l = source[args.named_child(0)?.byte_range()].trim();
                    let r = source[args.named_child(1)?.byte_range()].trim();
                    l == r
                }
                _ => false,
            };
            return taut.then(|| make_diag(file_path, span, text));
        }

        let matcher = &source[prop.byte_range()];
        let (exp, inv) = if obj.kind() == "call_expression" {
            (obj, false)
        } else if obj.kind() == "member_expression" {
            let p = obj.child_by_field_name("property")?;
            let o = obj.child_by_field_name("object")?;
            if &source[p.byte_range()] == "not" && o.kind() == "call_expression" {
                (o, true)
            } else {
                return None;
            }
        } else {
            return None;
        };

        let exp_fn = exp.child_by_field_name("function")?;
        if &source[exp_fn.byte_range()] == "expect" {
            let tgt = source[exp.child_by_field_name("arguments")?.named_child(0)?.byte_range()].trim();
            let m_arg = node.child_by_field_name("arguments")?.named_child(0).map(|a| source[a.byte_range()].trim());
            let taut = if !inv {
                (matches!(matcher, "toBe" | "toEqual" | "toStrictEqual" | "toMatchObject") && m_arg == Some(tgt))
                    || match matcher {
                        "toBeTruthy" => tgt == "true",
                        "toBeFalsy" => tgt == "false",
                        "toBeNull" => tgt == "null",
                        "toBeUndefined" => tgt == "undefined",
                        "toBeNaN" => tgt == "NaN",
                        "toBeDefined" => is_constant_literal(tgt),
                        _ => false,
                    }
            } else {
                matches!(matcher, "toBe" | "toEqual" | "toStrictEqual")
                    && m_arg.is_some_and(|a| is_constant_literal(tgt) && is_constant_literal(a) && tgt != a)
            };
            if taut {
                return Some(make_diag(file_path, span, text));
            }
        }
    }
    None
}

fn split_top_level_args(s: &str) -> Vec<&str> {
    let (mut args, mut depth, mut in_q, mut prev, mut start) = (Vec::new(), 0usize, None, '\0', 0);
    for (i, c) in s.char_indices() {
        if let Some(q) = in_q {
            if c == q && prev != '\\' { in_q = None; }
        } else {
            match c {
                '"' | '\'' | '`' => in_q = Some(c),
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth = depth.saturating_sub(1),
                ',' if depth == 0 => { args.push(&s[start..i]); start = i + 1; }
                _ => {}
            }
        }
        prev = c;
    }
    if start < s.len() { args.push(&s[start..]); }
    args
}

fn is_constant_literal(s: &str) -> bool {
    let s = s.trim();
    if matches!(s, "true" | "false" | "null" | "undefined" | "NaN") {
        return true;
    }
    if s.len() >= 2
        && matches!(s.as_bytes()[0], b'"' | b'\'' | b'`')
        && s.as_bytes()[s.len() - 1] == s.as_bytes()[0]
    {
        return true;
    }
    let digits = s.strip_prefix('-').unwrap_or(s);
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '_')
}

fn is_binary_tautology(expr: &str) -> bool {
    let Some((l, r, is_eq)) = find_binary_op(expr).map(|(l, r, eq)| (l.trim(), r.trim(), eq)) else {
        return false;
    };
    if is_eq {
        l == r
    } else {
        is_constant_literal(l) && is_constant_literal(r) && l != r
    }
}

fn find_binary_op(s: &str) -> Option<(&str, &str, bool)> {
    let (mut in_q, mut prev, bytes, len) = (None, '\0', s.as_bytes(), s.len());
    let mut i = 0;
    while i < len {
        let c = bytes[i] as char;
        if let Some(q) = in_q {
            if c == q && prev != '\\' { in_q = None; }
        } else if matches!(c, '"' | '\'' | '`') {
            in_q = Some(c);
        } else if i + 3 <= len && (&s[i..i + 3] == "===" || &s[i..i + 3] == "!==") {
            return Some((&s[..i], &s[i + 3..], &s[i..i + 3] == "==="));
        } else if i + 2 <= len && (&s[i..i + 2] == "==" || &s[i..i + 2] == "!=") {
            return Some((&s[..i], &s[i + 2..], &s[i..i + 2] == "=="));
        }
        prev = c;
        i += 1;
    }
    None
}