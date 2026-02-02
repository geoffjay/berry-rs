use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Visibility level for a memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum VisibilityLevel {
    /// Only visible to the creator
    Private,
    /// Visible to specific shared actors
    Shared,
    /// Visible to everyone (default)
    #[default]
    Public,
}

impl fmt::Display for VisibilityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VisibilityLevel::Private => write!(f, "private"),
            VisibilityLevel::Shared => write!(f, "shared"),
            VisibilityLevel::Public => write!(f, "public"),
        }
    }
}

impl FromStr for VisibilityLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "private" => Ok(VisibilityLevel::Private),
            "shared" => Ok(VisibilityLevel::Shared),
            "public" => Ok(VisibilityLevel::Public),
            _ => Err(format!("Invalid visibility level: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_default() {
        assert_eq!(VisibilityLevel::default(), VisibilityLevel::Public);
    }

    #[test]
    fn test_visibility_display() {
        assert_eq!(VisibilityLevel::Private.to_string(), "private");
        assert_eq!(VisibilityLevel::Shared.to_string(), "shared");
        assert_eq!(VisibilityLevel::Public.to_string(), "public");
    }

    #[test]
    fn test_visibility_from_str() {
        assert_eq!(
            VisibilityLevel::from_str("private").unwrap(),
            VisibilityLevel::Private
        );
        assert_eq!(
            VisibilityLevel::from_str("SHARED").unwrap(),
            VisibilityLevel::Shared
        );
        assert_eq!(
            VisibilityLevel::from_str("Public").unwrap(),
            VisibilityLevel::Public
        );
        assert!(VisibilityLevel::from_str("invalid").is_err());
    }

    #[test]
    fn test_visibility_serialization() {
        let json = serde_json::to_string(&VisibilityLevel::Private).unwrap();
        assert_eq!(json, "\"private\"");

        let parsed: VisibilityLevel = serde_json::from_str("\"shared\"").unwrap();
        assert_eq!(parsed, VisibilityLevel::Shared);
    }
}
