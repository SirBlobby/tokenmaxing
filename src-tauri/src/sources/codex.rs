use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::jsonio::{number, read_fresh_json, write_json};
use crate::paths::{cache_root, codex_session_roots, jsonl_files, modified_milliseconds, modified_within};
use crate::stats::{TokenBucket, UsageStats};
use crate::tally::UsageTally;
use crate::timeline::{local_date_from_milliseconds, local_date_from_timestamp, today_string};

const SESSION_MAX_AGE_SECONDS: u64 = 30 * 24 * 60 * 60;
const DEFAULT_MODEL: &str = "codex";

#[derive(Serialize, Deserialize, Default)]
struct CachedScan {
    stats: Option<UsageStats>,
}

pub fn scan_codex_usage(max_age_seconds: f64) -> Option<UsageStats> {
    let cache_file = cache_root().join("codex-native-sessions.json");
    if let Some(cached) = read_fresh_json::<CachedScan>(&cache_file, max_age_seconds) {
        return cached.stats;
    }

    let mut tally = UsageTally::new();
    for root in codex_session_roots() {
        for path in jsonl_files(&root) {
            if !modified_within(&path, SESSION_MAX_AGE_SECONDS) {
                continue;
            }
            scan_session_file(&path, &mut tally);
        }
    }

    let stats = tally.into_stats_if_used();
    let _ = write_json(&cache_file, &CachedScan { stats: stats.clone() });
    stats
}

fn scan_session_file(path: &Path, tally: &mut UsageTally) {
    let Ok(file) = File::open(path) else { return };
    let file_label = path.to_string_lossy().to_string();
    let mut current_model = DEFAULT_MODEL.to_string();

    for line in BufReader::new(file).lines() {
        let Ok(line) = line else { continue };
        let Ok(entry) = serde_json::from_str::<Value>(&line) else { continue };
        let entry_type = entry.get("type").and_then(Value::as_str).unwrap_or_default();

        if entry_type == "turn_context" {
            if let Some(model) = turn_context_model(&entry) {
                current_model = model;
            }
            continue;
        }

        let Some(usage) = last_turn_usage(&entry, entry_type) else { continue };
        let bucket = turn_tokens(usage);
        if bucket.total() <= 0 {
            continue;
        }

        let day = turn_day(&entry, path);
        tally.record(&current_model, &day, &file_label, bucket);
    }
}

fn turn_context_model(entry: &Value) -> Option<String> {
    let payload = entry.get("payload")?;
    payload
        .get("model")
        .or_else(|| payload.get("model_slug"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn last_turn_usage<'a>(entry: &'a Value, entry_type: &str) -> Option<&'a Value> {
    let mut payload = non_empty_object(entry.get("payload")).unwrap_or(entry);
    if entry_type == "response_item" {
        payload = non_empty_object(payload.get("payload")).unwrap_or(payload);
    }
    if payload.get("type").and_then(Value::as_str) != Some("token_count") {
        return None;
    }
    payload.get("info")?.get("last_token_usage")
}

fn non_empty_object(value: Option<&Value>) -> Option<&Value> {
    value.filter(|inner| match inner {
        Value::Object(map) => !map.is_empty(),
        Value::Null => false,
        _ => true,
    })
}

fn turn_tokens(usage: &Value) -> TokenBucket {
    let cache_read = number(usage.get("cached_input_tokens"));
    let cache_write = number(usage.get("cache_write_input_tokens"));
    let reported_input = number(usage.get("input_tokens"));

    TokenBucket {
        input_tokens: (reported_input - cache_read - cache_write).max(0),
        output_tokens: number(usage.get("output_tokens")),
        cache_read_input_tokens: cache_read,
        cache_creation_input_tokens: cache_write,
    }
}

fn turn_day(entry: &Value, path: &Path) -> String {
    let timestamp = entry.get("timestamp").filter(|value| match value {
        Value::String(text) => !text.trim().is_empty(),
        Value::Number(_) => true,
        _ => false,
    });
    if timestamp.is_some() {
        return local_date_from_timestamp(timestamp);
    }
    modified_milliseconds(path)
        .and_then(local_date_from_milliseconds)
        .unwrap_or_else(today_string)
}
