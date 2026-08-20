use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::timeline::{format_timestamp, parse_timestamp};

pub const USAGE_ENDPOINT: &str = "https://api.anthropic.com/api/oauth/usage";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RateLimit {
    pub label: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub title: String,
    pub percent: f64,
    pub resets_at: String,
}

pub enum ProbeOutcome {
    Ready(Vec<RateLimit>),
    Failed { help_text: String, transport_error: bool },
}

pub fn probe_limits(access_token: &str) -> ProbeOutcome {
    let response = ureq::get(USAGE_ENDPOINT)
        .set("Authorization", &format!("Bearer {access_token}"))
        .set("anthropic-beta", "oauth-2025-04-20")
        .set("Accept", "application/json")
        .timeout(REQUEST_TIMEOUT)
        .call();

    let payload = match response {
        Ok(response) => match read_json_body(response) {
            Some(payload) => payload,
            None => {
                return ProbeOutcome::Failed {
                    help_text: "Anthropic's usage endpoint returned an unreadable response. Local Claude Code stats are still shown.".to_string(),
                    transport_error: false,
                }
            }
        },
        Err(ureq::Error::Status(code, response)) => {
            return ProbeOutcome::Failed {
                help_text: status_help_text(code, response.header("retry-after")),
                transport_error: false,
            }
        }
        Err(_) => {
            return ProbeOutcome::Failed {
                help_text: "Couldn't reach Anthropic's usage endpoint. Retrying shortly. Local Claude Code stats are still shown.".to_string(),
                transport_error: true,
            }
        }
    };

    let limits = read_limits(&payload);
    if limits.is_empty() {
        return ProbeOutcome::Failed {
            help_text: "Anthropic's usage endpoint returned no limits. Local Claude Code stats are still shown.".to_string(),
            transport_error: false,
        };
    }
    ProbeOutcome::Ready(limits)
}

fn read_json_body(response: ureq::Response) -> Option<Value> {
    let body = response.into_string().ok()?;
    serde_json::from_str::<Value>(&body).ok()
}

fn status_help_text(code: u16, retry_after: Option<&str>) -> String {
    if code != 429 {
        return format!("Anthropic's usage endpoint returned status {code}. Local Claude Code stats are still shown.");
    }
    let wait = match retry_after {
        Some(seconds) if !seconds.is_empty() => format!(" (retry after {seconds}s)"),
        _ => String::new(),
    };
    format!("Anthropic's usage endpoint is rate limiting checks right now{wait}. Local Claude Code stats are still shown.")
}

fn read_limits(payload: &Value) -> Vec<RateLimit> {
    let weekly = usage_bucket(payload, "seven_day_oauth_apps").or_else(|| usage_bucket(payload, "seven_day"));
    let session = usage_bucket(payload, "five_hour");
    let percent_scale = uses_percent_scale(payload, session, weekly);

    let mut limits = Vec::new();
    if let Some(session) = session {
        push_bucket(&mut limits, "Session (5-hour)", session, percent_scale);
    }
    if let Some(weekly) = weekly {
        push_bucket(&mut limits, "Weekly (7-day)", weekly, percent_scale);
    }
    limits.extend(scoped_limits(payload, percent_scale));
    limits
}

fn usage_bucket<'a>(payload: &'a Value, key: &str) -> Option<&'a Value> {
    payload.get(key).filter(|value| value.is_object())
}

fn uses_percent_scale(payload: &Value, session: Option<&Value>, weekly: Option<&Value>) -> bool {
    let mut samples: Vec<Option<&Value>> = vec![
        session.and_then(|bucket| bucket.get("utilization")),
        weekly.and_then(|bucket| bucket.get("utilization")),
    ];
    if let Some(entries) = payload.get("limits").and_then(Value::as_array) {
        samples.extend(entries.iter().map(|entry| entry.get("percent")));
    }
    samples
        .into_iter()
        .flatten()
        .any(|value| parse_utilization(value).is_some_and(|number| number >= 1.0))
}

fn push_bucket(limits: &mut Vec<RateLimit>, label: &str, bucket: &Value, percent_scale: bool) {
    let Some(percent) = normalize_utilization(bucket.get("utilization"), percent_scale) else {
        return;
    };
    limits.push(RateLimit {
        label: label.to_string(),
        title: String::new(),
        percent,
        resets_at: normalize_reset_at(bucket.get("resets_at")),
    });
}

fn scoped_limits(payload: &Value, percent_scale: bool) -> Vec<RateLimit> {
    let Some(entries) = payload.get("limits").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut limits = Vec::new();
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();

    for entry in entries {
        let Some(model) = entry
            .get("scope")
            .and_then(|scope| scope.get("model"))
            .filter(|model| model.is_object())
        else {
            continue;
        };
        let name = model
            .get("display_name")
            .or_else(|| model.get("id"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let kind = entry.get("kind").and_then(Value::as_str).unwrap_or_default().trim().to_string();
        if name.is_empty() || !seen.insert((name.clone(), kind.clone())) {
            continue;
        }
        let Some(percent) = normalize_utilization(entry.get("percent"), percent_scale) else {
            continue;
        };

        let window = scoped_window(&kind);
        let title = if window.is_empty() { name } else { format!("{name} {window}") };
        limits.push(RateLimit {
            label: title.clone(),
            title,
            percent,
            resets_at: normalize_reset_at(entry.get("resets_at")),
        });
    }
    limits
}

fn scoped_window(kind: &str) -> &'static str {
    let text = kind.to_lowercase();
    if text.contains("month") {
        return "Monthly";
    }
    if text.contains("week") || text.contains("day") {
        return "Weekly";
    }
    if text.contains("hour") || text.contains("session") {
        return "Session";
    }
    ""
}

pub fn parse_utilization(value: &Value) -> Option<f64> {
    let raw = match value {
        Value::Number(number) => return number.as_f64(),
        Value::String(text) => text.trim().replace('%', ""),
        _ => return None,
    };
    raw.parse::<f64>().ok()
}

fn normalize_utilization(value: Option<&Value>, percent_scale: bool) -> Option<f64> {
    let number = value.and_then(parse_utilization)?;
    if !(number >= 0.0) {
        return None;
    }
    if percent_scale || number > 1.0 {
        return Some((number / 100.0).min(1.0));
    }
    Some(number.min(1.0))
}

pub fn normalize_reset_at(value: Option<&Value>) -> String {
    let Some(value) = value else { return String::new() };
    if let Some(number) = value.as_i64() {
        return from_epoch(number);
    }

    let raw = value.as_str().unwrap_or_default().trim();
    if raw.is_empty() {
        return String::new();
    }
    if raw.chars().all(|character| character.is_ascii_digit()) {
        return raw.parse::<i64>().map(from_epoch).unwrap_or_else(|_| raw.to_string());
    }
    parse_timestamp(raw)
        .map(|moment| format_timestamp(&moment))
        .unwrap_or_else(|| raw.to_string())
}

fn from_epoch(value: i64) -> String {
    let milliseconds = if value < 1_000_000_000_000 { value * 1000 } else { value };
    chrono::DateTime::from_timestamp_millis(milliseconds)
        .map(|moment| moment.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, false))
        .unwrap_or_else(|| value.to_string())
}
