//! Type definitions for result statistics

use std::fmt;

/// Type of elements in an array
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementType {
    /// Array contains only objects
    Objects,
    /// Array contains only arrays
    Arrays,
    /// Array contains only strings
    Strings,
    /// Array contains only numbers
    Numbers,
    /// Array contains only booleans
    Booleans,
    /// Array contains only nulls
    Nulls,
    /// Array contains mixed types
    Mixed,
    /// Array is empty
    Empty,
}

impl fmt::Display for ElementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElementType::Objects => write!(f, "objects"),
            ElementType::Arrays => write!(f, "arrays"),
            ElementType::Strings => write!(f, "strings"),
            ElementType::Numbers => write!(f, "numbers"),
            ElementType::Booleans => write!(f, "booleans"),
            ElementType::Nulls => write!(f, "nulls"),
            ElementType::Mixed => write!(f, "mixed"),
            ElementType::Empty => write!(f, ""),
        }
    }
}

/// Statistics about a JSON result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultStats {
    /// Array with count and element type
    Array {
        count: usize,
        element_type: ElementType,
    },
    /// Object (no key count - users care more about which keys exist)
    Object,
    /// String value
    String,
    /// Number value
    Number,
    /// Boolean value
    Boolean,
    /// Null value
    Null,
    /// Stream of separate JSON outputs (from jq iteration like .[])
    Stream { count: usize },
}

impl fmt::Display for ResultStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResultStats::Array {
                count,
                element_type,
            } => match element_type {
                ElementType::Empty => write!(f, "Array [0]"),
                _ => write!(f, "Array [{} {}]", count, element_type),
            },
            ResultStats::Object => write!(f, "Object"),
            ResultStats::String => write!(f, "String"),
            ResultStats::Number => write!(f, "Number"),
            ResultStats::Boolean => write!(f, "Boolean"),
            ResultStats::Null => write!(f, "null"),
            ResultStats::Stream { count } => write!(f, "Stream [{}]", count),
        }
    }
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
