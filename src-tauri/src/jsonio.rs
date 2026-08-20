use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

pub fn number(value: Option<&Value>) -> i64 {
    let Some(value) = value else { return 0 };
    let numeric = match value {
        Value::Number(inner) => inner.as_f64().unwrap_or(0.0),
        Value::String(inner) => inner.trim().parse::<f64>().unwrap_or(0.0),
        Value::Bool(inner) => return i64::from(*inner),
        _ => 0.0,
    };
    if numeric.is_finite() {
        numeric.round() as i64
    } else {
        0
    }
}

pub fn usage_token(usage: &Value, primary_key: &str, alternate_key: &str) -> i64 {
    number(usage.get(primary_key).or_else(|| usage.get(alternate_key)))
}

pub fn object_or_empty<'a>(parent: Option<&'a Value>, key: &str, empty: &'a Value) -> &'a Value {
    parent
        .and_then(|value| value.get(key))
        .filter(|value| value.is_object())
        .unwrap_or(empty)
}

pub fn read_fresh_json<T: DeserializeOwned>(path: &Path, max_age_seconds: f64) -> Option<T> {
    if max_age_seconds <= 0.0 {
        return None;
    }
    let metadata = fs::metadata(path).ok()?;
    if max_age_seconds.is_finite() {
        let age = SystemTime::now()
            .duration_since(metadata.modified().ok()?)
            .ok()?
            .as_secs_f64();
        if age > max_age_seconds {
            return None;
        }
    }
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

pub fn write_json<T: Serialize>(path: &Path, payload: &T) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let prefix = format!("{}.", path.file_name().unwrap_or_default().to_string_lossy());
    let mut scratch = tempfile::Builder::new()
        .prefix(&prefix)
        .suffix(".tmp")
        .tempfile_in(parent)?;

    scratch.write_all(serde_json::to_string(payload)?.as_bytes())?;
    scratch.write_all(b"\n")?;
    scratch.flush()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(scratch.path(), fs::Permissions::from_mode(0o644));
    }

    scratch.persist(path).map_err(|failure| failure.error)?;
    Ok(())
}
