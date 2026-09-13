//! AST node kind sequence fingerprinting and BLAKE3 structural hashing

use tree_sitter::Node;

/// Computes a structural fingerprint for an AST node using BLAKE3.
/// Traverses named CST node kinds, serializes their kinds
/// (ignoring identifier text, literal values, and whitespace),
/// and hashes with BLAKE3 returning a 64-char lowercase hex string.
pub fn compute_structural_fingerprint(node: &Node) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_node_structure(node, &mut hasher);
    hasher.finalize().to_hex().to_string()
}

fn hash_node_structure(node: &Node, hasher: &mut blake3::Hasher) {
    if node.is_named() {
        hasher.update(b"(");
        hasher.update(node.kind().as_bytes());
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            hash_node_structure(&child, hasher);
        }
        hasher.update(b")");
    } else {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            hash_node_structure(&child, hasher);
        }
    }
}
