use crate::commands::{print_success, resolve_id};
use crate::idish::IDish;
use crate::models::EventType;
use crate::repository::ChangeRepository;
use anyhow::Result;
use colored::Colorize;

pub async fn timer_start(id: Option<IDish>) -> Result<()> {
    let full_id = resolve_id(id).await?;
    let mut repo = ChangeRepository::open().await?;

    // Check if timer is already running first
    let already_running = repo
        .find_by_id(&full_id)
        .map(|c| c.is_timer_running())
        .unwrap_or(false);

    if already_running {
        let change = repo.find_by_id(&full_id).unwrap();
        println!(
            "{} Timer is already running for change {}",
            "→".blue(),
            full_id.to_string().cyan()
        );
        println!(
            "  Started: {} ({} ago)",
            change
                .timer_start
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S UTC"),
            format_duration(change.total_duration_secs())
        );
        return Ok(());
    }

    // Start the timer
    let started = if let Some(change) = repo.find_by_id_mut(&full_id) {
        change.timer_start()
    } else {
        return Err(anyhow::anyhow!("Change '{}' not found", full_id));
    };

    if started {
        // Add event
        let event = crate::models::Event::new(
            full_id.clone(),
            EventType::TimerStarted,
            serde_json::json!({}),
        );
        repo.db.add_event(event);
        repo.save().await?;

        print_success(&format!("Started timer for change {}", full_id));
    }

    Ok(())
}

pub async fn timer_stop(id: Option<IDish>) -> Result<()> {
    let full_id = resolve_id(id).await?;
    let mut repo = ChangeRepository::open().await?;

    // Get info before stopping
    let (was_running, total_after_stop) = if let Some(change) = repo.find_by_id(&full_id) {
        let was_running = change.is_timer_running();
        let total = change.total_duration_secs();
        (was_running, total)
    } else {
        return Err(anyhow::anyhow!("Change '{}' not found", full_id));
    };

    // Stop the timer
    let session_duration = if let Some(change) = repo.find_by_id_mut(&full_id) {
        change.timer_stop()
    } else {
        return Err(anyhow::anyhow!("Change '{}' not found", full_id));
    };

    if let Some(secs) = session_duration {
        // Add event
        let event = crate::models::Event::new(
            full_id.clone(),
            EventType::TimerStopped,
            serde_json::json!({"duration_secs": secs}),
        );
        repo.db.add_event(event);
        repo.save().await?;

        print_success(&format!(
            "Stopped timer for change {} (session: {}, total: {})",
            full_id,
            format_duration(secs),
            format_duration(total_after_stop)
        ));
    } else if was_running {
        repo.save().await?;
        println!(
            "{} Timer stopped for change {}",
            "✓".green(),
            full_id.to_string().cyan()
        );
        println!("  Total time: {}", format_duration(total_after_stop));
    } else {
        println!(
            "{} Timer was not running for change {}",
            "→".blue(),
            full_id.to_string().cyan()
        );
        println!("  Total time: {}", format_duration(total_after_stop));
    }

    Ok(())
}

pub async fn timer_status(id: Option<IDish>) -> Result<()> {
    let full_id = resolve_id(id).await?;
    let repo = ChangeRepository::open().await?;

    let change = repo
        .find_by_id(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

    println!("\n{}", format!("Timer status for change {}", full_id).bold());
    println!("{}", "─".repeat(60).dimmed());

    if change.is_timer_running() {
        let start = change.timer_start.unwrap();
        let elapsed = change.total_duration_secs();
        println!("\n  Status: {}", "RUNNING".green().bold());
        println!(
            "  Started: {}",
            start.format("%Y-%m-%d %H:%M:%S UTC").to_string().cyan()
        );
        println!(
            "  Elapsed this session: {}",
            format_duration(elapsed - change.accumulated_duration_secs).cyan()
        );
        println!(
            "  Total accumulated: {}",
            format_duration(change.accumulated_duration_secs).cyan()
        );
        println!("  Total time: {}", format_duration(elapsed).cyan().bold());
    } else {
        println!("\n  Status: {}", "STOPPED".dimmed());
        if change.accumulated_duration_secs > 0 {
            println!(
                "  Total time: {}",
                format_duration(change.accumulated_duration_secs).cyan()
            );
        } else {
            println!("  No time recorded yet.");
        }
    }

    println!();
    Ok(())
}

fn format_duration(total_secs: i64) -> String {
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
