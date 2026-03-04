use std::fs;
use std::process::Command;

#[test]
fn list_shows_invalid_markdown_row_for_malformed_frontmatter() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let pebbles_dir = temp_dir.path().join(".pebbles");
    let changes_dir = pebbles_dir.join("changes");

    fs::create_dir_all(&changes_dir).expect("create changes dir");

    let invalid_file = changes_dir.join("abc1.md");
    fs::write(
        &invalid_file,
        "---\nstatus: [draft\n---\n\n# Broken change\n\nBody",
    )
    .expect("write invalid markdown file");

    let output = Command::new(assert_cmd::cargo::cargo_bin!("pebbles"))
        .arg("list")
        .current_dir(temp_dir.path())
        .output()
        .expect("run pebbles list");

    assert!(output.status.success(), "list command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("abc1"),
        "stdout should include invalid change id, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("[invalid markdown]"),
        "stdout should include invalid markdown note, got:\n{}",
        stdout
    );
    assert!(
        stderr.contains("invalid change file") || stderr.contains("invalid change file(s)"),
        "stderr should include warning summary, got:\n{}",
        stderr
    );
}
