use std::fs;
use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};

pub fn config_dir() -> PathBuf {
    let configured = std::env::var("CLAUDE_CONFIG_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "~/.claude".to_string());
    expand_path(&configured)
}

pub fn expand_path(value: &str) -> PathBuf {
    let expanded = match value.strip_prefix('~') {
        Some(rest) => match dirs::home_dir() {
            Some(home) => home.join(rest.trim_start_matches('/')),
            None => PathBuf::from(value),
        },
        None => PathBuf::from(value),
    };
    fs::canonicalize(&expanded).unwrap_or(expanded)
}

pub fn cache_root() -> PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".cache"));
    let root = base.join("tokenmaxing");
    let _ = fs::create_dir_all(&root);
    root
}

pub fn opencode_database() -> Option<PathBuf> {
    let base = std::env::var("XDG_DATA_HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".local").join("share")))?;
    let database = base.join("opencode").join("opencode.db");
    database.is_file().then_some(database)
}

pub fn pi_session_roots() -> Vec<PathBuf> {
    match dirs::home_dir() {
        Some(home) => vec![
            home.join(".pi").join("agent").join("sessions"),
            home.join(".omp").join("agent").join("sessions"),
        ],
        None => Vec::new(),
    }
}

pub fn codex_home() -> PathBuf {
    std::env::var("CODEX_HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".codex"))
}

pub fn codex_session_roots() -> Vec<PathBuf> {
    let home = codex_home();
    vec![home.join("sessions"), home.join("archived_sessions")]
}

pub fn jsonl_files(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if !root.is_dir() {
        return found;
    }

    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(&directory) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => pending.push(path),
                Ok(_) if path.extension().is_some_and(|ext| ext == "jsonl") => found.push(path),
                _ => {}
            }
        }
    }
    found.sort();
    found
}

pub fn modified_within(path: &Path, max_age_seconds: u64) -> bool {
    let Ok(metadata) = fs::metadata(path) else { return false };
    let Ok(modified) = metadata.modified() else { return false };
    match std::time::SystemTime::now().duration_since(modified) {
        Ok(age) => age.as_secs() <= max_age_seconds,
        Err(_) => true,
    }
}

pub fn modified_milliseconds(path: &Path) -> Option<i64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    let elapsed = modified.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(elapsed.as_millis() as i64)
}

pub fn search_path_directories() -> Vec<PathBuf> {
    let mut directories: Vec<PathBuf> = std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .filter(|part| !part.is_empty())
        .map(PathBuf::from)
        .collect();

    if let Some(home) = dirs::home_dir() {
        directories.push(home.join(".local").join("bin"));
        directories.push(home.join(".npm-global").join("bin"));
        directories.push(home.join(".local").join("share").join("mise").join("shims"));
    }
    directories
}

pub fn find_executable(name: &str) -> Option<PathBuf> {
    search_path_directories()
        .into_iter()
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
}

pub fn short_digest(value: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())[..16].to_string()
}
