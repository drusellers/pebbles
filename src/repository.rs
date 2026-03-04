use crate::config::get_db_path;
use crate::db::{Db, InvalidChangeFile};
use crate::id::Id;
use crate::idish::{IDish, IDishError};
use crate::models::{Change, Event, EventType, Status};
use anyhow::Result;
use std::path::Path;

pub struct ChangeRepository {
    pub db: Db,
}

impl ChangeRepository {
    pub async fn open() -> Result<Self> {
        let path = get_db_path()?;
        Self::open_from(path).await
    }

    pub async fn open_from(path: impl AsRef<Path>) -> Result<Self> {
        let db = Db::open(path).await?;
        Ok(Self { db })
    }

    pub async fn save(&self) -> Result<()> {
        self.db.save().await
    }

    pub fn find_by_id(&self, id: &Id) -> Option<&Change> {
        self.db.get_change(id)
    }

    pub fn find_by_id_mut(&mut self, id: &Id) -> Option<&mut Change> {
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
                "parent": change.parent.as_ref().map(|p| p.to_string()),
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

    pub async fn update_status(&mut self, id: &Id, new_status: Status) -> Result<&Change> {
        let change = self
            .db
            .get_change_mut(id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", id))?;

        let old_status = change.status.clone();
        change.update_status(new_status.clone());

        // Add event
        let event = Event::new(
            id.clone(),
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

    /// Mark a change as done and auto-unblock any changes that were blocked by it.
    /// Returns the list of newly unblocked change IDs.
    pub async fn mark_done_and_unblock(&mut self, id: &Id) -> Result<Vec<Id>> {
        // First mark the change as done
        self.update_status(id, Status::Done).await?;

        // Find all changes blocked by this one
        let blocked_change_ids: Vec<Id> = self
            .db
            .data
            .changes
            .values()
            .filter(|c| c.is_blocked_by(id))
            .map(|c| c.id.clone())
            .collect();

        let mut unblocked = Vec::new();

        for blocked_id in blocked_change_ids {
            // Check if all blockers are now done
            let should_unblock = if let Some(blocked_change) = self.db.get_change(&blocked_id) {
                blocked_change
                    .blocked_by
                    .iter()
                    .all(|blocker_id| {
                        self.db
                            .get_change(blocker_id)
                            .map(|c| c.status == Status::Done)
                            .unwrap_or(true) // If blocker doesn't exist, treat as done
                    })
            } else {
                false
            };

            if should_unblock {
                // Unblock the change (change status from Blocked to Draft)
                if let Some(change_mut) = self.db.get_change_mut(&blocked_id) {
                    if change_mut.status == Status::Blocked {
                        change_mut.update_status(Status::Draft);

                        // Add event
                        let event = Event::new(
                            blocked_id.clone(),
                            EventType::StatusChanged,
                            serde_json::json!({
                                "from": "blocked",
                                "to": "draft",
                                "reason": "auto_unblocked",
                                "blocker_done": id.to_string(),
                            }),
                        );
                        self.db.add_event(event);

                        unblocked.push(blocked_id);
                    }
                }
            }
        }

        if !unblocked.is_empty() {
            self.db.save().await?;
        }

        Ok(unblocked)
    }

    /// List changes that are ready to work (not done, not blocked)
    pub fn list_ready(&self) -> Vec<&Change> {
        self.db
            .data
            .changes
            .values()
            .filter(|c| {
                c.status != Status::Done
                    && !c.has_blockers()
                    && c.status != Status::Blocked
            })
            .collect()
    }

    pub fn list(
        &self,
        status: Option<&str>,
        priority: Option<&str>,
        changelog: Option<&str>,
        include_done: bool,
    ) -> Vec<&Change> {
        self.db
            .list_changes(status, priority, changelog, include_done)
    }

    pub fn get_events(&self, change_id: &Id) -> Vec<&Event> {
        self.db.get_events_for_change(change_id)
    }

    pub fn invalid_changes(&self) -> Vec<&InvalidChangeFile> {
        self.db.list_invalid_changes()
    }

    pub fn invalid_change_by_id(&self, id: &Id) -> Option<&InvalidChangeFile> {
        self.db.invalid_changes.get(id)
    }

    pub fn resolve_invalid_idish(&self, idish: &IDish) -> Result<Option<Id>> {
        let input = idish.as_str().to_lowercase();

        if let Some(found) = self
            .db
            .invalid_changes
            .keys()
            .find(|id| id.as_str().to_lowercase() == input)
        {
            return Ok(Some(found.clone()));
        }

        let candidates: Vec<Id> = self
            .db
            .invalid_changes
            .keys()
            .filter(|id| id.as_str().to_lowercase().starts_with(&input))
            .cloned()
            .collect();

        match candidates.len() {
            0 => Ok(None),
            1 => Ok(candidates.first().cloned()),
            _ => Err(anyhow::anyhow!(IDishError::Ambiguous {
                prefix: idish.as_str().to_string(),
                candidates: candidates.iter().map(ToString::to_string).collect(),
            })),
        }
    }
}
