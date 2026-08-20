use std::fs::File;
use std::io::{BufRead, BufReader};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::jsonio::{number, object_or_empty, read_fresh_json, usage_token, write_json};
use crate::paths::{cache_root, jsonl_files, pi_session_roots};
use crate::stats::{TokenBucket, UsageStats};
use crate::tally::UsageTally;
use crate::timeline::local_date_from_timestamp;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PiProvider {
    Anthropic,
    OpenAiCodex,
}

impl PiProvider {
    fn cache_name(self) -> &'static str {
        match self {
            PiProvider::Anthropic => "claude-pi-sessions.json",
            PiProvider::OpenAiCodex => "codex-pi-sessions.json",
        }
    }

    fn matches(self, message: &Value) -> bool {
        let provider = message.get("provider").and_then(Value::as_str).unwrap_or_default();
        match self {
            PiProvider::Anthropic => provider == "anthropic",
            PiProvider::OpenAiCodex => {
                let api = message.get("api").and_then(Value::as_str).unwrap_or_default();
                provider == "openai-codex" || api.starts_with("openai-codex")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
struct CachedScan {
    stats: Option<UsageStats>,
}

pub fn scan_pi_usage(provider: PiProvider, max_age_seconds: f64) -> Option<UsageStats> {
    let cache_file = cache_root().join(provider.cache_name());
    if let Some(cached) = read_fresh_json::<CachedScan>(&cache_file, max_age_seconds) {
        return cached.stats;
    }

    let mut tally = UsageTally::new();
    for root in pi_session_roots() {
        for path in jsonl_files(&root) {
            let Ok(file) = File::open(&path) else { continue };
            let file_label = path.to_string_lossy().to_string();

            for (offset, line) in BufReader::new(file).lines().enumerate() {
                let Ok(line) = line else { continue };
                if !line.contains("\"usage\"") || !line.contains("\"assistant\"") {
                    continue;
                }
                let Ok(entry) = serde_json::from_str::<Value>(&line) else { continue };
                if !is_matching_assistant(&entry, provider) {
                    continue;
                }

                let suffix = entry
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| (offset + 1).to_string());
                if !tally.is_new_message(format!("{file_label}:{suffix}")) {
                    continue;
                }

                let message = entry.get("message");
                let empty = Value::Object(Default::default());
                let usage = object_or_empty(message, "usage", &empty);
                let bucket = session_tokens(usage);
                if bucket.total() <= 0 {
                    continue;
                }

                let model = message
                    .and_then(|value| value.get("model"))
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                    .unwrap_or(default_model(provider))
                    .to_string();
                let day = local_date_from_timestamp(
                    entry
                        .get("timestamp")
                        .or_else(|| message.and_then(|value| value.get("timestamp"))),
                );

                tally.record(&model, &day, &file_label, bucket);
            }
        }
    }

    let stats = tally.into_stats_if_used();
    let _ = write_json(&cache_file, &CachedScan { stats: stats.clone() });
    stats
}

fn default_model(provider: PiProvider) -> &'static str {
    match provider {
        PiProvider::Anthropic => "claude",
        PiProvider::OpenAiCodex => "codex",
    }
}

fn is_matching_assistant(entry: &Value, provider: PiProvider) -> bool {
    if entry.get("type").and_then(Value::as_str) != Some("message") {
        return false;
    }
    let Some(message) = entry.get("message").filter(|value| value.is_object()) else {
        return false;
    };
    message.get("role").and_then(Value::as_str) == Some("assistant") && provider.matches(message)
}

fn session_tokens(usage: &Value) -> TokenBucket {
    let bucket = TokenBucket {
        input_tokens: usage_token(usage, "input", "inputTokens"),
        output_tokens: usage_token(usage, "output", "outputTokens"),
        cache_read_input_tokens: usage_token(usage, "cacheRead", "cache_read_input_tokens"),
        cache_creation_input_tokens: usage_token(usage, "cacheWrite", "cache_creation_input_tokens"),
    };
    if bucket.total() > 0 {
        return bucket;
    }
    TokenBucket {
        input_tokens: number(usage.get("totalTokens")),
        ..Default::default()
    }
}
