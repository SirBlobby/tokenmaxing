use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::stats::{RecentDay, TokenBucket, UsageStats};
use crate::timeline::{recent_date_strings, today_string};

pub struct UsageTally {
    pub today: String,
    recent_tokens: BTreeMap<String, i64>,
    recent_order: Vec<String>,
    seen_messages: HashSet<String>,
    sessions: HashSet<String>,
    active_dates: BTreeSet<String>,
    today_sessions: HashSet<String>,
    today_tokens_by_model: BTreeMap<String, i64>,
    today_usage_by_model: BTreeMap<String, TokenBucket>,
    usage_by_model: BTreeMap<String, TokenBucket>,
    prompts: i64,
    today_prompts: i64,
    today_total_tokens: i64,
}

impl UsageTally {
    pub fn new() -> Self {
        let recent_order = recent_date_strings();
        let recent_tokens = recent_order.iter().map(|date| (date.clone(), 0)).collect();
        Self {
            today: today_string(),
            recent_tokens,
            recent_order,
            seen_messages: HashSet::new(),
            sessions: HashSet::new(),
            active_dates: BTreeSet::new(),
            today_sessions: HashSet::new(),
            today_tokens_by_model: BTreeMap::new(),
            today_usage_by_model: BTreeMap::new(),
            usage_by_model: BTreeMap::new(),
            prompts: 0,
            today_prompts: 0,
            today_total_tokens: 0,
        }
    }

    pub fn is_new_message(&mut self, key: String) -> bool {
        self.seen_messages.insert(key)
    }

    pub fn record(&mut self, model: &str, day: &str, session_key: &str, bucket: TokenBucket) {
        let total = bucket.total();
        self.sessions.insert(session_key.to_string());
        self.active_dates.insert(day.to_string());
        self.prompts += 1;
        self.usage_by_model.entry(model.to_string()).or_default().add(&bucket);

        if let Some(slot) = self.recent_tokens.get_mut(day) {
            *slot += total;
        }
        if day == self.today {
            self.today_prompts += 1;
            self.today_sessions.insert(session_key.to_string());
            self.today_total_tokens += total;
            *self.today_tokens_by_model.entry(model.to_string()).or_insert(0) += total;
            self.today_usage_by_model.entry(model.to_string()).or_default().add(&bucket);
        }
    }

    pub fn finish(self) -> UsageStats {
        let recent_days = self
            .recent_order
            .iter()
            .map(|date| RecentDay {
                date: date.clone(),
                message_count: self.recent_tokens.get(date).copied().unwrap_or(0),
            })
            .collect();

        UsageStats {
            today_prompts: self.today_prompts,
            today_sessions: self.today_sessions.len() as i64,
            today_total_tokens: self.today_total_tokens,
            today_tokens_by_model: self.today_tokens_by_model,
            today_model_usage: self.today_usage_by_model,
            recent_days,
            history: Vec::new(),
            model_usage: self.usage_by_model,
            total_prompts: self.prompts,
            total_sessions: self.sessions.len() as i64,
            active_days: self.active_dates.len() as i64,
            active_dates: self.active_dates.into_iter().collect(),
        }
    }

    pub fn into_stats_if_used(self) -> Option<UsageStats> {
        (self.prompts > 0).then(|| self.finish())
    }
}
