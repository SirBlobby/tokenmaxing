use serde::{Deserialize, Serialize};

use crate::limits::probe::RateLimit;
use crate::stats::UsageStats;

pub const SCHEMA_VERSION: u32 = 2;

const DEFAULT_CACHE_SECONDS: f64 = 20.0;
const LIMITS_ONLY_CACHE_SECONDS: f64 = 900.0;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RefreshOptions {
    pub force: bool,
    pub limits_only: bool,
}

impl RefreshOptions {
    pub fn scan_age_seconds(&self) -> f64 {
        if self.force {
            0.0
        } else if self.limits_only {
            LIMITS_ONLY_CACHE_SECONDS
        } else {
            DEFAULT_CACHE_SECONDS
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRecord {
    pub id: String,
    pub name: String,
    pub ready: bool,
    pub tier_label: String,
    pub usage_status_text: String,
    pub auth_help_text: String,
    pub limits: Vec<RateLimit>,
    pub retry_advised: bool,
    pub stats: UsageStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceReport {
    pub id: String,
    pub label: String,
    pub description: String,
    pub supported: bool,
    pub enabled: bool,
    pub found: bool,
    pub prompts: i64,
    pub total_tokens: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageReport {
    pub schema_version: u32,
    pub updated_at: String,
    pub agents: Vec<AgentRecord>,
    pub sources: Vec<SourceReport>,
}

pub struct SourceInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub supported: bool,
}

pub const SOURCE_CATALOG: &[SourceInfo] = &[
    SourceInfo {
        id: "claudeCode",
        label: "Claude Code",
        description: "Transcripts under ~/.claude/projects",
        supported: true,
    },
    SourceInfo {
        id: "pi",
        label: "pi and omp",
        description: "Sessions under ~/.pi and ~/.omp",
        supported: true,
    },
    SourceInfo {
        id: "opencode",
        label: "opencode",
        description: "Assistant messages in opencode.db",
        supported: true,
    },
    SourceInfo {
        id: "codex",
        label: "Codex",
        description: "Codex CLI sessions and app-server limits",
        supported: true,
    },
    SourceInfo {
        id: "gemini",
        label: "Gemini",
        description: "",
        supported: false,
    },
    SourceInfo {
        id: "copilot",
        label: "Copilot",
        description: "",
        supported: false,
    },
];
