use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::models::{Change, Event};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Database {
    pub changes: HashMap<String, Change>,
    pub events: Vec<Event>,
}

pub struct Db {
    pub(crate) path: PathBuf,
    pub(crate) data: Database,
}

impl Db {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        if path.exists() {
            let content = tokio::fs::read_to_string(&path)
                .await
                .context("Failed to read database file")?;
            let data: Database = serde_json::from_str(&content)
                .context("Failed to parse database file")?;
            Ok(Self { path, data })
        } else {
            Ok(Self {
                path,
                data: Database::default(),
            })
        }
    }

    pub async fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.data)
            .context("Failed to serialize database")?;
        
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create database directory")?;
        }
        
        tokio::fs::write(&self.path, content)
            .await
            .context("Failed to write database file")?;
        
        Ok(())
    }

    pub fn get_change(&self, id: &str) -> Option<&Change> {
        self.data.changes.get(id)
    }

    pub fn get_change_mut(&mut self, id: &str) -> Option<&mut Change> {
        self.data.changes.get_mut(id)
    }

    pub fn insert_change(&mut self, change: Change) -> Result<()> {
        if self.data.changes.contains_key(&change.id) {
            anyhow::bail!("Change with ID '{}' already exists", change.id);
        }
        self.data.changes.insert(change.id.clone(), change);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn update_change(&mut self, change: Change) -> Result<()> {
        if !self.data.changes.contains_key(&change.id) {
            anyhow::bail!("Change with ID '{}' not found", change.id);
        }
        self.data.changes.insert(change.id.clone(), change);
        Ok(())
    }

    pub fn add_event(&mut self, event: Event) {
        self.data.events.push(event);
    }

    pub fn list_changes(&self,
        status: Option<&str>,
        priority: Option<&str>,
        include_done: bool,
    ) -> Vec<&Change> {
        let mut changes: Vec<&Change> = self.data.changes.values().collect();
        
        if let Some(status) = status {
            changes.retain(|c| c.status.to_string() == status);
        }
        
        if let Some(priority) = priority {
            changes.retain(|c| c.priority.to_string() == priority);
        }
        
        if !include_done {
            changes.retain(|c| !matches!(c.status, crate::models::Status::Done));
        }
        
        changes
    }

    pub fn get_events_for_change(&self, change_id: &str) -> Vec<&Event> {
        self.data.events
            .iter()
            .filter(|e| e.change_id == change_id)
            .collect()
    }

    pub fn delete_change(&mut self, id: &str) -> Result<()> {
        if !self.data.changes.contains_key(id) {
            anyhow::bail!("Change with ID '{}' not found", id);
        }
        self.data.changes.remove(id);
        Ok(())
    }

    // Case-insensitive exact match
    pub fn find_by_id_case_insensitive(&self, id: &str) -> Option<&Change> {
        let id_lower = id.to_lowercase();
        self.data.changes.values()
            .find(|c| c.id.to_lowercase() == id_lower)
    }

    // Case-insensitive prefix search
    pub fn find_by_prefix_case_insensitive(&self, prefix: &str) -> Vec<&Change> {
        let prefix_lower = prefix.to_lowercase();
        self.data.changes.values()
            .filter(|c| c.id.to_lowercase().starts_with(&prefix_lower))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Priority, Status};
    use chrono::Utc;

    fn create_test_change(id: &str, title: &str) -> Change {
        let now = Utc::now();
        Change {
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
        }
    }

    #[test]
    fn test_find_by_id_case_insensitive_exact_match() {
        let mut db = Database::default();
        let change = create_test_change("abc1", "Test Change");
        db.changes.insert(change.id.clone(), change);

        let db_wrapper = Db {
            path: PathBuf::from("/tmp/test"),
            data: db,
        };

        // Should find with exact case
        assert!(db_wrapper.find_by_id_case_insensitive("abc1").is_some());
        // Should find with different case
        assert!(db_wrapper.find_by_id_case_insensitive("ABC1").is_some());
        assert!(db_wrapper.find_by_id_case_insensitive("Abc1").is_some());
        // Should not find non-existent ID
        assert!(db_wrapper.find_by_id_case_insensitive("xyz9").is_none());
    }

    #[test]
    fn test_find_by_prefix_case_insensitive() {
        let mut db = Database::default();
        db.changes.insert("abc1".to_string(), create_test_change("abc1", "Change 1"));
        db.changes.insert("abc2".to_string(), create_test_change("abc2", "Change 2"));
        db.changes.insert("def1".to_string(), create_test_change("def1", "Change 3"));

        let db_wrapper = Db {
            path: PathBuf::from("/tmp/test"),
            data: db,
        };

        // Should find multiple with prefix "ab" (case insensitive)
        let results = db_wrapper.find_by_prefix_case_insensitive("ab");
        assert_eq!(results.len(), 2);

        // Should find multiple with uppercase prefix
        let results = db_wrapper.find_by_prefix_case_insensitive("AB");
        assert_eq!(results.len(), 2);

        // Should find single with longer prefix
        let results = db_wrapper.find_by_prefix_case_insensitive("abc1");
        assert_eq!(results.len(), 1);

        // Should find none with non-matching prefix
        let results = db_wrapper.find_by_prefix_case_insensitive("xyz");
        assert_eq!(results.len(), 0);

        // Should find with different case
        let results = db_wrapper.find_by_prefix_case_insensitive("DEF");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_find_by_prefix_single_char() {
        let mut db = Database::default();
        db.changes.insert("abc1".to_string(), create_test_change("abc1", "Change 1"));
        db.changes.insert("abc2".to_string(), create_test_change("abc2", "Change 2"));
        db.changes.insert("def1".to_string(), create_test_change("def1", "Change 3"));

        let db_wrapper = Db {
            path: PathBuf::from("/tmp/test"),
            data: db,
        };

        // Should find multiple with single char prefix
        let results = db_wrapper.find_by_prefix_case_insensitive("a");
        assert_eq!(results.len(), 2);

        let results = db_wrapper.find_by_prefix_case_insensitive("d");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_delete_change() {
        let mut db = Database::default();
        db.changes.insert("abc1".to_string(), create_test_change("abc1", "Change 1"));

        let mut db_wrapper = Db {
            path: PathBuf::from("/tmp/test"),
            data: db,
        };

        // Should delete existing change
        assert!(db_wrapper.delete_change("abc1").is_ok());
        assert!(db_wrapper.get_change("abc1").is_none());

        // Should error when deleting non-existent change
        assert!(db_wrapper.delete_change("xyz9").is_err());
    }
}
