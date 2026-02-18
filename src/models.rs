use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: String,
    pub title: String,
    pub body: String,
    pub status: Status,
    pub priority: Priority,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Change {
    pub fn new(id: String, title: String, priority: Priority) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            body: String::new(),
            status: Status::Draft,
            priority,
            parent: None,
            children: Vec::new(),
            dependencies: Vec::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_body(mut self, body: String) -> Self {
        self.body = body;
        self
    }

    pub fn with_parent(mut self, parent: String) -> Self {
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

    pub fn to_string(&self) -> String {
        match self {
            Status::Draft => "draft".to_string(),
            Status::Approved => "approved".to_string(),
            Status::InProgress => "in_progress".to_string(),
            Status::Review => "review".to_string(),
            Status::Done => "done".to_string(),
            Status::Blocked => "blocked".to_string(),
            Status::Paused => "paused".to_string(),
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

impl Priority {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(Priority::Low),
            "medium" => Some(Priority::Medium),
            "high" => Some(Priority::High),
            "critical" => Some(Priority::Critical),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Priority::Low => "low".to_string(),
            Priority::Medium => "medium".to_string(),
            Priority::High => "high".to_string(),
            Priority::Critical => "critical".to_string(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub change_id: String,
    pub event_type: EventType,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl Event {
    pub fn new(change_id: String, event_type: EventType, data: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
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
    DependencyAdded,
    DependencyRemoved,
    ScratchpadAppended,
    ParentChanged,
}
