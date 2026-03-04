use crate::id::Id;
use crate::models::{Change, ChangelogType, Event, EventType, Priority, Status};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

pub type ParseResult<T> = std::result::Result<T, MarkdownParseError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownParseError {
    MissingFrontmatterClose,
    InvalidFrontmatter(String),
    InvalidParentId(String),
    InvalidIdField { field: String, value: String },
    InvalidEventTimestamp(String),
    InvalidEventId(String),
}

impl std::fmt::Display for MarkdownParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkdownParseError::MissingFrontmatterClose => {
                write!(f, "frontmatter is not closed with '---'")
            }
            MarkdownParseError::InvalidFrontmatter(err) => {
                write!(f, "invalid yaml frontmatter: {}", err)
            }
            MarkdownParseError::InvalidParentId(id) => {
                write!(f, "invalid parent id in frontmatter: '{}'", id)
            }
            MarkdownParseError::InvalidIdField { field, value } => {
                write!(f, "invalid id in '{}': '{}'", field, value)
            }
            MarkdownParseError::InvalidEventTimestamp(ts) => {
                write!(f, "invalid event timestamp '{}'", ts)
            }
            MarkdownParseError::InvalidEventId(id) => write!(f, "invalid event id '{}'", id),
        }
    }
}

impl std::error::Error for MarkdownParseError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Frontmatter {
    id: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    changelog_type: Option<String>,
    parent: Option<String>,
    children: Option<Vec<String>>,
    dependencies: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct ParsedChangeFile {
    pub change: Change,
    pub events: Vec<Event>,
    pub normalized_content: Option<String>,
}

enum EventHeaderStyle {
    Heading,
    Numbered,
}

struct ParsedBody {
    title: String,
    description: String,
    events: Vec<Event>,
    used_legacy_event_headers: bool,
}

pub fn parse_change_file(id: &Id, content: &str) -> ParseResult<ParsedChangeFile> {
    let now = Utc::now();
    let (fm, body, normalized_missing_frontmatter) = if has_frontmatter(content) {
        let (fm, body) = split_frontmatter(content)?;
        (Some(fm), body, false)
    } else {
        (None, content.to_string(), true)
    };

    let parsed_body = parse_body(id, &body)?;

    let mut frontmatter = fm.unwrap_or(Frontmatter {
        id: Some(id.to_string()),
        status: Some("draft".to_string()),
        priority: Some("medium".to_string()),
        changelog_type: None,
        parent: None,
        children: Some(Vec::new()),
        dependencies: Some(Vec::new()),
        tags: Some(Vec::new()),
        created_at: Some(now),
        updated_at: Some(now),
    });

    if frontmatter.id.is_none() {
        frontmatter.id = Some(id.to_string());
    }
    if frontmatter.status.is_none() {
        frontmatter.status = Some("draft".to_string());
    }
    if frontmatter.priority.is_none() {
        frontmatter.priority = Some("medium".to_string());
    }
    if frontmatter.children.is_none() {
        frontmatter.children = Some(Vec::new());
    }
    if frontmatter.dependencies.is_none() {
        frontmatter.dependencies = Some(Vec::new());
    }
    if frontmatter.tags.is_none() {
        frontmatter.tags = Some(Vec::new());
    }
    if frontmatter.created_at.is_none() {
        frontmatter.created_at = Some(now);
    }
    if frontmatter.updated_at.is_none() {
        frontmatter.updated_at = Some(now);
    }

    let status = Status::from_string(frontmatter.status.as_deref().unwrap_or("draft"))
        .unwrap_or(Status::Draft);
    let priority = Priority::from_string(frontmatter.priority.as_deref().unwrap_or("medium"))
        .unwrap_or(Priority::Medium);
    let changelog_type = frontmatter
        .changelog_type
        .as_deref()
        .and_then(ChangelogType::from_string);

    let parent = frontmatter
        .parent
        .as_deref()
        .map(Id::new)
        .transpose()
        .map_err(|_| MarkdownParseError::InvalidParentId(frontmatter.parent.unwrap_or_default()))?;

    let children = parse_ids(frontmatter.children.unwrap_or_default(), "children")?;
    let dependencies = parse_ids(frontmatter.dependencies.unwrap_or_default(), "dependencies")?;

    let change = Change {
        id: id.clone(),
        title: parsed_body.title,
        body: parsed_body.description,
        status,
        priority,
        changelog_type,
        parent,
        children,
        dependencies,
        tags: frontmatter.tags.unwrap_or_default(),
        created_at: frontmatter.created_at.unwrap_or(now),
        updated_at: frontmatter.updated_at.unwrap_or(now),
    };

    let normalized_content =
        if normalized_missing_frontmatter || parsed_body.used_legacy_event_headers {
            Some(write_change_file(&change, &parsed_body.events))
        } else {
            None
        };

    Ok(ParsedChangeFile {
        change,
        events: parsed_body.events,
        normalized_content,
    })
}

pub fn write_change_file(change: &Change, events: &[Event]) -> String {
    let frontmatter = Frontmatter {
        id: Some(change.id.to_string()),
        status: Some(change.status.to_string()),
        priority: Some(change.priority.to_string()),
        changelog_type: change.changelog_type.as_ref().map(ToString::to_string),
        parent: change.parent.as_ref().map(ToString::to_string),
        children: Some(change.children.iter().map(ToString::to_string).collect()),
        dependencies: Some(
            change
                .dependencies
                .iter()
                .map(ToString::to_string)
                .collect(),
        ),
        tags: Some(change.tags.clone()),
        created_at: Some(change.created_at),
        updated_at: Some(change.updated_at),
    };

    let mut out = String::new();
    out.push_str("---\n");
    let frontmatter_yaml = serde_yaml::to_string(&frontmatter)
        .unwrap_or_else(|_| "status: draft\npriority: medium\n".to_string());
    out.push_str(&frontmatter_yaml);
    out.push_str("---\n\n");

    out.push_str("# ");
    out.push_str(change.title.trim());
    out.push_str("\n\n");

    if !change.body.trim().is_empty() {
        out.push_str(change.body.trim_end());
        out.push_str("\n\n");
    }

    out.push_str("## Events\n\n");
    for (idx, event) in events.iter().enumerate() {
        out.push_str(&format!(
            "{}. {} [{}] {}\n",
            idx + 1,
            event.created_at.to_rfc3339(),
            event.id,
            event.event_type
        ));
        let payload = serde_json::to_string(&event.data).unwrap_or_else(|_| "{}".to_string());
        for line in payload.lines() {
            out.push_str("   ");
            out.push_str(line);
            out.push('\n');
        }
        out.push_str("\n\n");
    }

    out
}

pub async fn write_change_file_to_path(
    path: &Path,
    change: &Change,
    events: &[Event],
) -> anyhow::Result<()> {
    let content = write_change_file(change, events);
    tokio::fs::write(path, content)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", path.display(), e))
}

fn has_frontmatter(content: &str) -> bool {
    content
        .lines()
        .next()
        .map(|line| line.trim() == "---")
        .unwrap_or(false)
}

fn split_frontmatter(content: &str) -> ParseResult<(Frontmatter, String)> {
    let mut lines = content.lines();
    let first = lines.next().unwrap_or_default();
    if first.trim() != "---" {
        return Err(MarkdownParseError::InvalidFrontmatter(
            "file must start with frontmatter delimiter '---'".to_string(),
        ));
    }

    let mut fm_lines = Vec::new();
    let mut closed = false;
    for line in &mut lines {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        fm_lines.push(line);
    }

    if !closed {
        return Err(MarkdownParseError::MissingFrontmatterClose);
    }

    let body = lines.collect::<Vec<_>>().join("\n");
    let fm: Frontmatter = serde_yaml::from_str(&fm_lines.join("\n"))
        .map_err(|e| MarkdownParseError::InvalidFrontmatter(e.to_string()))?;

    Ok((fm, body.trim_start_matches('\n').to_string()))
}

fn parse_body(id: &Id, body: &str) -> ParseResult<ParsedBody> {
    let lines = body.lines().collect::<Vec<_>>();
    let mut title = String::new();
    let mut description_lines = Vec::new();
    let mut events = Vec::new();
    let mut in_events = false;
    let mut current_event: Option<Event> = None;
    let mut current_event_data = Vec::new();
    let mut used_legacy_event_headers = false;

    for line in lines {
        if title.is_empty() && line.starts_with("# ") {
            title = line.trim_start_matches("# ").trim().to_string();
            continue;
        }

        if line.trim() == "## Events" {
            in_events = true;
            continue;
        }

        if in_events {
            if let Some((ts, event_id, event_type, style)) = parse_event_header(line)? {
                if let Some(mut e) = current_event.take() {
                    e.data = parse_event_data(&current_event_data.join("\n"));
                    events.push(e);
                    current_event_data.clear();
                }

                if matches!(style, EventHeaderStyle::Heading) {
                    used_legacy_event_headers = true;
                }

                current_event = Some(Event {
                    id: event_id,
                    change_id: id.clone(),
                    event_type,
                    data: Value::Null,
                    created_at: ts,
                });
                continue;
            }

            if current_event.is_some() {
                current_event_data.push(line.to_string());
            }
            continue;
        }

        description_lines.push(line);
    }

    if let Some(mut e) = current_event.take() {
        e.data = parse_event_data(&current_event_data.join("\n"));
        events.push(e);
    }

    if title.is_empty() {
        title = format!("Change {}", id);
    }

    let description = description_lines.join("\n").trim().to_string();
    Ok(ParsedBody {
        title,
        description,
        events,
        used_legacy_event_headers,
    })
}

fn parse_event_header(
    line: &str,
) -> ParseResult<Option<(DateTime<Utc>, Id, EventType, EventHeaderStyle)>> {
    let trimmed = line.trim();
    let (rest, style) = if trimmed.starts_with("### ") {
        (
            trimmed.trim_start_matches("### ").trim(),
            EventHeaderStyle::Heading,
        )
    } else if let Some(rest) = strip_numbered_list_prefix(trimmed) {
        (rest, EventHeaderStyle::Numbered)
    } else {
        return Ok(None);
    };

    let open = rest.find('[');
    let close = rest.find(']');

    let (open, close) = match (open, close) {
        (Some(o), Some(c)) if c > o => (o, c),
        _ => return Ok(None),
    };

    let ts = rest[..open].trim();
    let event_id = rest[open + 1..close].trim();
    let event_type = rest[close + 1..].trim();

    let ts = DateTime::parse_from_rfc3339(ts)
        .map_err(|_| MarkdownParseError::InvalidEventTimestamp(ts.to_string()))?
        .with_timezone(&Utc);
    let event_id =
        Id::new(event_id).map_err(|_| MarkdownParseError::InvalidEventId(event_id.to_string()))?;
    let event_type = EventType::from_string(event_type).unwrap_or(EventType::Updated);

    Ok(Some((ts, event_id, event_type, style)))
}

fn strip_numbered_list_prefix(s: &str) -> Option<&str> {
    let (prefix, rest) = s.split_once(". ")?;
    if prefix.is_empty() || !prefix.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(rest)
}

fn parse_event_data(raw: &str) -> Value {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(trimmed).unwrap_or_else(|_| serde_json::json!({"message": trimmed}))
    }
}

