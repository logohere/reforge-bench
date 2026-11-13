use serde_json::Value;

pub fn capture_id(id: &str) -> bool {
    ["scope-", "logic-"].iter().any(|prefix| {
        id.strip_prefix(prefix).is_some_and(|hash| {
            hash.len() == 64
                && hash
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
    })
}

fn allowed(path: &str, body: Option<&Value>) -> bool {
    match (path, body) {
        (
            "/api/scope/status" | "/api/scope/setup" | "/api/logic/status" | "/api/pico/status"
            | "/api/captures",
            None,
        ) => true,
        ("/api/scope/disconnect", Some(body)) => *body == serde_json::json!({"op":"disconnect"}),
        ("/api/scope/recording", Some(body)) => [
            serde_json::json!({"op":"start"}),
            serde_json::json!({"op":"stop"}),
        ]
        .contains(body),
        ("/api/logic/scan", Some(body)) => *body == serde_json::json!({"op":"scan"}),
        ("/api/logic/capture", Some(body)) => [
            serde_json::json!({"profile":"preview"}),
            serde_json::json!({"profile":"pulse-loop"}),
        ]
        .contains(body),
        ("/api/pico/control", Some(body)) => [
            serde_json::json!({"op":"stop"}),
            serde_json::json!({"op":"start","wiring_confirmed":true}),
        ]
        .contains(body),
        ("/api/captures", Some(body)) => {
            let Some(object) = body.as_object() else {
                return false;
            };
            match body["op"].as_str() {
                Some("save") => {
                    object.len() == 3
                        && matches!(body["source"].as_str(), Some("scope" | "logic"))
                        && body["key"]
                            .as_str()
                            .is_some_and(|key| !key.is_empty() && key.len() <= 128)
                }
                Some("load" | "trash" | "restore") => {
                    object.len() == 2 && body["id"].as_str().is_some_and(capture_id)
                }
                Some("trash_all" | "purge_trash") => object.len() == 2 && body["confirm"] == true,
                _ => false,
            }
        }
        _ => false,
    }
}

fn transfer(port: u16, path: &str, body: Option<&Value>, limit: usize) -> Result<Vec<u8>, String> {
    super::local_http::transfer(port, path, body, limit, if body.is_some() { 35 } else { 5 })
}

pub fn request(port: u16, path: &str, body: Option<&Value>) -> Result<Value, String> {
    if !allowed(path, body) {
        return Err("Unsupported instrument operation".into());
    }
    let result: Value = serde_json::from_slice(&transfer(port, path, body, 16 * 1_048_576)?)
        .map_err(|_| "Invalid instrument response")?;
    if result["ok"] != true {
        return Err(result["error"]
            .as_str()
            .unwrap_or("Instrument operation failed")
            .into());
    }
    Ok(result)
}

pub fn export(port: u16, id: Option<&str>, source: &str, csv: bool) -> Result<Vec<u8>, String> {
    let path = if let Some(id) = id {
        if !capture_id(id) || csv && !id.starts_with("scope-") {
            return Err("Invalid capture export".into());
        }
        format!(
            "/api/captures/file?id={id}&format={}",
            if csv { "csv" } else { "raw" }
        )
    } else {
        match (source, csv) {
            ("scope", true) => "/api/scope/export?format=csv".into(),
            ("scope", false) => "/api/scope/export".into(),
            ("logic", false) => "/api/logic/export".into(),
            _ => return Err("Unsupported capture export".into()),
        }
    };
    transfer(port, &path, None, if csv { 128 } else { 16 } * 1_048_576)
}

pub fn validate_frame(frame: &Value) -> Result<(), String> {
    let samples = frame["samples"]
        .as_u64()
        .filter(|count| *count > 0)
        .ok_or("Missing sample count")?;
    let hash = frame["sha256"].as_str().ok_or("Missing capture hash")?;
    if hash.len() != 64
        || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        || !matches!(
            frame["origin"].as_str(),
            Some("hardware" | "replay" | "usb-analyzer")
        )
    {
        return Err("Invalid capture identity".into());
    }
    let time = frame["time"]
        .as_array()
        .filter(|time| !time.is_empty() && time.len() <= 116002)
        .ok_or("Invalid plotted sample count")?;
    let mut previous = -1.0;
    for stamp in time {
        let stamp = stamp
            .as_f64()
            .filter(|stamp| stamp.is_finite() && *stamp >= previous && *stamp >= 0.0)
            .ok_or("Invalid capture timebase")?;
        previous = stamp;
    }
    let valid_series = |values: &Value, digital: bool| {
        values.as_array().is_some_and(|values| {
            values.len() == time.len()
                && values.iter().all(|value| {
                    value.as_f64().is_some_and(|value| {
                        value.is_finite()
                            && if digital {
                                value == 0.0 || value == 1.0
                            } else {
                                (-128.0..=127.0).contains(&value)
                            }
                    })
                })
        })
    };
    if frame["counts"].is_array() {
        if frame["origin"] == "usb-analyzer"
            || time.len() > 4096
            || !valid_series(&frame["counts"], false)
            || !frame["sample_rate_hz"]
                .as_f64()
                .is_some_and(|rate| rate.is_finite() && rate > 0.0)
            || samples > 2_097_152
        {
            return Err("Invalid CH1 samples or clock".into());
        }
        if let Some(indices) = frame["sample_indices"].as_array() {
            if indices.len() != time.len() {
                return Err("Invalid original sample indices".into());
            }
            let mut previous = None;
            for index in indices {
                let index = index
                    .as_u64()
                    .filter(|index| *index < samples && previous.is_none_or(|last| *index > last))
                    .ok_or("Invalid original sample index")?;
                previous = Some(index);
            }
        }
    } else if !frame["channels"].as_array().is_some_and(|channels| {
        channels.len() == 2 && channels.iter().all(|channel| valid_series(channel, true))
    }) {
        return Err("Invalid digital CH0/CH1 samples".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn instrument_commands_reject_arbitrary_paths_unconfirmed_output_and_capture_ids() {
        assert!(!allowed("/api/pico/control", Some(&json!({"op":"start"}))));
        assert!(allowed(
            "/api/pico/control",
            Some(&json!({"op":"start","wiring_confirmed":true}))
        ));
        assert!(!allowed(
            "/api/captures",
            Some(&json!({"op":"purge_trash","confirm":false}))
        ));
        assert!(!allowed(
            "/api/captures",
            Some(&json!({"op":"load","id":"../../private"}))
        ));
        assert!(!allowed("/api/oem/run", Some(&json!({}))));
        assert!(capture_id(&format!("scope-{}", "a".repeat(64))));
        assert!(!capture_id(&format!("logic-{}", "A".repeat(64))));
    }

    #[test]
    fn waveform_validation_rejects_mismatched_units_timebase_and_sample_identity() {
        let frame = json!({"samples":100,"sample_rate_hz":1000,"sha256":"a".repeat(64),"origin":"replay","time":[0,0.001,0.099],"counts":[-128,0,127],"sample_indices":[0,1,99]});
        assert!(validate_frame(&frame).is_ok());
        for (field, value) in [
            ("origin", json!("live")),
            ("counts", json!([0, 1])),
            ("time", json!([0, 0.099, 0.001])),
            ("sample_indices", json!([0, 1, 100])),
            ("sample_rate_hz", json!(0)),
        ] {
            let mut changed = frame.clone();
            changed[field] = value;
            assert!(validate_frame(&changed).is_err(), "{field}");
        }
        let digital = json!({"samples":10000000,"sha256":"b".repeat(64),"origin":"usb-analyzer","time":[0,0.1,10],"channels":[[0,1,0],[1,0,1]]});
        assert!(validate_frame(&digital).is_ok());
        let mut bad = digital;
        bad["channels"][0][0] = json!(3.3);
        assert!(validate_frame(&bad).is_err());
    }
}
