use chrono::DateTime;
use serde_json::{Value, json};

use super::catalog::{valid_id, value};

pub struct History {
    pub moments: Vec<Value>,
    pub snapshots: Value,
    pub lanes: Vec<Value>,
    pub timezone: chrono_tz::Tz,
}

pub fn timestamp(moment: &Value) -> Result<i64, String> {
    DateTime::parse_from_rfc3339(value(moment, "recorded_at", ""))
        .map(|date| date.timestamp_millis())
        .map_err(|_| "Checkpoint has an invalid recorded date.".into())
}

impl History {
    pub fn parse(
        data: Value,
        visuals: Value,
        history: Value,
        experiments: Value,
        case: &str,
        app: bool,
    ) -> Result<Self, String> {
        if !valid_id(case) {
            return Err("Choose an exact firmware image first.".into());
        }
        let timezone = value(&data, "timezone", "")
            .parse()
            .map_err(|_| "Timeline timezone is not recognized.")?;
        let mut moments: Vec<_> = data["moments"]
            .as_array()
            .ok_or("The saved timeline could not be loaded.")?
            .iter()
            .filter(|moment| {
                moment["history"] == if app { "app" } else { "rea" }
                    && (app
                        || moment["targets"]
                            .as_array()
                            .is_some_and(|targets| targets.iter().any(|target| target == case)))
            })
            .cloned()
            .collect();
        let mut snapshots = visuals["snapshots"].clone();
        if !snapshots.is_object() {
            snapshots = json!({});
        }
        if !app {
            append_firmware(&history, case, &mut moments, &mut snapshots)?;
            append_experiments(&experiments, &history, case, &mut moments, &mut snapshots)?;
        }
        let lanes = data[if app { "lanes" } else { "rea_lanes" }]
            .as_array()
            .ok_or("Timeline stages are unavailable.")?
            .clone();
        let mut ids = std::collections::HashSet::new();
        for moment in &moments {
            timestamp(moment)?;
            let id = value(moment, "id", "");
            if id.is_empty()
                || !ids.insert(id)
                || !lanes
                    .iter()
                    .any(|lane| lane["id"] == moment[if app { "lane" } else { "stage" }])
            {
                return Err("Timeline has ambiguous checkpoints or stages.".into());
            }
        }
        moments.sort_by(|a, b| {
            timestamp(a)
                .unwrap()
                .cmp(&timestamp(b).unwrap())
                .then_with(|| value(a, "id", "").cmp(value(b, "id", "")))
        });
        Ok(Self {
            moments,
            snapshots,
            lanes,
            timezone,
        })
    }

    pub fn snapshot(&self, moment: &Value) -> Value {
        let snapshot = &self.snapshots[value(moment, "id", "")];
        let matched = ["commit", "checkpoint"]
            .iter()
            .any(|key| moment[key].is_string() && snapshot[key] == moment[key]);
        if !matched || !snapshot["metrics"].is_array() || !snapshot["images"].is_array() {
            return Value::Null;
        }
        let mut result = snapshot.clone();
        let excerpt = &snapshot["excerpt"];
        if !excerpt.is_null() {
            let bound = moment["sources"]
                .as_array()
                .into_iter()
                .flatten()
                .chain(snapshot["sources"].as_array().into_iter().flatten())
                .any(|source| {
                    source["path"] == excerpt["path"] && source["sha256"] == excerpt["sha256"]
                });
            if !bound
                || value(excerpt, "text", "").is_empty()
                || value(excerpt, "text", "").encode_utf16().count() > 16000
                || !excerpt["title"].is_string()
                || !matches!(
                    excerpt["language"].as_str(),
                    Some("c" | "rust" | "verilog" | "json" | "text")
                )
            {
                result["excerpt"] = Value::Null;
            }
        }
        result
    }

    pub fn times(&self) -> Vec<i64> {
        self.moments
            .iter()
            .map(|moment| timestamp(moment).unwrap())
            .collect()
    }
}

fn records<'a>(history: &'a Value, case: &str, key: &str) -> Result<&'a Vec<Value>, String> {
    if history["ok"] != true || history["firmware_sha256"] != case {
        return Err("Checkpoint history does not match the selected firmware.".into());
    }
    history[key]
        .as_array()
        .ok_or_else(|| "Saved firmware records could not be loaded.".into())
}

