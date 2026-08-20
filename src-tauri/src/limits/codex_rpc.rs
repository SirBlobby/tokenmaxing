use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::jsonio::number;
use crate::limits::probe::RateLimit;
use crate::paths::find_executable;

pub const AUTH_HELP: &str = "Run `codex login` to authenticate.";

const INITIALIZE_TIMEOUT: Duration = Duration::from_secs(8);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Default)]
pub struct CodexAccount {
    pub tier_label: String,
    pub limits: Vec<RateLimit>,
    pub usage_status_text: String,
    pub auth_help_text: String,
}

pub fn fetch_codex_account() -> CodexAccount {
    let Some(executable) = find_executable("codex") else {
        return CodexAccount {
            usage_status_text: "Codex unavailable".to_string(),
            auth_help_text: "codex not found in PATH".to_string(),
            ..Default::default()
        };
    };

    let mut session = match RpcSession::start(&executable) {
        Ok(session) => session,
        Err(failure) => {
            return CodexAccount {
                usage_status_text: "Codex unavailable".to_string(),
                auth_help_text: failure,
                ..Default::default()
            }
        }
    };

    match read_account(&mut session) {
        Ok(account) => account,
        Err(failure) => CodexAccount {
            usage_status_text: "Codex limits unavailable".to_string(),
            auth_help_text: failure,
            ..Default::default()
        },
    }
}

fn read_account(session: &mut RpcSession) -> Result<CodexAccount, String> {
    let client = json!({ "clientInfo": { "name": "tokenmaxing", "version": "1" } });
    session.request(1, "initialize", client, INITIALIZE_TIMEOUT)?;
    session.notify("initialized")?;

    let account = session.request(2, "account/read", json!({}), REQUEST_TIMEOUT)?;
    let rate_limits = session.request(3, "account/rateLimits/read", json!({}), REQUEST_TIMEOUT)?;

    let account = account.get("result").and_then(|result| result.get("account")).cloned();
    let rate_limits = rate_limits
        .get("result")
        .and_then(|result| result.get("rateLimits"))
        .cloned()
        .unwrap_or(Value::Null);

    let mut limits = Vec::new();
    for key in ["primary", "secondary"] {
        if let Some(limit) = limit_window(rate_limits.get(key)) {
            limits.push(limit);
        }
    }

    Ok(CodexAccount {
        tier_label: plan_label(&rate_limits, account.as_ref()),
        limits,
        usage_status_text: String::new(),
        auth_help_text: AUTH_HELP.to_string(),
    })
}

fn plan_label(rate_limits: &Value, account: Option<&Value>) -> String {
    let from_account = |key: &str| account.and_then(|value| value.get(key)).and_then(Value::as_str);
    rate_limits
        .get("planType")
        .and_then(Value::as_str)
        .or_else(|| from_account("planType"))
        .or_else(|| from_account("type"))
        .unwrap_or_default()
        .to_string()
}

fn limit_window(window: Option<&Value>) -> Option<RateLimit> {
    let window = window.filter(|value| value.is_object())?;
    let used = window.get("usedPercent").filter(|value| !value.is_null())?;
    let percent = match used {
        Value::Number(value) => value.as_f64()?,
        Value::String(value) => value.trim().parse::<f64>().ok()?,
        _ => return None,
    };

    let minutes = number(window.get("windowDurationMins"));
    let label = if minutes == 10080 {
        "Weekly (7-day)".to_string()
    } else if minutes > 0 && minutes % 60 == 0 {
        format!("{}h window", minutes / 60)
    } else if minutes > 0 {
        format!("{minutes}m window")
    } else {
        "Limit".to_string()
    };

    let resets_at = match number(window.get("resetsAt")) {
        seconds if seconds > 0 => chrono::DateTime::from_timestamp(seconds, 0)
            .map(|moment| moment.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, false))
            .unwrap_or_default(),
        _ => String::new(),
    };

    Some(RateLimit {
        label,
        title: String::new(),
        percent: (percent / 100.0).clamp(0.0, 1.0),
        resets_at,
    })
}

struct RpcSession {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<String>,
}

impl RpcSession {
    fn start(executable: &std::path::Path) -> Result<Self, String> {
        let mut child = Command::new(executable)
            .args(["-s", "read-only", "-a", "untrusted", "app-server"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|failure| failure.to_string())?;

        let stdin = child.stdin.take().ok_or("codex stdin unavailable")?;
        let stdout = child.stdout.take().ok_or("codex stdout unavailable")?;

        let (sender, lines) = channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });

        Ok(Self { child, stdin, lines })
    }

    fn notify(&mut self, method: &str) -> Result<(), String> {
        self.send(&json!({ "method": method, "params": {} }))
    }

    fn request(&mut self, id: u32, method: &str, params: Value, timeout: Duration) -> Result<Value, String> {
        self.send(&json!({ "id": id, "method": method, "params": params }))?;

        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(format!("{method} timed out"));
            }
            match self.lines.recv_timeout(remaining) {
                Ok(line) => {
                    let Ok(message) = serde_json::from_str::<Value>(&line) else { continue };
                    if message.get("id").and_then(Value::as_u64) == Some(id as u64) {
                        return Ok(message);
                    }
                }
                Err(RecvTimeoutError::Timeout) => return Err(format!("{method} timed out")),
                Err(RecvTimeoutError::Disconnected) => return Err(format!("{method} closed the connection")),
            }
        }
    }

    fn send(&mut self, payload: &Value) -> Result<(), String> {
        writeln!(self.stdin, "{payload}").map_err(|failure| failure.to_string())?;
        self.stdin.flush().map_err(|failure| failure.to_string())
    }
}

impl Drop for RpcSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
