//! AST node kind sequence fingerprinting and BLAKE3 structural hashing

use tree_sitter::Node;
use crate::parser::get_source_text;

/// Computes a structural fingerprint for an AST node using BLAKE3.
/// Traverses named CST node kinds, serializes their kinds
/// (ignoring variable identifiers, but hashing literal values in assertion contexts),
/// and hashes with BLAKE3 returning a 64-char lowercase hex string.
pub fn compute_structural_fingerprint(node: &Node) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_node_structure(node, false, &mut hasher);
    hasher.finalize().to_hex().to_string()
}

fn is_assertion_context(node: &Node) -> bool {
    let kind = node.kind();
    if kind == "macro_invocation" {
        return true;
    }
    if kind == "call_expression" {
        if let Some(func) = node.child_by_field_name("function") {
            let func_kind = func.kind();
            if func_kind == "identifier" {
                if let Some(text) = get_source_text(&func) {
                    let t = text.trim();
                    if t == "assert" || t == "expect" {
                        return true;
                    }
                }
            } else if func_kind == "member_expression" {
                if let Some(text) = get_source_text(&func) {
                    let t = text.trim();
                    if t.starts_with("expect(") || t.contains("expect.") || t.starts_with("assert.") || t.starts_with("t.") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn is_atomic_literal_leaf(node: &Node) -> bool {
    if node.named_child_count() > 0 {
        return false;
    }
    matches!(
        node.kind(),
        "integer_literal"
            | "float_literal"
            | "char_literal"
            | "boolean_literal"
            | "number"
            | "string_content"
            | "string_fragment"
            | "escape_sequence"
            | "true"
            | "false"
    )
}

fn hash_node_structure(node: &Node, in_assertion: bool, hasher: &mut blake3::Hasher) {
    let currently_in_assertion = in_assertion || is_assertion_context(node);

    if node.is_named() {
        let kind = node.kind();

        hasher.update(b"(");
        hasher.update(kind.as_bytes());

        // Подмешиваем значение литерала ТОЛЬКО внутри ассертов
        if currently_in_assertion && is_atomic_literal_leaf(node) {
            if let Some(text) = get_source_text(node) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    hasher.update(b":");
                    hasher.update(trimmed.as_bytes());
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            hash_node_structure(&child, currently_in_assertion, hasher);
        }
        hasher.update(b")");
    } else {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            hash_node_structure(&child, currently_in_assertion, hasher);
        }
    }
}