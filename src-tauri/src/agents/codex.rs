use crate::limits::codex_rpc::fetch_codex_account;
use crate::report::{AgentRecord, RefreshOptions};
use crate::settings::Settings;
use crate::sources::codex::scan_codex_usage;
use crate::sources::opencode::scan_opencode_usage;
use crate::sources::pi::{scan_pi_usage, PiProvider};

use super::{merge_all, AgentScan, SourceScan};

pub const AGENT_ID: &str = "codex";
pub const AGENT_NAME: &str = "Codex";

const OPENCODE_PROVIDER: &str = "openai";
const OPENCODE_DEFAULT_MODEL: &str = "codex";

pub fn is_enabled(settings: &Settings) -> bool {
    settings.sources.codex
}

pub fn scan(options: RefreshOptions, settings: &Settings) -> AgentScan {
    let scan_age = options.scan_age_seconds();

    let native = scan_codex_usage(scan_age);
    let pi = settings
        .sources
        .pi
        .then(|| scan_pi_usage(PiProvider::OpenAiCodex, scan_age))
        .flatten();
    let opencode = settings
        .sources
        .opencode
        .then(|| scan_opencode_usage(OPENCODE_PROVIDER, OPENCODE_DEFAULT_MODEL, scan_age))
        .flatten();

    let stats = merge_all(&[native.clone(), pi.clone(), opencode.clone()]);
    let account = fetch_codex_account();

    AgentScan {
        record: AgentRecord {
            id: AGENT_ID.to_string(),
            name: AGENT_NAME.to_string(),
            ready: stats.total_prompts > 0 || !account.limits.is_empty(),
            tier_label: account.tier_label,
            usage_status_text: account.usage_status_text,
            auth_help_text: account.auth_help_text,
            limits: account.limits,
            retry_advised: false,
            stats,
        },
        sources: vec![
            SourceScan { id: "codex", stats: native },
            SourceScan { id: "pi", stats: pi },
            SourceScan { id: "opencode", stats: opencode },
        ],
    }
}
