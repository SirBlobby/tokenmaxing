use std::path::Path;

use serde_json::Value;

use crate::jsonio::number;

#[derive(Default)]
pub struct OauthLogin {
    pub access_token: String,
    pub expires_at_milliseconds: i64,
    pub plan_label: String,
}

pub fn oauth_login(claude_dir: &Path) -> OauthLogin {
    let raw = credentials_file(claude_dir).or_else(keychain_credentials);
    let Some(raw) = raw else { return OauthLogin::default() };
    let Ok(data) = serde_json::from_str::<Value>(&raw) else { return OauthLogin::default() };
    let Some(login) = data.get("claudeAiOauth").filter(|value| value.is_object()) else {
        return OauthLogin::default();
    };

    OauthLogin {
        access_token: login
            .get("accessToken")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        expires_at_milliseconds: number(login.get("expiresAt")),
        plan_label: plan_label(
            login.get("rateLimitTier").and_then(Value::as_str).unwrap_or_default(),
            login.get("subscriptionType").and_then(Value::as_str).unwrap_or_default(),
        ),
    }
}

fn credentials_file(claude_dir: &Path) -> Option<String> {
    std::fs::read_to_string(claude_dir.join(".credentials.json")).ok()
}

#[cfg(target_os = "macos")]
fn keychain_credentials() -> Option<String> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!text.is_empty()).then_some(text)
}

#[cfg(not(target_os = "macos"))]
fn keychain_credentials() -> Option<String> {
    None
}

fn plan_label(tier: &str, subscription: &str) -> String {
    if let Some(multiplier) = max_multiplier(tier) {
        return format!("Max {multiplier}");
    }
    if subscription.is_empty() {
        return String::new();
    }
    let mut characters = subscription.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => String::new(),
    }
}

fn max_multiplier(tier: &str) -> Option<String> {
    let lowered = tier.to_lowercase();
    let bytes = lowered.as_bytes();
    let mut start = 0;

    while let Some(found) = lowered[start..].find("max_") {
        let mut cursor = start + found + "max_".len();
        let digits_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if cursor > digits_start && cursor < bytes.len() && bytes[cursor] == b'x' {
            return Some(lowered[digits_start..=cursor].to_string());
        }
        start = start + found + 1;
    }
    None
}
