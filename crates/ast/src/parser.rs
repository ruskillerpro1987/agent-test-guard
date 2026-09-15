//! Fault-tolerant Concrete Syntax Tree (CST) parser

use std::cell::RefCell;
use std::collections::HashMap;
use thiserror::Error;
use tree_sitter::{Node, Parser, Tree};

use crate::grammar::GrammarLanguage;

thread_local! {
    static SOURCE_REGISTRY: RefCell<HashMap<usize, String>> = RefCell::new(HashMap::new());
}

pub(crate) fn get_source_text(node: &Node) -> Option<String> {
    let mut curr = *node;
    while let Some(parent) = curr.parent() {
        curr = parent;
    }
    let root_id = curr.id();
    let range = node.byte_range();

    SOURCE_REGISTRY.with(|reg| {
        let map = reg.borrow();
        let src = map.get(&root_id)?;
        if range.end <= src.len() {
            Some(src[range].to_string())
        } else {
            None
        }
    })
}

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
    let tree = parser.parse(source, None).ok_or(ParseError::FailedToParse)?;
    
    let root_id = tree.root_node().id();
    SOURCE_REGISTRY.with(|reg| {
        let mut map = reg.borrow_mut();
        if map.len() > 64 {
            map.clear();
        }
        map.insert(root_id, source.to_string());
    });

    Ok(tree)
}