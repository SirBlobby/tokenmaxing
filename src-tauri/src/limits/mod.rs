pub mod codex_rpc;
pub mod credentials;
pub mod probe;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::jsonio::{read_fresh_json, write_json};
use crate::paths::cache_root;
use crate::timeline::{now_milliseconds, parse_timestamp, ParsedTimestamp};

use probe::{probe_limits, ProbeOutcome, RateLimit};

pub const AUTH_HELP: &str = "Run `claude auth login` to restore authoritative usage.";
const PROBE_MIN_INTERVAL_SECONDS: f64 = 15.0;

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct CachedProbe {
    fetched_at_ms: f64,
    limits: Vec<RateLimit>,
}

pub struct LimitsReport {
    pub limits: Vec<RateLimit>,
    pub usage_status_text: String,
    pub auth_help_text: String,
    pub retry_advised: bool,
}

impl Default for LimitsReport {
    fn default() -> Self {
        Self {
            limits: Vec::new(),
            usage_status_text: String::new(),
            auth_help_text: AUTH_HELP.to_string(),
            retry_advised: false,
        }
    }
}

pub fn collect_limits(access_token: &str, expires_at_milliseconds: i64, force: bool) -> LimitsReport {
    let cache_file = cache_root().join("claude-limits.json");
    let cached = read_fresh_json::<CachedProbe>(&cache_file, f64::INFINITY).unwrap_or_default();
    let fallback = open_window_limits(&cached.limits);

    if access_token.is_empty() {
        return LimitsReport {
            limits: fallback,
            usage_status_text: "Waiting for auth".to_string(),
            ..Default::default()
        };
    }

    if expires_at_milliseconds > 0 && (expires_at_milliseconds as f64) <= now_milliseconds() {
        let suffix = if fallback.is_empty() { "." } else { " - showing the last known limits." };
        return LimitsReport {
            limits: fallback,
            usage_status_text: "Sign-in expired".to_string(),
            auth_help_text: format!(
                "Claude Code's saved sign-in expired{suffix} Start Claude Code, or run `claude auth login`, to refresh it."
            ),
            retry_advised: false,
        };
    }

    let seconds_since_probe = (now_milliseconds() - cached.fetched_at_ms) / 1000.0;
    if !fallback.is_empty() && !force && seconds_since_probe < PROBE_MIN_INTERVAL_SECONDS {
        return LimitsReport { limits: fallback, ..Default::default() };
    }

    match probe_limits(access_token) {
        ProbeOutcome::Ready(limits) => {
            let _ = write_json(
                &cache_file,
                &CachedProbe {
                    fetched_at_ms: now_milliseconds().round(),
                    limits: limits.clone(),
                },
            );
            LimitsReport { limits, ..Default::default() }
        }
        ProbeOutcome::Failed { help_text, transport_error } => {
            if !fallback.is_empty() {
                return LimitsReport {
                    limits: fallback,
                    retry_advised: transport_error,
                    ..Default::default()
                };
            }
            LimitsReport {
                limits: Vec::new(),
                usage_status_text: "Claude limits unavailable".to_string(),
                auth_help_text: help_text,
                retry_advised: transport_error,
            }
        }
    }
}

fn open_window_limits(limits: &[RateLimit]) -> Vec<RateLimit> {
    let now = Utc::now();
    limits
        .iter()
        .filter(|limit| window_is_open(limit, now))
        .cloned()
        .collect()
}

fn window_is_open(limit: &RateLimit, now: DateTime<Utc>) -> bool {
    if limit.resets_at.is_empty() {
        return true;
    }
    match parse_timestamp(&limit.resets_at) {
        Some(ParsedTimestamp::Zoned(resets_at)) => resets_at > now,
        Some(ParsedTimestamp::Floating(resets_at)) => resets_at.and_utc() > now,
        None => true,
    }
}
