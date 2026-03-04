use std::fs;
use std::process::Command;

#[test]
fn migrate_converts_db_json_and_removes_legacy_file() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let pebbles_dir = temp_dir.path().join(".pebbles");
    fs::create_dir_all(&pebbles_dir).expect("create .pebbles dir");

    let db_json = pebbles_dir.join("db.json");
    let db_content = r#"{
  "changes": {
    "abc1": {
      "id": "abc1",
      "title": "Legacy task",
      "body": "Legacy body",
      "status": "draft",
      "priority": "medium",
      "changelog_type": null,
      "parent": null,
      "children": [],
      "dependencies": [],
      "tags": ["legacy"],
      "created_at": "2026-03-03T10:00:00Z",
      "updated_at": "2026-03-03T10:00:00Z"
    }
  },
  "events": [
    {
      "id": "e001",
      "change_id": "abc1",
      "event_type": "created",
      "data": {"source": "json"},
      "created_at": "2026-03-03T10:00:00Z"
    }
  ]
}
"#;
    fs::write(&db_json, db_content).expect("write legacy db.json");

    let output = Command::new(assert_cmd::cargo::cargo_bin!("pebbles"))
        .arg("migrate")
        .current_dir(temp_dir.path())
        .output()
        .expect("run pebbles migrate");

    assert!(output.status.success(), "migrate should succeed");
    assert!(!db_json.exists(), "legacy db.json should be removed");

    let markdown_path = pebbles_dir.join("changes").join("abc1.md");
    assert!(markdown_path.exists(), "markdown task file should exist");
    let markdown = fs::read_to_string(&markdown_path).expect("read migrated markdown");

    assert!(markdown.contains("status: draft"));
    assert!(markdown.contains("priority: medium"));
    assert!(markdown.contains("# Legacy task"));
    assert!(markdown.contains("## Events"));
    assert!(markdown.contains("[e001] created"));
}
