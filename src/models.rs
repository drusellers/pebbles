use crate::id::Id;
use chrono::{DateTime, Utc};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: Id,
    pub title: String,
    pub body: String,
    pub status: Status,
    pub priority: Priority,
    pub changelog_type: Option<ChangelogType>,
    pub parent: Option<Id>,
    pub children: Vec<Id>,
    pub dependencies: Vec<Id>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Change {
    pub fn new(id: Id, title: String, priority: Priority) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            body: String::new(),
            status: Status::Draft,
            priority,
            changelog_type: None,
            parent: None,
            children: Vec::new(),
            dependencies: Vec::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(dead_code)]
    pub fn with_body(mut self, body: String) -> Self {
        self.body = body;
        self
    }

    #[allow(dead_code)]
    pub fn with_parent(mut self, parent: Id) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn update_status(&mut self, new_status: Status) {
        self.status = new_status;
        self.updated_at = Utc::now();
    }

    pub fn update_title(&mut self, title: String) {
        self.title = title;
        self.updated_at = Utc::now();
    }

    pub fn update_body(&mut self, body: String) {
        self.body = body;
        self.updated_at = Utc::now();
    }

    pub fn update_priority(&mut self, priority: Priority) {
        self.priority = priority;
        self.updated_at = Utc::now();
    }

    pub fn update_changelog_type(&mut self, changelog_type: ChangelogType) {
        self.changelog_type = Some(changelog_type);
        self.updated_at = Utc::now();
    }

    #[allow(dead_code)]
    pub fn update_parent(&mut self, parent: Option<Id>) {
        self.parent = parent;
        self.updated_at = Utc::now();
    }

    #[allow(dead_code)]
    pub fn add_child(&mut self, child_id: Id) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
        self.updated_at = Utc::now();
    }

    #[allow(dead_code)]
    pub fn add_dependency(&mut self, dep_id: Id) {
        if !self.dependencies.contains(&dep_id) {
            self.dependencies.push(dep_id);
        }
        self.updated_at = Utc::now();
    }

    #[allow(dead_code)]
    pub fn remove_dependency(&mut self, dep_id: &Id) {
        self.dependencies.retain(|id| id != dep_id);
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Draft,
    Approved,
    InProgress,
    Review,
    Done,
    Blocked,
    Paused,
}

struct StatusVisitor;

impl<'de> Visitor<'de> for StatusVisitor {
    type Value = Status;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid status string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Status::from_string(value).unwrap_or(Status::Draft))
    }
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(StatusVisitor)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Draft => write!(f, "draft"),
            Status::Approved => write!(f, "approved"),
            Status::InProgress => write!(f, "in_progress"),
            Status::Review => write!(f, "review"),
            Status::Done => write!(f, "done"),
            Status::Blocked => write!(f, "blocked"),
            Status::Paused => write!(f, "paused"),
        }
    }
}

impl Status {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "draft" => Some(Status::Draft),
            "approved" => Some(Status::Approved),
            "inprogress" | "in_progress" | "in-progress" => Some(Status::InProgress),
            "review" => Some(Status::Review),
            "done" => Some(Status::Done),
            "blocked" => Some(Status::Blocked),
            "paused" => Some(Status::Paused),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChangelogType {
    Feature,
    Fix,
    Change,
    Deprecated,
    Removed,
    Security,
    Internal,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Low => write!(f, "low"),
            Priority::Medium => write!(f, "medium"),
            Priority::High => write!(f, "high"),
            Priority::Critical => write!(f, "critical"),
        }
    }
}

impl fmt::Display for ChangelogType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangelogType::Feature => write!(f, "feature"),
            ChangelogType::Fix => write!(f, "fix"),
            ChangelogType::Change => write!(f, "change"),
            ChangelogType::Deprecated => write!(f, "deprecated"),
            ChangelogType::Removed => write!(f, "removed"),
            ChangelogType::Security => write!(f, "security"),
            ChangelogType::Internal => write!(f, "internal"),
        }
    }
}

impl ChangelogType {
    #[allow(dead_code)]
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "feature" => Some(ChangelogType::Feature),
            "fix" => Some(ChangelogType::Fix),
            "change" => Some(ChangelogType::Change),
            "deprecated" => Some(ChangelogType::Deprecated),
            "removed" => Some(ChangelogType::Removed),
            "security" => Some(ChangelogType::Security),
            "internal" => Some(ChangelogType::Internal),
            _ => None,
        }
    }
}

impl From<crate::cli::PriorityArg> for Priority {
    fn from(p: crate::cli::PriorityArg) -> Self {
        match p {
            crate::cli::PriorityArg::Low => Priority::Low,
            crate::cli::PriorityArg::Medium => Priority::Medium,
            crate::cli::PriorityArg::High => Priority::High,
            crate::cli::PriorityArg::Critical => Priority::Critical,
        }
    }
}

impl From<crate::cli::ChangelogTypeArg> for ChangelogType {
    fn from(ct: crate::cli::ChangelogTypeArg) -> Self {
        match ct {
            crate::cli::ChangelogTypeArg::Feature => ChangelogType::Feature,
            crate::cli::ChangelogTypeArg::Fix => ChangelogType::Fix,
            crate::cli::ChangelogTypeArg::Change => ChangelogType::Change,
            crate::cli::ChangelogTypeArg::Deprecated => ChangelogType::Deprecated,
            crate::cli::ChangelogTypeArg::Removed => ChangelogType::Removed,
            crate::cli::ChangelogTypeArg::Security => ChangelogType::Security,
            crate::cli::ChangelogTypeArg::Internal => ChangelogType::Internal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Id,
    pub change_id: Id,
    pub event_type: EventType,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl Event {
    pub fn new(change_id: Id, event_type: EventType, data: serde_json::Value) -> Self {
        Self {
            id: Id::generate(),
            change_id,
            event_type,
            data,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Created,
    StatusChanged,
    Updated,
    PriorityChanged,
    ChangelogTypeChanged,
    DependencyAdded,
    DependencyRemoved,
    ScratchpadAppended,
    ParentChanged,
}