fn recorded_at(record: &Value) -> Result<String, String> {
    let ms = record["started_at_ms"]
        .as_i64()
        .filter(|ms| ms.unsigned_abs() <= 9_007_199_254_740_991)
        .ok_or("Invalid firmware checkpoint identity or date.")?;
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|date| date.to_rfc3339_opts(chrono::SecondsFormat::Millis, false))
        .ok_or_else(|| "Invalid firmware checkpoint date.".into())
}

fn metric(metrics: &mut Vec<Value>, label: &str, number: Value) {
    metrics.push(json!({"label":label,"value":number}));
}

pub fn append_firmware(
    history: &Value,
    case: &str,
    moments: &mut Vec<Value>,
    snapshots: &mut Value,
) -> Result<(), String> {
    for saved in records(history, case, "checkpoints")? {
        let record = &saved["record"];
        let result = &record["result"];
        if record["firmware_sha256"] != case || !result.is_object() {
            return Err("Invalid firmware checkpoint identity or date.".into());
        }
        let date = recorded_at(record)?;
        if result["subject"]["digest"]["sha256"].is_string()
            && result["subject"]["digest"]["sha256"] != record["analysis_image_sha256"]
        {
            return Err("Checkpoint analysis image differs from its saved result.".into());
        }
        if result["firmware_sha256"].is_string() && result["firmware_sha256"] != case {
            return Err("Checkpoint result belongs to another firmware.".into());
        }
        let findings = &result["normalized_result"];
        let mut metrics = Vec::new();
        let mut excerpt = Value::Null;
        let mut stage = "recover";
        let mut kind = "Inspected";
        let mut step = "target";
        let (title, summary, limit): (String, String, String);
        match value(result, "operation", "") {
            "binary_overview" => {
                kind = "Analyzed";
                title = "Analyze the selected executable".into();
                summary = format!(
                    "{} found {} functions and {} strings in the mapped image.",
                    value(&result["provider"], "name", "Analysis provider"),
                    findings["procedure_count"],
                    findings["string_count"]
                );
                limit = "Static discovery and mapped code inspection; not a source rebuild, controller boot or physical operation.".into();
                for (label, key) in [
                    ("Discovered functions", "procedure_count"),
                    ("Mapped regions", "segment_count"),
                    ("Defined strings", "string_count"),
                ] {
                    metric(&mut metrics, label, findings[key].clone());
                }
                excerpt = json!({"title":"Analysis result","language":"json","text":serde_json::to_string_pretty(&json!({"procedure_count":findings["procedure_count"],"segment_count":findings["segment_count"],"string_count":findings["string_count"]})).unwrap()});
            }
            "procedure_pseudo_code" => {
                let procedure = value(&result["parameters"], "procedure", "Unknown");
                title = format!("Inspect function {procedure}");
                summary =
                    "Decompile a discovered function in the exact selected analysis image.".into();
                limit = "Decompiler pseudocode is not original source, compiled reconstruction or execution evidence.".into();
                let code = findings.as_str().ok_or("Invalid pseudocode checkpoint.")?;
                metric(&mut metrics, "Pseudocode bytes", code.len().into());
                excerpt = json!({"title":format!("Decompiler pseudocode · {procedure}"),"language":"text","text":code.split('\n').take(12).collect::<Vec<_>>().join("\n").chars().take(4000).collect::<String>()});
            }
            "decompile_all" => {
                kind = if record["ok"] == true {
                    "Exported"
                } else {
                    "Partial export"
                };
                title = "Decompile firmware functions".into();
                let counts = &findings["counts"];
                summary = format!(
                    "{} new files · {} retained entries · {} failures · {} remaining.",
                    counts["written"], counts["skipped"], counts["failed"], counts["remaining"]
                );
                limit = "Pseudocode for discovered functions only. External functions and decompiler failures remain explicit; not a complete source rebuild or controller boot.".into();
                for (label, key) in [
                    ("Selected functions", "selected"),
                    ("New files", "written"),
                    ("Failed functions", "failed"),
                    ("Remaining", "remaining"),
                ] {
                    metric(&mut metrics, label, counts[key].clone());
                }
            }
            "file_triage" => {
                stage = "target";
                title = "Inspect original file bytes".into();
                summary = format!(
                    "Retain the file header and embedded text from {} bytes.",
                    findings["file_bytes"]
                );
                limit = "Byte triage only; executable decoding, architecture and behavior remain unverified.".into();
            }
            _ => match value(record, "operation", "") {
                "resources" => {
                    kind = "Decoded";
                    title = "Decode original robot resources".into();
                    let streams = result["parameter_streams"].as_array().map_or(0, Vec::len);
                    let models = result["model_resources"].as_array().map_or(0, Vec::len);
                    summary = format!(
                        "{streams} parameter streams checked against the original PowerPC loader · {models} robot resource variants."
                    );
                    limit = value(result, "scope", "").into();
                    metric(&mut metrics, "Checked parameter streams", streams.into());
                    metric(&mut metrics, "Robot resource variants", models.into());
                }
                "investigate" => {
                    kind = if record["ok"] == true {
                        "Analyzed"
                    } else {
                        "Failure retained"
                    };
                    title = "Recover runtime dependencies".into();
                    summary = if let Some(modules) = result["modules"].as_array() {
                        metric(&mut metrics, "Original modules", modules.len().into());
                        metric(
                            &mut metrics,
                            "Dependency-bound function bytes",
                            result["dependency_bound_body_bytes"].clone(),
                        );
                        format!(
                            "{} original modules · {} mapped regions · {} repeated function checks.",
                            modules.len(),
                            result["mapped_regions"],
                            result["functions"].as_array().map_or(0, Vec::len)
                        )
                    } else {
                        "Investigation failed; retained attempt and telemetry remain inspectable."
                            .into()
                    };
                    limit = value(result, "scope", "").into();
                }
                "test" => {
                    stage = "compare";
                    step = "compare";
                    kind = if record["ok"] == true {
                        "Compared"
                    } else {
                        "Failure retained"
                    };
                    title = "Compare original and recovered code".into();
                    summary = value(
                        result,
                        "scope",
                        "Retained target-specific comparison result.",
                    )
                    .into();
                    limit =
                        value(result, "scope", "Only the recorded experiment is covered.").into();
                    for (key, label) in [
                        ("comparison_cases_passed", "Matching cases"),
                        ("case_count", "Bounded cases"),
                        ("cases", "Bounded cases"),
                    ] {
                        if result[key].is_number() {
                            metric(&mut metrics, label, result[key].clone());
                        }
                    }
                    if let Some(controls) = result["negative_controls"].as_array() {
                        metric(
                            &mut metrics,
                            "Rejected wrong builds",
                            controls
                                .iter()
                                .filter(|item| item["rejected"] == true)
                                .count()
                                .into(),
                        );
                    }
                }
                "startup-emulation" => {
                    stage = "execute";
                    step = "boot";
                    let modeled = result["console_model"] == true;
                    kind = if modeled {
                        "Modeled startup"
                    } else {
                        "Dependency stop"
                    };
                    title = if result["backend"].is_string() {
                        "Execute selected application"
                    } else {
                        "Rehost selected startup"
                    }
                    .into();
                    let backend = if result["backend"].is_string() {
                        format!("Provider · {}", value(result, "engine", "Unknown"))
                    } else if modeled {
                        "Inferred console services".into()
                    } else {
                        "Original monitor unresolved".into()
                    };
                    let events = result["events"].as_array().cloned().unwrap_or_default();
                    let missing = events
                        .iter()
                        .rev()
                        .find(|event| event["event"] == "unmapped_access")
                        .map(|event| format!(" · missing {}", value(event, "address", "Unknown")))
                        .unwrap_or_default();
                    summary = format!(
                        "{backend} · stopped at {}{missing}.",
                        value(result, "pc", "Unknown")
                    );
                    let saves = events
                        .iter()
                        .filter(|event| event["event"] == "original_fsave_after")
                        .count();
                    if saves > 0 {
                        metric(&mut metrics, "Original FPU frame saves", saves.into());
                    }
                    limit = value(result, "scope", "").into();
                }
                "startup-probe" => {
                    stage = "execute";
                    step = "boot";
                    kind = "Dependency stop";
                    title = "Probe original Haas startup".into();
                    summary = format!(
                        "{} original instructions · stopped at {}.",
                        result["instruction_count"],
                        value(result, "pc", "Unknown")
                    );
                    limit = value(result, "scope", "").into();
                    metric(
                        &mut metrics,
                        "Instructions observed",
                        result["instruction_count"].clone(),
                    );
                }
                "emulate" => {
                    stage = "execute";
                    step = "boot";
                    kind = if record["ok"] == true {
                        "Executed"
                    } else {
                        "Failure retained"
                    };
                    title = "Run the target-specific controller check".into();
                    summary = value(
                        result,
                        "scope",
                        "Retained execution result for this firmware.",
                    )
                    .into();
                    limit = value(
                        result,
                        "scope",
                        "This check does not establish a complete original hardware model.",
                    )
                    .into();
                }
                _ => continue,
            },
        }
        let id = format!("run-{}", value(saved, "id", ""));
        moments.push(json!({"id":id,"recorded_at":date,"history":"rea","targets":[case],"stage":stage,"lane":stage,"kind":kind,"title":title,"summary":summary,"limit":limit,"step":step,"checkpoint":saved["id"],"sources":[saved["source"]]}));
        for metric in &mut metrics {
            metric["source"] = saved["source"]["path"].clone();
        }
        let mut snapshot =
            json!({"checkpoint":saved["id"],"images":[],"metrics":metrics,"record":record});
        if !excerpt.is_null() {
            excerpt["path"] = saved["source"]["path"].clone();
            excerpt["sha256"] = saved["source"]["sha256"].clone();
            snapshot["excerpt"] = excerpt;
        }
        snapshots[id] = snapshot;
    }
    Ok(())
}

