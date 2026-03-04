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
    /// Task IDs that block this change (who I depend on)
    pub blocked_by: Vec<Id>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Timer tracking fields
    #[serde(default)]
    pub timer_start: Option<DateTime<Utc>>,
    #[serde(default)]
    pub accumulated_duration_secs: i64,
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
            blocked_by: Vec::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            timer_start: None,
            accumulated_duration_secs: 0,
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

    /// Add a blocker (task that blocks this change)
    pub fn add_blocker(&mut self, blocker_id: Id) {
        if !self.blocked_by.contains(&blocker_id) {
            self.blocked_by.push(blocker_id);
        }
        self.updated_at = Utc::now();
    }

    /// Remove a specific blocker
    pub fn remove_blocker(&mut self, blocker_id: &Id) {
        self.blocked_by.retain(|id| id != blocker_id);
        self.updated_at = Utc::now();
    }

    /// Remove all blockers
    pub fn clear_blockers(&mut self) {
        self.blocked_by.clear();
        self.updated_at = Utc::now();
    }

    /// Check if this change is blocked by a specific change
    pub fn is_blocked_by(&self, blocker_id: &Id) -> bool {
        self.blocked_by.contains(blocker_id)
    }

    /// Check if this change has any blockers
    pub fn has_blockers(&self) -> bool {
        !self.blocked_by.is_empty()
    }

    /// Check if all acceptance criteria are completed
    /// Returns true if all acceptance criteria items are checked, or if there are no acceptance criteria
    pub fn check_acceptance_criteria(&self) -> bool {
        // Look for unchecked items in acceptance criteria section
        let mut in_acceptance_criteria = false;

        for line in self.body.lines() {
            let trimmed = line.trim();

            // Check for acceptance criteria header
            if trimmed.to_lowercase().contains("acceptance criteria") {
                in_acceptance_criteria = true;
                continue;
            }

            // Exit if we hit another section
            if in_acceptance_criteria && trimmed.starts_with("##") {
                break;
            }

            // Check for unchecked items
            if in_acceptance_criteria && trimmed.starts_with("- [ ]") {
                return false;
            }
        }

        true
    }

    /// Check if the timer is currently running
    pub fn is_timer_running(&self) -> bool {
        self.timer_start.is_some()
    }

    /// Get the total accumulated duration plus any active timer time
    pub fn total_duration_secs(&self) -> i64 {
        let accumulated = self.accumulated_duration_secs;
        if let Some(start) = self.timer_start {
            let active_duration = Utc::now().signed_duration_since(start).num_seconds();
            accumulated + active_duration
        } else {
            accumulated
        }
    }

    /// Start the timer if not already running
    pub fn timer_start(&mut self) -> bool {
        if self.timer_start.is_none() {
            self.timer_start = Some(Utc::now());
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Stop the timer if running and add to accumulated duration
    pub fn timer_stop(&mut self) -> Option<i64> {
        if let Some(start) = self.timer_start.take() {
            let duration = Utc::now().signed_duration_since(start).num_seconds();
            self.accumulated_duration_secs += duration;
            self.updated_at = Utc::now();
            Some(duration)
        } else {
            None
        }
    }

    /// Format duration as human-readable string (e.g., "2h 15m" or "45m 30s")
    pub fn format_duration(&self) -> String {
        let total_secs = self.total_duration_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;

        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
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
    TimerStarted,
    TimerStopped,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EventType::Created => "created",
            EventType::StatusChanged => "status_changed",
            EventType::Updated => "updated",
            EventType::PriorityChanged => "priority_changed",
            EventType::ChangelogTypeChanged => "changelog_type_changed",
            EventType::DependencyAdded => "dependency_added",
            EventType::DependencyRemoved => "dependency_removed",
            EventType::ScratchpadAppended => "scratchpad_appended",
            EventType::ParentChanged => "parent_changed",
            EventType::TimerStarted => "timer_started",
            EventType::TimerStopped => "timer_stopped",
        };
        write!(f, "{}", s)
    }
}

impl EventType {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "created" => Some(EventType::Created),
            "statuschanged" | "status_changed" | "status-changed" => Some(EventType::StatusChanged),
            "updated" => Some(EventType::Updated),
            "prioritychanged" | "priority_changed" | "priority-changed" => {
                Some(EventType::PriorityChanged)
            }
            "changelogtypechanged" | "changelog_type_changed" | "changelog-type-changed" => {
                Some(EventType::ChangelogTypeChanged)
            }
            "dependencyadded" | "dependency_added" | "dependency-added" => {
                Some(EventType::DependencyAdded)
            }
            "dependencyremoved" | "dependency_removed" | "dependency-removed" => {
                Some(EventType::DependencyRemoved)
            }
            "scratchpadappended" | "scratchpad_appended" | "scratchpad-appended" => {
                Some(EventType::ScratchpadAppended)
            }
            "parentchanged" | "parent_changed" | "parent-changed" => Some(EventType::ParentChanged),
            "timerstarted" | "timer_started" | "timer-started" => Some(EventType::TimerStarted),
            "timerstopped" | "timer_stopped" | "timer-stopped" => Some(EventType::TimerStopped),
            _ => None,
        }
    }
}
