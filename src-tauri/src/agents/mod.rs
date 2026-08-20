pub mod claude;
pub mod codex;

use crate::report::{AgentRecord, RefreshOptions, SourceReport, UsageReport, SCHEMA_VERSION, SOURCE_CATALOG};
use crate::settings::Settings;
use crate::stats::UsageStats;
use crate::timeline::{today_string, utc_now_rfc3339};

pub struct SourceScan {
    pub id: &'static str,
    pub stats: Option<UsageStats>,
}

pub struct AgentScan {
    pub record: AgentRecord,
    pub sources: Vec<SourceScan>,
}

pub fn build_report(options: RefreshOptions, settings: &Settings) -> UsageReport {
    let mut agents: Vec<AgentRecord> = Vec::new();
    let mut scans: Vec<SourceScan> = Vec::new();
    let today = today_string();

    if claude::is_enabled(settings) {
        let scan = claude::scan(options, settings);
        agents.push(with_history(scan.record, &today));
        scans.extend(scan.sources);
    }

    if codex::is_enabled(settings) {
        let scan = codex::scan(options, settings);
        agents.push(with_history(scan.record, &today));
        scans.extend(scan.sources);
    }

    UsageReport {
        schema_version: SCHEMA_VERSION,
        updated_at: utc_now_rfc3339(),
        agents,
        sources: source_reports(settings, &scans),
    }
}

fn with_history(mut record: AgentRecord, today: &str) -> AgentRecord {
    record.stats.history = crate::history::record_today(
        &record.id,
        today,
        record.stats.today_total_tokens,
        record.stats.today_prompts,
        &record.stats.recent_days,
    );
    record
}

fn source_reports(settings: &Settings, scans: &[SourceScan]) -> Vec<SourceReport> {
    SOURCE_CATALOG
        .iter()
        .map(|info| {
            let found: Vec<&UsageStats> = scans
                .iter()
                .filter(|scan| scan.id == info.id)
                .filter_map(|scan| scan.stats.as_ref())
                .collect();

            SourceReport {
                id: info.id.to_string(),
                label: info.label.to_string(),
                description: info.description.to_string(),
                supported: info.supported,
                enabled: settings.sources.enabled(info.id),
                found: !found.is_empty(),
                prompts: found.iter().map(|stats| stats.total_prompts).sum(),
                total_tokens: found.iter().map(|stats| stats.total_tokens()).sum(),
            }
        })
        .collect()
}

pub fn merge_all(parts: &[Option<UsageStats>]) -> UsageStats {
    let mut merged = UsageStats::default();
    for part in parts.iter().flatten() {
        merged = crate::stats::merge_stats(&merged, part);
    }
    merged
}