fn append_experiments(
    experiments: &Value,
    history: &Value,
    case: &str,
    moments: &mut Vec<Value>,
    snapshots: &mut Value,
) -> Result<(), String> {
    let checkpoints = records(history, case, "checkpoints")?;
    for group in records(experiments, case, "groups")? {
        if group["firmware_sha256"] != case
            || !group["source"]["path"].is_string()
            || !group["manifest_sha256"].is_string()
            || group["source"]["sha256"] != group["manifest_sha256"]
        {
            return Err("Invalid experiment identity, date or receipt.".into());
        }
        let date = recorded_at(group)?;
        if checkpoints
            .iter()
            .any(|saved| saved["record"]["result"]["experiment"]["id"] == group["id"])
        {
            continue;
        }
        let haas = group["pipeline"] == "haas-coldfire-rea-v1";
        let kind = if group["sealed"] != true {
            "In progress / unsealed"
        } else if group["status"] == "passed" {
            if haas { "Exported" } else { "Repeated" }
        } else if group["status"] == "partial" {
            "Partial export"
        } else {
            "Failure retained"
        };
        let id = format!("experiment-{}", value(group, "id", ""));
        moments.push(json!({"id":id,"recorded_at":date,"history":"rea","targets":[case],"stage":"recover","lane":"recover","kind":kind,"title":if haas {"Decompile Haas firmware"} else {"Repeat isolated dependency recovery"},"summary":format!("{} / {} runs finished · {}.",group["completed_runs"],group["planned_runs"],value(group,"status","Unknown")),"limit":group["isolation"],"step":"target","checkpoint":group["id"],"sources":[group["source"]]}));
        let mut metrics = Vec::new();
        for (label, number) in [
            ("Runs finished", group["completed_runs"].clone()),
            ("Runs passed", group["passed_runs"].clone()),
            (
                "Normalized matches",
                group["benchmark"]["semantic_matches"]
                    .as_u64()
                    .unwrap_or(0)
                    .into(),
            ),
        ] {
            metrics.push(json!({"label":label,"value":number,"source":group["source"]["path"]}));
        }
        snapshots[id] = json!({"checkpoint":group["id"],"images":[],"metrics":metrics,"record":group,"experiment":group});
    }
    Ok(())
}

