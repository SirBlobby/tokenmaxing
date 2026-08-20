use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use serde_json::Value;

use crate::jsonio::number;
use crate::stats::{RecentDay, UsageStats};
use crate::timeline::{start_of_today_milliseconds, today_string};

pub fn stats_cache_fallback(claude_dir: &Path) -> Option<UsageStats> {
    let text = std::fs::read_to_string(claude_dir.join("stats-cache.json")).ok()?;
    let data: Value = serde_json::from_str(&text).ok()?;

    let today_tokens = today_tokens_by_model(&data);
    let daily_activity = object_list(&data, "dailyActivity");
    let active_dates = dates_with_activity(&daily_activity);
    let (today_prompts, today_sessions) = today_prompts_from_history(claude_dir);

    Some(UsageStats {
        today_prompts,
        today_sessions,
        today_total_tokens: today_tokens.values().sum(),
        today_tokens_by_model: today_tokens,
        today_model_usage: BTreeMap::new(),
        recent_days: last_seven_days(&daily_activity),
        history: Vec::new(),
        model_usage: data
            .get("modelUsage")
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default(),
        total_prompts: number(data.get("totalMessages")),
        total_sessions: number(data.get("totalSessions")),
        active_days: active_dates.len() as i64,
        active_dates,
    })
}

fn object_list<'a>(data: &'a Value, key: &str) -> Vec<&'a Value> {
    data.get(key)
        .and_then(Value::as_array)
        .map(|entries| entries.iter().filter(|entry| entry.is_object()).collect())
        .unwrap_or_default()
}

fn today_tokens_by_model(data: &Value) -> BTreeMap<String, i64> {
    let today = today_string();
    object_list(data, "dailyModelTokens")
        .into_iter()
        .find(|entry| entry.get("date").and_then(Value::as_str) == Some(today.as_str()))
        .and_then(|entry| entry.get("tokensByModel"))
        .and_then(Value::as_object)
        .map(|tokens| {
            tokens
                .iter()
                .map(|(model, value)| (model.clone(), number(Some(value))))
                .collect()
        })
        .unwrap_or_default()
}

fn dates_with_activity(daily_activity: &[&Value]) -> Vec<String> {
    let mut dates: Vec<String> = daily_activity
        .iter()
        .filter(|day| number(day.get("messageCount")) > 0)
        .filter_map(|day| day.get("date").and_then(Value::as_str).map(str::to_string))
        .collect();
    dates.sort();
    dates.dedup();
    dates
}

fn last_seven_days(daily_activity: &[&Value]) -> Vec<RecentDay> {
    daily_activity
        .iter()
        .skip(daily_activity.len().saturating_sub(7))
        .map(|day| RecentDay {
            date: day.get("date").and_then(Value::as_str).unwrap_or_default().to_string(),
            message_count: number(day.get("messageCount")),
        })
        .collect()
}

pub fn today_prompts_from_history(claude_dir: &Path) -> (i64, i64) {
    let Ok(text) = std::fs::read_to_string(claude_dir.join("history.jsonl")) else {
        return (0, 0);
    };

    let start_of_day = start_of_today_milliseconds();
    let mut prompts = 0;
    let mut sessions: HashSet<String> = HashSet::new();

    for line in text.lines().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(entry) = serde_json::from_str::<Value>(line) else { continue };
        if (number(entry.get("timestamp")) as f64) < start_of_day {
            break;
        }
        prompts += 1;
        if let Some(session) = entry.get("sessionId").and_then(Value::as_str) {
            sessions.insert(session.to_string());
        }
    }

    (prompts, sessions.len() as i64)
}
