use std::path::Path;

use crate::limits::collect_limits;
use crate::limits::credentials::oauth_login;
use crate::paths::config_dir;
use crate::report::{AgentRecord, RefreshOptions};
use crate::settings::Settings;
use crate::sources::fallback::{stats_cache_fallback, today_prompts_from_history};
use crate::sources::opencode::scan_opencode_usage;
use crate::sources::pi::{scan_pi_usage, PiProvider};
use crate::sources::transcripts::cached_scan;
use crate::stats::UsageStats;

use super::{merge_all, AgentScan, SourceScan};

pub const AGENT_ID: &str = "claude";
pub const AGENT_NAME: &str = "Claude Code";

const OPENCODE_PROVIDER: &str = "anthropic";
const OPENCODE_DEFAULT_MODEL: &str = "claude";

pub fn is_enabled(settings: &Settings) -> bool {
    settings.sources.claude_code || settings.sources.pi || settings.sources.opencode
}

pub fn scan(options: RefreshOptions, settings: &Settings) -> AgentScan {
    let claude_dir = config_dir();
    let scan_age = options.scan_age_seconds();

    let transcripts = settings
        .sources
        .claude_code
        .then(|| transcript_stats(&claude_dir, scan_age))
        .flatten();
    let pi = settings
        .sources
        .pi
        .then(|| scan_pi_usage(PiProvider::Anthropic, scan_age))
        .flatten();
    let opencode = settings
        .sources
        .opencode
        .then(|| scan_opencode_usage(OPENCODE_PROVIDER, OPENCODE_DEFAULT_MODEL, scan_age))
        .flatten();

    let stats = merge_all(&[transcripts.clone(), pi.clone(), opencode.clone()]);

    let login = oauth_login(&claude_dir);
    let limits = collect_limits(&login.access_token, login.expires_at_milliseconds, options.force);

    AgentScan {
        record: AgentRecord {
            id: AGENT_ID.to_string(),
            name: AGENT_NAME.to_string(),
            ready: stats.total_prompts > 0 || !limits.limits.is_empty(),
            tier_label: login.plan_label,
            usage_status_text: limits.usage_status_text,
            auth_help_text: limits.auth_help_text,
            limits: limits.limits,
            retry_advised: limits.retry_advised,
            stats,
        },
        sources: vec![
            SourceScan { id: "claudeCode", stats: transcripts },
            SourceScan { id: "pi", stats: pi },
            SourceScan { id: "opencode", stats: opencode },
        ],
    }
}

fn transcript_stats(claude_dir: &Path, scan_age: f64) -> Option<UsageStats> {
    let mut stats = cached_scan(&claude_dir.join("projects"), scan_age);

    if stats.total_prompts <= 0 {
        match stats_cache_fallback(claude_dir) {
            Some(cached) => stats = cached,
            None => {
                let (today_prompts, today_sessions) = today_prompts_from_history(claude_dir);
                stats.today_prompts = today_prompts;
                stats.today_sessions = today_sessions;
            }
        }
    }

    (stats.total_prompts > 0 || stats.today_prompts > 0).then_some(stats)
}
