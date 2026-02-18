use crate::db::Db;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct IDish(String);

impl IDish {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Resolve IDish to full ID
    /// - Case-insensitive exact match wins
    /// - Otherwise finds unique prefix match (case-insensitive)
    /// - Returns error if ambiguous or not found
    pub fn resolve(&self, db: &Db) -> Result<String, IDishError> {
        let input = self.0.to_lowercase();

        // 1. Check for exact match (case-insensitive)
        if let Some(change) = db.find_by_id_case_insensitive(&input) {
            return Ok(change.id.clone());
        }

        // 2. Find prefix matches (case-insensitive)
        let candidates: Vec<String> = db
            .find_by_prefix_case_insensitive(&input)
            .into_iter()
            .map(|c| c.id.clone())
            .collect();

        match candidates.len() {
            0 => Err(IDishError::NotFound {
                prefix: self.0.clone(),
            }),
            1 => Ok(candidates[0].clone()),
            _ => Err(IDishError::Ambiguous {
                prefix: self.0.clone(),
                candidates,
            }),
        }
    }
}

#[derive(Debug)]
pub enum IDishError {
    NotFound {
        prefix: String,
    },
    Ambiguous {
        prefix: String,
        candidates: Vec<String>,
    },
}

impl std::fmt::Display for IDishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IDishError::NotFound { prefix } => {
                write!(f, "ID '{}' not found", prefix)
            }
            IDishError::Ambiguous { prefix, candidates } => {
                writeln!(f, "ID prefix '{}' is ambiguous", prefix)?;
                write!(f, "hint: Candidates are: {}", candidates.join(", "))
            }
        }
    }
}

impl std::error::Error for IDishError {}

impl FromStr for IDish {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(IDish(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::models::{Change, Priority, Status};
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_db_with_changes(changes: Vec<(&str, &str)>) -> Db {
        let mut db = Database::default();
        for (id, title) in changes {
            let now = Utc::now();
            let change = Change {
                id: id.to_string(),
                title: title.to_string(),
                body: String::new(),
                status: Status::Draft,
                priority: Priority::Medium,
                parent: None,
                children: Vec::new(),
                dependencies: Vec::new(),
                tags: Vec::new(),
                created_at: now,
                updated_at: now,
            };
            db.changes.insert(id.to_string(), change);
        }
        Db {
            path: PathBuf::from("/tmp/test"),
            data: db,
        }
    }

    #[test]
    fn test_resolve_exact_match() {
        let db = create_test_db_with_changes(vec![("abc1", "Change 1"), ("def2", "Change 2")]);

        let idish = IDish("abc1".to_string());
        let result = idish.resolve(&db);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "abc1");
    }

    #[test]
    fn test_resolve_case_insensitive_exact_match() {
        let db = create_test_db_with_changes(vec![("abc1", "Change 1")]);

        let idish = IDish("ABC1".to_string());
        let result = idish.resolve(&db);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "abc1");
    }

    #[test]
    fn test_resolve_unique_prefix() {
        let db = create_test_db_with_changes(vec![("abc1", "Change 1"), ("def2", "Change 2")]);

        // "ab" should uniquely identify "abc1"
        let idish = IDish("ab".to_string());
        let result = idish.resolve(&db);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "abc1");
    }

    #[test]
    fn test_resolve_ambiguous_prefix() {
        let db = create_test_db_with_changes(vec![
            ("abc1", "Change 1"),
            ("abc2", "Change 2"),
            ("def3", "Change 3"),
        ]);

        // "ab" should be ambiguous between "abc1" and "abc2"
        let idish = IDish("ab".to_string());
        let result = idish.resolve(&db);
        assert!(result.is_err());

        match result.unwrap_err() {
            IDishError::Ambiguous { prefix, candidates } => {
                assert_eq!(prefix, "ab");
                assert_eq!(candidates.len(), 2);
                assert!(candidates.contains(&"abc1".to_string()));
                assert!(candidates.contains(&"abc2".to_string()));
            }
            _ => panic!("Expected Ambiguous error"),
        }
    }

    #[test]
    fn test_resolve_not_found() {
        let db = create_test_db_with_changes(vec![("abc1", "Change 1")]);

        let idish = IDish("xyz9".to_string());
        let result = idish.resolve(&db);
        assert!(result.is_err());

        match result.unwrap_err() {
            IDishError::NotFound { prefix } => {
                assert_eq!(prefix, "xyz9");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_resolve_full_id_ambiguous() {
        let db = create_test_db_with_changes(vec![("abc1", "Change 1"), ("abc2", "Change 2")]);

        // Full ID "abc1" should match exactly even though "abc" is ambiguous
        let idish = IDish("abc1".to_string());
        let result = idish.resolve(&db);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "abc1");
    }

    #[test]
    fn test_from_str() {
        let idish: IDish = "abc1".parse().unwrap();
        assert_eq!(idish.as_str(), "abc1");
    }
}
