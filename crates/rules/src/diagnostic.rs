//! Diagnostic data structures and error codes (E001..E010)

use std::fmt;
use serde::{Deserialize, Serialize};

/// Diagnostic violation rule codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleCode {
    /// E001: Test skipping, ignoring, or focusing detected
    #[serde(rename = "E001")]
    E001,
    /// E002: Tautological assertion detected
    #[serde(rename = "E002")]
    E002,
    /// E003: Test body assertion floor violated or hollowed
    #[serde(rename = "E003")]
    E003,
    /// E004: Prohibited mock boundary leak detected
    #[serde(rename = "E004")]
    E004,
    /// E005: Source import integrity violated
    #[serde(rename = "E005")]
    E005,
    /// E010: Ratchet invariant or baseline test count violated
    #[serde(rename = "E010")]
    E010,
}

impl RuleCode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::E001 => "E001",
            Self::E002 => "E002",
            Self::E003 => "E003",
            Self::E004 => "E004",
            Self::E005 => "E005",
            Self::E010 => "E010",
        }
    }
}

impl fmt::Display for RuleCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Severity level of diagnostic violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
        }
    }
}

/// Source text location span (1-indexed line and column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl Span {
    pub const fn new(
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
        start_byte: usize,
        end_byte: usize,
    ) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
            start_byte,
            end_byte,
        }
    }

    pub fn from_node(node: &tree_sitter::Node<'_>) -> Self {
        let start = node.start_position();
        let end = node.end_position();
        Self {
            start_line: start.row + 1,
            start_col: start.column + 1,
            end_line: end.row + 1,
            end_col: end.column + 1,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
        }
    }
}

/// Machine-readable Diagnostic Card for AI agents and developer tooling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: RuleCode,
    pub severity: Severity,
    pub file_path: String,
    pub span: Span,
    pub message: String,
    pub fix_hint: String,
}

impl Diagnostic {
    pub fn new(
        code: RuleCode,
        severity: Severity,
        file_path: impl Into<String>,
        span: Span,
        message: impl Into<String>,
        fix_hint: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity,
            file_path: file_path.into(),
            span,
            message: message.into(),
            fix_hint: fix_hint.into(),
        }
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}:{}:{}: {} - {}",
            self.code, self.file_path, self.span.start_line, self.span.start_col, self.message, self.fix_hint
        )
    }
}
