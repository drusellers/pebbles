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
    path: PathBuf,
    data: Database,
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
}