fn parse_ids(values: Vec<String>, field: &str) -> ParseResult<Vec<Id>> {
    values
        .into_iter()
        .map(|raw| {
            Id::new(raw.clone()).map_err(|_| MarkdownParseError::InvalidIdField {
                field: field.to_string(),
                value: raw,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn parse_file_without_frontmatter_auto_adds_defaults() {
        let id = Id::new("abc1").expect("valid id");
        let content = "# Sample change\n\nThis was hand-written without frontmatter.";

        let parsed = parse_change_file(&id, content).expect("should parse");

        assert_eq!(parsed.change.id, id);
        assert_eq!(parsed.change.status, Status::Draft);
        assert_eq!(parsed.change.priority, Priority::Medium);
        assert_eq!(parsed.change.title, "Sample change");
        assert!(parsed.normalized_content.is_some());

        let normalized = parsed
            .normalized_content
            .as_ref()
            .expect("normalized content should be present");
        assert!(normalized.starts_with("---\n"));
        assert!(normalized.contains("status: draft"));
        assert!(normalized.contains("priority: medium"));
        assert!(normalized.contains("## Events"));
    }

    #[test]
    fn parse_file_with_legacy_heading_events_autoconverts() {
        let id = Id::new("def2").expect("valid id");
        let content = r#"---
id: def2
status: in_progress
priority: high
dependencies: [abc1]
tags: [backend]
created_at: 2026-03-03T10:00:00Z
updated_at: 2026-03-03T11:00:00Z
---

# Implement API endpoint

Need to build and wire endpoint.

## Events

### 2026-03-03T12:00:00Z [a1b2] updated
{"field":"status","from":"draft","to":"in_progress"}
"#;

        let parsed = parse_change_file(&id, content).expect("should parse");

        assert_eq!(parsed.change.status, Status::InProgress);
        assert_eq!(parsed.change.priority, Priority::High);
        assert_eq!(parsed.change.title, "Implement API endpoint");
        assert_eq!(parsed.change.dependencies.len(), 1);
        assert_eq!(parsed.change.dependencies[0].as_str(), "abc1");
        assert_eq!(parsed.events.len(), 1);
        assert_eq!(parsed.events[0].event_type.to_string(), "updated");
        assert_eq!(
            parsed.events[0].created_at,
            Utc.with_ymd_and_hms(2026, 3, 3, 12, 0, 0)
                .single()
                .expect("valid timestamp")
        );
        let normalized = parsed
            .normalized_content
            .as_ref()
            .expect("legacy heading events should be normalized");
        assert!(normalized.contains("## Events"));
        assert!(normalized.contains("1. 2026-03-03T12:00:00+00:00 [a1b2] updated"));
    }

    #[test]
    fn parse_file_with_numbered_events_does_not_rewrite() {
        let id = Id::new("def2").expect("valid id");
        let content = r#"---
id: def2
status: in_progress
priority: high
dependencies: [abc1]
tags: [backend]
created_at: 2026-03-03T10:00:00Z
updated_at: 2026-03-03T11:00:00Z
---

# Implement API endpoint

Need to build and wire endpoint.

## Events

1. 2026-03-03T12:00:00Z [a1b2] updated
   {"field":"status","from":"draft","to":"in_progress"}
"#;

        let parsed = parse_change_file(&id, content).expect("should parse");
        assert_eq!(parsed.events.len(), 1);
        assert!(parsed.normalized_content.is_none());
    }

    #[test]
    fn write_then_parse_roundtrip_preserves_core_fields() {
        let id = Id::new("ghi3").expect("valid id");
        let now = Utc
            .with_ymd_and_hms(2026, 3, 3, 10, 0, 0)
            .single()
            .expect("valid timestamp");

        let change = Change {
            id: id.clone(),
            title: "Roundtrip Change".to_string(),
            body: "Body text".to_string(),
            status: Status::Review,
            priority: Priority::Low,
            changelog_type: Some(ChangelogType::Change),
            parent: None,
            children: Vec::new(),
            dependencies: vec![Id::new("abc1").expect("valid id")],
            tags: vec!["docs".to_string()],
            created_at: now,
            updated_at: now,
        };

        let events = vec![Event {
            id: Id::new("z9y8").expect("valid id"),
            change_id: id.clone(),
            event_type: EventType::Updated,
            data: serde_json::json!({"field":"title"}),
            created_at: now,
        }];

        let serialized = write_change_file(&change, &events);
        let parsed = parse_change_file(&id, &serialized).expect("should parse");

        assert_eq!(parsed.change.id, change.id);
        assert_eq!(parsed.change.title, change.title);
        assert_eq!(parsed.change.status, change.status);
        assert_eq!(parsed.change.priority, change.priority);
        assert_eq!(parsed.events.len(), 1);
    }

    #[test]
    fn malformed_frontmatter_returns_typed_error() {
        let id = Id::new("abc1").expect("valid id");
        let content = "---\nstatus: [draft\n---\n\n# Bad yaml";

        let err = match parse_change_file(&id, content) {
            Ok(_) => panic!("expected parse error"),
            Err(err) => err,
        };
        match err {
            MarkdownParseError::InvalidFrontmatter(_) => {}
            other => panic!("unexpected error: {}", other),
        }
    }
}
