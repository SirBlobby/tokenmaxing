use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::jsonio::{read_fresh_json, usage_token, write_json};
use crate::paths::{cache_root, jsonl_files, short_digest};
use crate::stats::{TokenBucket, UsageStats};
use crate::tally::UsageTally;
use crate::timeline::local_date_from_timestamp;

pub fn scan_projects(projects_path: &Path) -> UsageStats {
    let mut tally = UsageTally::new();

    for path in jsonl_files(projects_path) {
        let Ok(file) = File::open(&path) else { continue };
        let file_label = path.to_string_lossy().to_string();

        for (offset, line) in BufReader::new(file).lines().enumerate() {
            let Ok(line) = line else { continue };
            if !line.contains("\"usage\":") {
                continue;
            }
            let Ok(entry) = serde_json::from_str::<Value>(&line) else { continue };
            let Some(bucket) = assistant_tokens(&entry) else { continue };

            let message = entry.get("message").filter(|value| value.is_object());
            let unique_key = message_key(&entry, message, &file_label, offset + 1);
            if !tally.is_new_message(unique_key) {
                continue;
            }
            if bucket.total() <= 0 {
                continue;
            }

            let model = message
                .and_then(|value| value.get("model"))
                .or_else(|| entry.get("model"))
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .unwrap_or("claude")
                .to_string();
            let day = local_date_from_timestamp(
                entry
                    .get("timestamp")
                    .or_else(|| message.and_then(|value| value.get("timestamp"))),
            );
            let session_key = entry
                .get("sessionId")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| file_label.clone());

            tally.record(&model, &day, &session_key, bucket);
        }
    }

    tally.finish()
}

fn assistant_tokens(entry: &Value) -> Option<TokenBucket> {
    let message = entry.get("message").filter(|value| value.is_object());
    let entry_type = entry.get("type").and_then(Value::as_str).unwrap_or_default();
    let role = message
        .and_then(|value| value.get("role"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if entry_type != "assistant" && role != "assistant" {
        return None;
    }

    let usage = message
        .and_then(|value| value.get("usage"))
        .or_else(|| entry.get("usage"))
        .filter(|value| value.is_object())?;

    Some(TokenBucket {
        input_tokens: usage_token(usage, "input_tokens", "inputTokens"),
        output_tokens: usage_token(usage, "output_tokens", "outputTokens"),
        cache_read_input_tokens: usage_token(usage, "cache_read_input_tokens", "cacheReadInputTokens"),
        cache_creation_input_tokens: usage_token(
            usage,
            "cache_creation_input_tokens",
            "cacheCreationInputTokens",
        ),
    })
}

fn message_key(entry: &Value, message: Option<&Value>, file_label: &str, line_number: usize) -> String {
    let message_id = message
        .and_then(|value| value.get("id"))
        .or_else(|| entry.get("messageId"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !message_id.is_empty() {
        return message_id.to_string();
    }

    let suffix = entry
        .get("uuid")
        .or_else(|| entry.get("requestId"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| line_number.to_string());
    format!("{file_label}:{suffix}")
}

fn scan_cache_paths(projects_path: &Path) -> (PathBuf, PathBuf) {
    let digest = short_digest(&projects_path.to_string_lossy());
    let root = cache_root();
    (
        root.join(format!("claude-scan-{digest}.json")),
        root.join(format!("claude-scan-{digest}.lock")),
    )
}

pub fn cached_scan(projects_path: &Path, max_age_seconds: f64) -> UsageStats {
    let (cache_file, lock_file) = scan_cache_paths(projects_path);

    if let Some(cached) = read_fresh_json::<UsageStats>(&cache_file, max_age_seconds) {
        return cached;
    }

    let guard = File::create(&lock_file).ok();
    if let Some(guard) = &guard {
        let _ = guard.lock();
    }

    let summary = match read_fresh_json::<UsageStats>(&cache_file, max_age_seconds) {
        Some(cached) => cached,
        None => {
            let scanned = scan_projects(projects_path);
            let _ = write_json(&cache_file, &scanned);
            scanned
        }
    };

    if let Some(guard) = &guard {
        let _ = guard.unlock();
    }
    summary
}
