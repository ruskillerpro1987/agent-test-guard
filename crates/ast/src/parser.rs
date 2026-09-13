//! Fault-tolerant Concrete Syntax Tree (CST) parser

use thiserror::Error;
use tree_sitter::{Parser, Tree};

use crate::grammar::GrammarLanguage;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Language initialization failed: {0}")]
    LanguageInitialization(#[from] tree_sitter::LanguageError),

    #[error("Tree-sitter parser failed to produce an AST")]
    FailedToParse,
}

pub fn parse(language: GrammarLanguage, source: &str) -> Result<Tree, ParseError> {
    let mut parser = Parser::new();
    parser.set_language(&language.tree_sitter_language())?;
    parser.parse(source, None).ok_or(ParseError::FailedToParse)
}
