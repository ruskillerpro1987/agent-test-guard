//! S-expression query catalog for test declaration and assertion detection

use thiserror::Error;
use tree_sitter::{Node, Tree};

use crate::grammar::GrammarLanguage;

/// Representation of a discovered test item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredTest {
    pub name: String,
    pub assertion_count: usize,
    pub is_skipped: bool,
    pub is_focused: bool,
    pub suite_name: Option<String>,
}

/// Errors occurring during query execution or test discovery.
#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Tree-sitter query error: {0}")]
    Query(#[from] tree_sitter::QueryError),

    #[error("Language not supported for discovery: {0:?}")]
    UnsupportedLanguage(GrammarLanguage),
}

/// Discovers tests and counts assertions across supported languages.
pub fn discover_tests(
    tree: &Tree,
    source: &str,
    language: GrammarLanguage,
) -> Result<Vec<DiscoveredTest>, QueryError> {
    let mut tests = Vec::new();
    match language {
        GrammarLanguage::Rust => collect_rust_tests(&tree.root_node(), source, &mut tests),
        GrammarLanguage::TypeScript | GrammarLanguage::Tsx | GrammarLanguage::JavaScript => {
            walk_js_node(&tree.root_node(), source, &mut Vec::new(), &mut tests);
        }
    }
    Ok(tests)
}

fn walk_descendants<F: FnMut(&Node)>(node: &Node, f: &mut F) {
    f(node);
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_descendants(&child, f);
    }
}

fn collect_rust_tests(root: &Node, source: &str, tests: &mut Vec<DiscoveredTest>) {
    walk_descendants(root, &mut |node| {
        if node.kind() == "function_item" {
            if let Some(test) = inspect_rust_function(node, source) {
                tests.push(test);
            }
        }
    });
}