pub struct Marker {
    pub indices: Vec<usize>,
    pub x: f32,
    pub width: f32,
}

pub fn bounds(times: &[i64]) -> (f64, f64) {
    let first = *times.first().unwrap_or(&0) as f64;
    let last = *times.last().unwrap_or(&0) as f64;
    let padding = ((last - first) * 0.05).ceil().max(1_800_000.0);
    (first - padding, last + padding)
}

pub fn ratio(time: f64, start: f64, end: f64) -> f32 {
    (0.04 + 0.92 * (time - start) / (end - start)) as f32
}

pub fn markers(indices: &[usize], times: &[i64], start: f64, end: f64, width: f32) -> Vec<Marker> {
    let width = width.max(1.0);
    let mut groups: Vec<Marker> = Vec::new();
    for &index in indices {
        let size = width.min(44.0);
        let x =
            (width * ratio(times[index] as f64, start, end)).clamp(size / 2.0, width - size / 2.0);
        groups.push(Marker {
            indices: vec![index],
            x,
            width: size,
        });
        while groups.len() > 1 {
            let b = &groups[groups.len() - 1];
            let a = &groups[groups.len() - 2];
            if a.x + a.width / 2.0 + 12.0 <= b.x - b.width / 2.0 {
                break;
            }
            let b = groups.pop().unwrap();
            let a = groups.last_mut().unwrap();
            a.indices.extend(b.indices);
            a.x = b.x;
        }
    }
    groups
}
