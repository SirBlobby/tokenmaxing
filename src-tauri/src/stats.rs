use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TokenBucket {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_input_tokens: i64,
    pub cache_creation_input_tokens: i64,
}

impl TokenBucket {
    pub fn add(&mut self, other: &TokenBucket) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.cache_read_input_tokens += other.cache_read_input_tokens;
        self.cache_creation_input_tokens += other.cache_creation_input_tokens;
    }

    pub fn total(&self) -> i64 {
        self.input_tokens
            + self.output_tokens
            + self.cache_read_input_tokens
            + self.cache_creation_input_tokens
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RecentDay {
    pub date: String,
    pub message_count: i64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HistoryPoint {
    pub date: String,
    pub total_tokens: i64,
    pub prompts: i64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UsageStats {
    pub today_prompts: i64,
    pub today_sessions: i64,
    pub today_total_tokens: i64,
    pub today_tokens_by_model: BTreeMap<String, i64>,
    pub today_model_usage: BTreeMap<String, TokenBucket>,
    pub recent_days: Vec<RecentDay>,
    pub history: Vec<HistoryPoint>,
    pub model_usage: BTreeMap<String, TokenBucket>,
    pub total_prompts: i64,
    pub total_sessions: i64,
    pub active_days: i64,
    pub active_dates: Vec<String>,
}

impl UsageStats {
    pub fn total_tokens(&self) -> i64 {
        self.model_usage.values().map(TokenBucket::total).sum()
    }
}

pub fn merge_stats(base: &UsageStats, extra: &UsageStats) -> UsageStats {
    let mut merged = base.clone();
    merged.today_prompts += extra.today_prompts;
    merged.today_sessions += extra.today_sessions;
    merged.today_total_tokens += extra.today_total_tokens;
    merged.total_prompts += extra.total_prompts;
    merged.total_sessions += extra.total_sessions;

    for (model, tokens) in &extra.today_tokens_by_model {
        *merged.today_tokens_by_model.entry(model.clone()).or_insert(0) += tokens;
    }
    for (model, bucket) in &extra.today_model_usage {
        merged.today_model_usage.entry(model.clone()).or_default().add(bucket);
    }
    for (model, bucket) in &extra.model_usage {
        merged.model_usage.entry(model.clone()).or_default().add(bucket);
    }

    let mut tokens_by_date: BTreeMap<String, i64> = BTreeMap::new();
    for day in base.recent_days.iter().chain(extra.recent_days.iter()) {
        if !day.date.is_empty() {
            *tokens_by_date.entry(day.date.clone()).or_insert(0) += day.message_count;
        }
    }
    merged.recent_days = tokens_by_date
        .into_iter()
        .map(|(date, message_count)| RecentDay { date, message_count })
        .collect();

    let combined_dates: std::collections::BTreeSet<String> = base
        .active_dates
        .iter()
        .chain(extra.active_dates.iter())
        .cloned()
        .collect();
    merged.active_days = (combined_dates.len() as i64)
        .max(base.active_days)
        .max(extra.active_days);
    merged.active_dates = combined_dates.into_iter().collect();
    merged
}
