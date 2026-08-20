use std::path::PathBuf;

use crate::jsonio::write_json;
use crate::paths::cache_root;
use crate::stats::{HistoryPoint, RecentDay};

const MAX_HISTORY_DAYS: usize = 90;

fn history_path(agent_id: &str) -> PathBuf {
    cache_root().join("history").join(format!("{agent_id}.json"))
}

fn read_points(path: &PathBuf) -> Vec<HistoryPoint> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

/// Upserts today's totals into the agent's on-disk history and returns the
/// full (capped) series. On first run the series is seeded from the days
/// already visible in the live transcript scan, so the chart isn't empty.
pub fn record_today(agent_id: &str, date: &str, total_tokens: i64, prompts: i64, bootstrap: &[RecentDay]) -> Vec<HistoryPoint> {
    let path = history_path(agent_id);
    let mut points = read_points(&path);

    if points.is_empty() {
        points = bootstrap
            .iter()
            .filter(|day| !day.date.is_empty() && day.date != date)
            .map(|day| HistoryPoint {
                date: day.date.clone(),
                total_tokens: day.message_count,
                prompts: 0,
            })
            .collect();
    }

    match points.iter_mut().find(|point| point.date == date) {
        Some(existing) => {
            existing.total_tokens = total_tokens;
            existing.prompts = prompts;
        }
        None => points.push(HistoryPoint {
            date: date.to_string(),
            total_tokens,
            prompts,
        }),
    }

    points.sort_by(|a, b| a.date.cmp(&b.date));
    if points.len() > MAX_HISTORY_DAYS {
        let excess = points.len() - MAX_HISTORY_DAYS;
        points.drain(0..excess);
    }

    let _ = write_json(&path, &points);
    points
}
