use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The type of memory being stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    /// A question that was asked
    Question,
    /// A request that was made
    Request,
    /// General information (default)
    #[default]
    Information,
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryType::Question => write!(f, "question"),
            MemoryType::Request => write!(f, "request"),
            MemoryType::Information => write!(f, "information"),
        }
    }
}

impl FromStr for MemoryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "question" => Ok(MemoryType::Question),
            "request" => Ok(MemoryType::Request),
            "information" => Ok(MemoryType::Information),
            _ => Err(format!("Invalid memory type: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_type_default() {
        assert_eq!(MemoryType::default(), MemoryType::Information);
    }

    #[test]
    fn test_memory_type_display() {
        assert_eq!(MemoryType::Question.to_string(), "question");
        assert_eq!(MemoryType::Request.to_string(), "request");
        assert_eq!(MemoryType::Information.to_string(), "information");
    }

    #[test]
    fn test_memory_type_from_str() {
        assert_eq!(
            MemoryType::from_str("question").unwrap(),
            MemoryType::Question
        );
        assert_eq!(
            MemoryType::from_str("REQUEST").unwrap(),
            MemoryType::Request
        );
        assert_eq!(
            MemoryType::from_str("Information").unwrap(),
            MemoryType::Information
        );
        assert!(MemoryType::from_str("invalid").is_err());
    }

    #[test]
    fn test_memory_type_serialization() {
        let json = serde_json::to_string(&MemoryType::Question).unwrap();
        assert_eq!(json, "\"question\"");

        let parsed: MemoryType = serde_json::from_str("\"request\"").unwrap();
        assert_eq!(parsed, MemoryType::Request);
    }
}