fn inspect_rust_function(node: &Node, source: &str) -> Option<DiscoveredTest> {
    let (mut is_test, mut is_skipped) = (false, false);
    let mut prev = node.prev_sibling();
    while let Some(sibling) = prev {
        let kind = sibling.kind();
        if kind == "attribute_item" {
            for i in 0..sibling.child_count() {
                if let Some(name_node) = sibling
                    .child(i)
                    .filter(|a| a.kind() == "attribute")
                    .and_then(|a| a.named_child(0))
                {
                    let name = &source[name_node.byte_range()];
                    if name == "test" || name.ends_with("::test") {
                        is_test = true;
                    } else if name == "ignore" {
                        is_skipped = true;
                    }
                }
            }
            prev = sibling.prev_sibling();
        } else if matches!(kind, "line_comment" | "block_comment" | "comment") {
            prev = sibling.prev_sibling();
        } else {
            break;
        }
    }
    if !is_test {
        return None;
    }

    let name = source[node.child_by_field_name("name")?.byte_range()].to_string();
    let body = node.child_by_field_name("body")?;
    let assertion_count = count_rust_assertions(&body, source);

    Some(DiscoveredTest {
        name,
        assertion_count,
        is_skipped,
        is_focused: false,
        suite_name: None,
    })
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

struct SuiteEntry {
    name: String,
    is_skipped: bool,
    is_focused: bool,
}

enum JsCallKind {
    Describe {
        name: String,
        is_skipped: bool,
        is_focused: bool,
    },
    Test {
        name: String,
        is_skipped: bool,
        is_focused: bool,
    },
}

fn walk_js_node(
    node: &Node,
    source: &str,
    suites: &mut Vec<SuiteEntry>,
    tests: &mut Vec<DiscoveredTest>,
) {
    if node.kind() == "call_expression" {
        match classify_js_call(node, source) {
            Some(JsCallKind::Describe {
                name,
                is_skipped,
                is_focused,
            }) => {
                let parent_skipped = suites.iter().any(|s| s.is_skipped);
                let parent_focused = suites.iter().any(|s| s.is_focused);
                suites.push(SuiteEntry {
                    name,
                    is_skipped: is_skipped || parent_skipped,
                    is_focused: is_focused || parent_focused,
                });
                if let Some(args) = node.child_by_field_name("arguments") {
                    let mut cursor = args.walk();
                    for child in args.children(&mut cursor) {
                        walk_js_node(&child, source, suites, tests);
                    }
                }
                suites.pop();
                return;
            }
            Some(JsCallKind::Test {
                name,
                is_skipped,
                is_focused,
            }) => {
                let parent_skipped = suites.iter().any(|s| s.is_skipped);
                let parent_focused = suites.iter().any(|s| s.is_focused);
                let suite_name = (!suites.is_empty()).then(|| {
                    suites
                        .iter()
                        .map(|s| s.name.as_str())
                        .collect::<Vec<_>>()
                        .join(" > ")
                });
                let assertion_count = count_js_assertions(node, source);
                tests.push(DiscoveredTest {
                    name,
                    assertion_count,
                    is_skipped: is_skipped || parent_skipped,
                    is_focused: is_focused || parent_focused,
                    suite_name,
                });
                return;
            }
            None => {}
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_js_node(&child, source, suites, tests);
    }
}

fn classify_js_call(node: &Node, source: &str) -> Option<JsCallKind> {
    let func = node.child_by_field_name("function")?;
    let args = node.child_by_field_name("arguments")?;
    let name = extract_string_literal(&args.named_child(0)?, source)?;

    match func.kind() {
        "identifier" => match &source[func.byte_range()] {
            "describe" => Some(JsCallKind::Describe {
                name,
                is_skipped: false,
                is_focused: false,
            }),
            "xdescribe" => Some(JsCallKind::Describe {
                name,
                is_skipped: true,
                is_focused: false,
            }),
            "fdescribe" => Some(JsCallKind::Describe {
                name,
                is_skipped: false,
                is_focused: true,
            }),
            "test" | "it" => Some(JsCallKind::Test {
                name,
                is_skipped: false,
                is_focused: false,
            }),
            "xtest" | "xit" => Some(JsCallKind::Test {
                name,
                is_skipped: true,
                is_focused: false,
            }),
            "fit" => Some(JsCallKind::Test {
                name,
                is_skipped: false,
                is_focused: true,
            }),
            _ => None,
        },
        "member_expression" => {
            let obj = &source[func.child_by_field_name("object")?.byte_range()];
            let prop = &source[func.child_by_field_name("property")?.byte_range()];
            match (obj, prop) {
                ("describe", "skip") | ("xdescribe", _) => Some(JsCallKind::Describe {
                    name,
                    is_skipped: true,
                    is_focused: false,
                }),
                ("describe", "only") | ("fdescribe", _) => Some(JsCallKind::Describe {
                    name,
                    is_skipped: false,
                    is_focused: true,
                }),
                ("describe", _) => Some(JsCallKind::Describe {
                    name,
                    is_skipped: false,
                    is_focused: false,
                }),
                ("test" | "it", "skip") | ("xtest" | "xit", _) => Some(JsCallKind::Test {
                    name,
                    is_skipped: true,
                    is_focused: false,
                }),
                ("test" | "it", "only") | ("fit", _) => Some(JsCallKind::Test {
                    name,
                    is_skipped: false,
                    is_focused: true,
                }),
                _ => None,
            }
        }
        _ => None,
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

fn count_js_assertions(test_node: &Node, source: &str) -> usize {
    let mut count = 0;
    if let Some(args) = test_node.child_by_field_name("arguments") {
        let mut cursor = args.walk();
        for child in args.children(&mut cursor) {
            if matches!(
                child.kind(),
                "arrow_function" | "function_expression" | "function"
            ) {
                walk_descendants(&child, &mut |n| {
                    if is_js_assertion_call(n, source) {
                        count += 1;
                    }
                });
            }
        }
    }
    count
}

fn is_js_assertion_call(node: &Node, source: &str) -> bool {
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
            .filter(|obj| obj.kind() == "identifier")
            .is_some_and(|obj| matches!(&source[obj.byte_range()], "assert" | "t")),
        _ => false,
    }
}
