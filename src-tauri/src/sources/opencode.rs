use std::time::Duration;

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::jsonio::{number, object_or_empty, read_fresh_json, write_json};
use crate::paths::{cache_root, opencode_database, short_digest};
use crate::stats::{TokenBucket, UsageStats};
use crate::tally::UsageTally;
use crate::timeline::local_date_from_milliseconds;

#[derive(Serialize, Deserialize, Default)]
struct CachedScan {
    stats: Option<UsageStats>,
}

pub fn scan_opencode_usage(provider_id: &str, default_model: &str, max_age_seconds: f64) -> Option<UsageStats> {
    let database = opencode_database()?;
    let cache_file = cache_root().join(format!(
        "opencode-{provider_id}-{}.json",
        short_digest(&database.to_string_lossy())
    ));
    if let Some(cached) = read_fresh_json::<CachedScan>(&cache_file, max_age_seconds) {
        return cached.stats;
    }

    let connection = Connection::open_with_flags(&database, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;
    let _ = connection.busy_timeout(Duration::from_secs(2));
    let _ = connection.execute_batch("PRAGMA query_only = ON");

    let mut tally = UsageTally::new();
    {
        let mut statement = connection.prepare(SELECT_ASSISTANT_MESSAGES).ok()?;
        let mut rows = statement.query([provider_id]).ok()?;

        while let Ok(Some(row)) = rows.next() {
            let session_id: String = row.get(0).unwrap_or_default();
            let Ok(raw) = row.get::<_, String>(1) else { continue };
            let Ok(entry) = serde_json::from_str::<Value>(&raw) else { continue };
            if !is_assistant_from(&entry, provider_id) {
                continue;
            }

            let bucket = message_tokens(&entry);
            if bucket.total() <= 0 {
                continue;
            }

            let created = number(entry.get("time").and_then(|time| time.get("created")));
            let day = (created > 0)
                .then(|| local_date_from_milliseconds(created))
                .flatten()
                .unwrap_or_else(|| tally.today.clone());

            tally.record(
                &model_name(&entry, default_model),
                &day,
                &format!("opencode:{session_id}"),
                bucket,
            );
        }
    }

    let stats = tally.into_stats_if_used();
    let _ = write_json(&cache_file, &CachedScan { stats: stats.clone() });
    stats
}

const SELECT_ASSISTANT_MESSAGES: &str = "SELECT session_id, data FROM message \
     WHERE data LIKE '%\"role\"%:%\"assistant\"%' \
     AND CASE WHEN json_valid(data) THEN json_extract(data, '$.role') END = 'assistant' \
     AND CASE WHEN json_valid(data) THEN json_extract(data, '$.providerID') END = ?1";

fn is_assistant_from(entry: &Value, provider_id: &str) -> bool {
    entry.is_object()
        && entry.get("role").and_then(Value::as_str) == Some("assistant")
        && entry.get("providerID").and_then(Value::as_str) == Some(provider_id)
}

fn message_tokens(entry: &Value) -> TokenBucket {
    let empty = Value::Object(Default::default());
    let tokens = object_or_empty(Some(entry), "tokens", &empty);
    let cache = object_or_empty(Some(tokens), "cache", &empty);

    TokenBucket {
        input_tokens: number(tokens.get("input")),
        output_tokens: number(tokens.get("output")) + number(tokens.get("reasoning")),
        cache_read_input_tokens: number(cache.get("read")),
        cache_creation_input_tokens: number(cache.get("write")),
    }
}

fn model_name(entry: &Value, fallback: &str) -> String {
    entry
        .get("modelID")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_string()
}
