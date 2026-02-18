use anyhow::Result;
use std::path::PathBuf;

use crate::db::Db;
use crate::models::{Change, Event, EventType, Status};

pub struct ChangeRepository {
    pub db: Db,
}

impl ChangeRepository {
    pub async fn open(path: PathBuf) -> Result<Self> {
        let db = Db::open(path).await?;
        Ok(Self { db })
    }

    pub async fn save(&self) -> Result<()> {
        self.db.save().await
    }

    pub fn find_by_id(&self, id: &str) -> Option<&Change> {
        self.db.get_change(id)
    }

    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut Change> {
        self.db.get_change_mut(id)
    }

    pub async fn create(&mut self, change: Change) -> Result<&Change> {
        let id = change.id.clone();
        
        // Add event
        let event = Event::new(
            id.clone(),
            EventType::Created,
            serde_json::json!({
                "title": change.title,
                "priority": change.priority.to_string(),
                "parent": change.parent,
            }),
        );
        self.db.add_event(event);
        
        // Insert change
        self.db.insert_change(change)?;
        self.db.save().await?;
        
        Ok(self.db.get_change(&id).unwrap())
    }

    #[allow(dead_code)]
    pub async fn update(&mut self, change: Change) -> Result<&Change> {
        let id = change.id.clone();
        self.db.update_change(change)?;
        self.db.save().await?;
        Ok(self.db.get_change(&id).unwrap())
    }

    pub async fn update_status(
        &mut self,
        id: &str,
        new_status: Status,
    ) -> Result<&Change> {
        let change = self.db.get_change_mut(id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", id))?;
        
        let old_status = change.status.clone();
        change.update_status(new_status.clone());
        
        // Add event
        let event = Event::new(
            id.to_string(),
            EventType::StatusChanged,
            serde_json::json!({
                "from": old_status.to_string(),
                "to": new_status.to_string(),
            }),
        );
        self.db.add_event(event);
        
        self.db.save().await?;
        Ok(self.db.get_change(id).unwrap())
    }

    pub fn list(
        &self,
        status: Option<&str>,
        priority: Option<&str>,
        include_done: bool,
    ) -> Vec<&Change> {
        self.db.list_changes(status, priority, include_done)
    }

    pub fn get_events(&self, change_id: &str) -> Vec<&Event> {
        self.db.get_events_for_change(change_id)
    }
}
