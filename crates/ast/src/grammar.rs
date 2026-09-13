//! Supported language enumerations and Tree-sitter grammar loaders

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrammarLanguage {
    Rust,
    TypeScript,
    Tsx,
    JavaScript,
}

impl GrammarLanguage {
    pub fn tree_sitter_language(&self) -> tree_sitter::Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::TypeScript | Self::JavaScript => {
                tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
            }
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        }
    }
}

pub fn detect_language(path: impl AsRef<Path>) -> Option<GrammarLanguage> {
    let path = path.as_ref();
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some(GrammarLanguage::Rust),
        "ts" | "mts" | "cts" => Some(GrammarLanguage::TypeScript),
        "tsx" => Some(GrammarLanguage::Tsx),
        "js" | "mjs" | "cjs" | "jsx" => Some(GrammarLanguage::JavaScript),
        _ => None,
    }
}
