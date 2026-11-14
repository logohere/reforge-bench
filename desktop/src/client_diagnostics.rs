use serde_json::{Value, json};

fn text(value: &Value, key: &str, limit: usize) -> String {
    value[key]
        .as_str()
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_control())
        .take(limit)
        .collect()
}

fn path(value: &Value, key: &str) -> String {
    let value = text(value, key, 240);
    let value = value.split(['?', '#']).next().unwrap_or_default();
    if value.starts_with('/') && !value.starts_with("//") && !value.contains('\\') {
        value.into()
    } else {
        String::new()
    }
}

pub fn parse(message: &str) -> Option<Value> {
    if message.len() > 16384 {
        return None;
    }
    let value: Value = serde_json::from_str(message).ok()?;
    if value["kind"] != "client-error"
        || !matches!(
            value["phase"].as_str(),
            Some("uncaught" | "rejection" | "resource")
        )
    {
        return None;
    }
    let action = text(&value, "action", 96);
    let action = if action
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "_.:-".contains(c))
    {
        action
    } else {
        String::new()
    };
    Some(json!({
        "origin": if cfg!(target_os = "windows") { "webview2" } else { "webkit" },
        "operation": value["phase"], "path": path(&value, "path"),
        "action": action, "detail": text(&value, "message", 320),
        "source": path(&value, "source"), "line": value["line"].as_u64().unwrap_or(0),
        "column": value["column"].as_u64().unwrap_or(0), "stack": text(&value, "stack", 1600),
        "session": text(&value, "session", 64), "sequence": value["sequence"].as_u64().unwrap_or(0),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_bounded_client_failures_without_query_or_payload() {
        let message = json!({"kind":"client-error","phase":"rejection","path":"/lab/?private=1#x",
            "source":"https://foreign.invalid/private", "message":"a\nb", "stack":"s".repeat(3000),
            "action":"material", "line":-1, "samples":[1,2,3]})
        .to_string();
        let event = parse(&message).unwrap();
        assert_eq!(event["path"], "/lab/");
        assert_eq!(event["source"], "");
        assert_eq!(event["detail"], "ab");
        assert_eq!(event["line"], 0);
        assert_eq!(event["stack"].as_str().unwrap().len(), 1600);
        assert!(event.get("samples").is_none());
        for message in [
            "{}",
            "[]",
            "null",
            "bad",
            &"x".repeat(16385),
            r#"{"kind":"client-error","phase":"arbitrary"}"#,
        ] {
            assert!(parse(message).is_none());
        }
    }
}
