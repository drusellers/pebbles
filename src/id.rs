use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

const ALPHANUMERIC: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Id(String);

impl Id {
    pub fn new(id: impl Into<String>) -> Result<Self, IdError> {
        let id = id.into();
        if id.is_empty() {
            return Err(IdError::Empty);
        }

        // Validate that all characters are alphanumeric lowercase
        for c in id.chars() {
            if !ALPHANUMERIC.contains(c) {
                return Err(IdError::InvalidCharacter(c));
            }
        }

        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn to_lowercase(&self) -> String {
        self.0.to_lowercase()
    }

    pub fn generate() -> Self {
        use rand::{Rng, thread_rng};
        let mut rng = thread_rng();
        let id: String = (0..4)
            .map(|_| {
                let idx = rng.gen_range(0..ALPHANUMERIC.len());
                ALPHANUMERIC.chars().nth(idx).unwrap()
            })
            .collect();
        Self(id)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Id {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for Id {
    type Error = IdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Id {
    type Error = IdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdError {
    Empty,
    InvalidCharacter(char),
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdError::Empty => write!(f, "ID cannot be empty"),
            IdError::InvalidCharacter(c) => write!(f, "Invalid character '{}' in ID", c),
        }
    }
}

impl std::error::Error for IdError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_new_valid() {
        let id = Id::new("abc1").unwrap();
        assert_eq!(id.as_str(), "abc1");
    }

    #[test]
    fn test_id_new_empty() {
        let result = Id::new("");
        assert!(matches!(result, Err(IdError::Empty)));
    }

    #[test]
    fn test_id_new_invalid_char() {
        let result = Id::new("abc!");
        assert!(matches!(result, Err(IdError::InvalidCharacter('!'))));
    }

    #[test]
    fn test_id_new_uppercase() {
        let result = Id::new("ABC1");
        assert!(matches!(result, Err(IdError::InvalidCharacter('A'))));
    }

    #[test]
    fn test_id_display() {
        let id = Id::new("abc1").unwrap();
        assert_eq!(format!("{}", id), "abc1");
    }

    #[test]
    fn test_id_from_str() {
        let id: Id = "abc1".parse().unwrap();
        assert_eq!(id.as_str(), "abc1");
    }

    #[test]
    fn test_id_generate() {
        let id = Id::generate();
        assert_eq!(id.as_str().len(), 4);
        // Verify all characters are valid
        for c in id.as_str().chars() {
            assert!(ALPHANUMERIC.contains(c));
        }
    }

    #[test]
    fn test_id_serialize() {
        let id = Id::new("abc1").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"abc1\"");
    }

    #[test]
    fn test_id_deserialize() {
        let id: Id = serde_json::from_str("\"abc1\"").unwrap();
        assert_eq!(id.as_str(), "abc1");
    }

    #[test]
    fn test_id_try_from_string() {
        let id = Id::try_from("abc1".to_string()).unwrap();
        assert_eq!(id.as_str(), "abc1");
    }

    #[test]
    fn test_id_try_from_str() {
        let id = Id::try_from("abc1").unwrap();
        assert_eq!(id.as_str(), "abc1");
    }
}
