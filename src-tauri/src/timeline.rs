use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Days, Local, NaiveDate, NaiveDateTime, NaiveTime, SecondsFormat, TimeZone, Utc};
use serde_json::Value;

pub const RECENT_DAY_COUNT: u64 = 7;

pub enum ParsedTimestamp {
    Zoned(DateTime<Utc>),
    Floating(NaiveDateTime),
}

pub fn now_milliseconds() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}

pub fn date_string(day: NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}

pub fn today_string() -> String {
    date_string(Local::now().date_naive())
}

pub fn start_of_today_milliseconds() -> f64 {
    Local::now()
        .date_naive()
        .and_time(NaiveTime::MIN)
        .and_local_timezone(Local)
        .earliest()
        .map(|moment| moment.timestamp_millis() as f64)
        .unwrap_or(0.0)
}

pub fn recent_date_strings() -> Vec<String> {
    let today = Local::now().date_naive();
    (0..RECENT_DAY_COUNT)
        .rev()
        .filter_map(|offset| today.checked_sub_days(Days::new(offset)))
        .map(date_string)
        .collect()
}

pub fn local_date_from_milliseconds(milliseconds: i64) -> Option<String> {
    match Local.timestamp_millis_opt(milliseconds) {
        chrono::LocalResult::Single(moment) => Some(date_string(moment.date_naive())),
        _ => None,
    }
}

pub fn local_date_from_timestamp(value: Option<&Value>) -> String {
    let Some(value) = value else { return today_string() };

    if let Some(numeric) = value.as_f64() {
        let seconds = if numeric > 10_000_000_000.0 { numeric / 1000.0 } else { numeric };
        return local_date_from_milliseconds((seconds * 1000.0).round() as i64)
            .unwrap_or_else(today_string);
    }

    let Some(raw) = value.as_str().map(str::trim) else { return today_string() };
    if raw.is_empty() {
        return today_string();
    }

    match parse_timestamp(raw) {
        Some(ParsedTimestamp::Zoned(moment)) => date_string(moment.with_timezone(&Local).date_naive()),
        Some(ParsedTimestamp::Floating(moment)) => date_string(moment.date()),
        None => today_string(),
    }
}

pub fn parse_timestamp(raw: &str) -> Option<ParsedTimestamp> {
    let normalized = raw.replace('Z', "+00:00");

    if let Ok(moment) = DateTime::parse_from_rfc3339(&normalized) {
        return Some(ParsedTimestamp::Zoned(moment.with_timezone(&Utc)));
    }
    for format in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M"] {
        if let Ok(moment) = NaiveDateTime::parse_from_str(&normalized, format) {
            return Some(ParsedTimestamp::Floating(moment));
        }
    }
    NaiveDate::parse_from_str(&normalized, "%Y-%m-%d")
        .ok()
        .and_then(|day| day.and_hms_opt(0, 0, 0))
        .map(ParsedTimestamp::Floating)
}

pub fn format_timestamp(moment: &ParsedTimestamp) -> String {
    match moment {
        ParsedTimestamp::Zoned(zoned) => zoned.to_rfc3339_opts(SecondsFormat::AutoSi, false),
        ParsedTimestamp::Floating(floating) => floating.format("%Y-%m-%dT%H:%M:%S").to_string(),
    }
}

pub fn utc_now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::AutoSi, false)
}
