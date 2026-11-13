use serde_json::Value;

pub const DDCS_SHA: &str = "c70ff4eceadb98c811ccfe81efd0e62bd3d6c13e5ac18dfb8b590f75790f9b05";

pub fn value<'a>(item: &'a Value, key: &str, fallback: &'a str) -> &'a str {
    item[key]
        .as_str()
        .filter(|v| !v.is_empty())
        .unwrap_or(fallback)
}

pub fn valid_id(id: &str) -> bool {
    id.len() == 64
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Debug)]
pub struct Info<'a> {
    pub name: &'a str,
    pub short_name: Option<&'a str>,
    pub vendor: &'a str,
    pub purpose: &'a str,
    pub family: &'a str,
    pub version: &'a str,
    pub format: &'a str,
    pub component: &'a str,
    pub group: &'a str,
}

pub fn info(item: &Value) -> Info<'_> {
    let meta = &item["library"];
    if item["artifact_type"] == "dotdog" {
        return Info {
            name: value(meta, "model", "Evidence graph"),
            short_name: None,
            vendor: value(meta, "vendor", "User import"),
            purpose: "REA evidence graph",
            family: "Evidence maps",
            version: value(meta, "version", "Evidence graph"),
            format: "Dotdog 1.0 JSON",
            component: "Evidence graph",
            group: value(item, "case_id", ""),
        };
    }
    let profile = value(item, "profile_id", "");
    let fanuc = profile.starts_with("fanuc-crx-v940p90-");
    let haas = profile.starts_with("haas-coldfire2-");
    let known = match profile {
        "ddcs-expert-2020" => Some((
            "DDCS Expert",
            "CNC controller application",
            "2020-09-28-00",
            "ARM Linux executable · ELF",
        )),
        "ddcs-v41-motiondev" => Some((
            "DDCS V4.1 motion driver",
            "Linux driver research",
            "Build dated 4 Apr 2025",
            "Linux kernel module · ELF",
        )),
        "ddcs-expert-m350-motiondev" => Some((
            "Expert / M350 motion driver",
            "Related-controller driver research",
            "Backup dated 31 Dec 2025",
            "Linux kernel module · ELF",
        )),
        "centroid-acorn-v482" => Some((
            "Acorn",
            "CNC controller firmware",
            "CNC12 v4.82",
            "TI GP ARM image",
        )),
        "dobot-magician-lite-2022" => Some((
            "Magician Lite",
            "Robot firmware update",
            "Archive dated 6 Jul 2022",
            "DfuSe update container",
        )),
        "roland-mdx40a-v120" => Some((
            "MDX-40A",
            "Desktop mill firmware update",
            "v1.20",
            "Roland update container",
        )),
        _ => None,
    };
    let short_name = match profile {
        "haas-coldfire2-lathe" => Some("Haas Classic Control · Lathe"),
        "haas-coldfire2-mill" => Some("Haas Classic Control · Mill"),
        "haas-coldfire2-loader" => Some("Haas ColdFire 2 · Loader"),
        _ => None,
    };
    Info {
        name: if fanuc {
            "CRX controller"
        } else {
            known
                .map(|v| v.0)
                .unwrap_or_else(|| value(meta, "model", "Unidentified firmware"))
        },
        short_name,
        vendor: value(meta, "vendor", "Unknown manufacturer"),
        purpose: if fanuc {
            "Robot controller update"
        } else if haas {
            match profile {
                "haas-coldfire2-lathe" => "Lathe controller firmware",
                "haas-coldfire2-mill" => "Mill controller firmware",
                "haas-coldfire2-loader" => "Controller startup loader",
                _ => value(meta, "artifact_role", "Firmware investigation"),
            }
        } else {
            known
                .map(|v| v.1)
                .unwrap_or_else(|| value(meta, "artifact_role", "Firmware investigation"))
        },
        family: if fanuc {
            "CRX"
        } else if haas {
            "Classic Control · ColdFire 2"
        } else if profile.starts_with("ddcs-")
            || value(meta, "model", "")
                .to_ascii_lowercase()
                .starts_with("ddcs ")
            || value(meta, "model", "").eq_ignore_ascii_case("ddcs")
        {
            "DDCS"
        } else {
            value(meta, "family_name", value(meta, "model", "Other firmware"))
        },
        version: if fanuc {
            "V9.40P/90"
        } else {
            known
                .map(|v| v.2)
                .unwrap_or_else(|| value(meta, "version", "Version not identified"))
        },
        format: if fanuc {
            "Packed FANUC load module"
        } else if haas {
            "ColdFire firmware · mapped analysis ELF"
        } else {
            known.map(|v| v.3).unwrap_or("Format not identified")
        },
        component: if fanuc {
            "Internal package component · role not yet verified"
        } else {
            known
                .map(|v| v.1)
                .unwrap_or_else(|| value(meta, "artifact_role", "Role not yet established"))
        },
        group: if fanuc {
            "fanuc-crx-v940p90"
        } else if haas {
            "haas-coldfire2"
        } else {
            value(item, "profile_id", value(item, "case_id", ""))
        },
    }
}

pub fn recovery<'a>(item: &Value, profiles: &'a Value) -> Option<&'a Value> {
    let record = &profiles[value(item, "profile_id", "")];
    (record.is_object() && record["sha256"] == item["case_id"] && item["case_id"].is_string())
        .then_some(record)
}

pub fn has_adapter(item: &Value) -> bool {
    matches!(
        item["emulation"].as_str(),
        Some("recorded" | "not_run" | "failed")
    )
}

pub fn progress<'a>(item: &Value, profiles: &Value) -> (i64, &'a str, &'a str) {
    let record = recovery(item, profiles);
    if let Some(record) = record
        && record["compared"] == true
    {
        return (
            300 + record["rank"].as_i64().unwrap_or(0),
            "Bounded comparison",
            "success",
        );
    }
    if item["last_runs"]["analyze"]["ok"] == true {
        return (200, "Inspected", "info");
    }
    if record
        .and_then(|v| v["milestones"].as_array())
        .is_some_and(|v| {
            v.iter()
                .any(|v| v[0] == "Analyze" && matches!(v[1].as_str(), Some("Partial" | "Analyzed")))
        })
    {
        return (200, "Partial analysis", "info");
    }
    if item["last_runs"]["analyze"]["ok"] == false {
        return (100, "Analysis failed", "danger");
    }
    (0, "Acquired", "secondary")
}

pub fn support(item: &Value, profiles: &Value) -> &'static str {
    let id = value(item, "case_id", "");
    let Some(record) = recovery(item, profiles) else {
        return "Investigation only";
    };
    if id == DDCS_SHA
        && item["profile_id"] == "ddcs-expert-2020"
        && record["lab_contract"]["firmware_sha256"] == id
    {
        "Research emulation"
    } else if [
        "7b33f8e7f5f290c34e9b21eb4027eab6ab4a5fbea7b725b9c7974fcd82c15171",
        "148fde6e22ee1e7ef6e16fb45c5bd70776feb53c20979a4b6e9289162461b9d7",
        "e9b9039e1241a54233033ffb5bfe135c7fc3d177d3793e8df3ff8a25f123a898",
    ]
    .contains(&id)
    {
        "Bounded Haas research"
    } else {
        "Investigation only"
    }
}

fn device_query(id: &str) -> String {
    const ROUTES: &str = include_str!("../../../../../web/system-routes.js");
    let systems = ROUTES
        .split_once("const systems={")
        .and_then(|(_, s)| s.split_once("\n};"))
        .expect("shared system route table must exist")
        .0;
    for entry in systems.lines().filter(|s| !s.trim().is_empty()) {
        let (slug, hash) = entry
            .trim()
            .trim_end_matches(',')
            .split_once(':')
            .expect("shared system route must be a slug/hash pair");
        let slug = slug.trim_matches('\'');
        let hash = hash.trim_matches('\'');
        assert!(
            valid_id(hash),
            "shared system route must identify exact firmware"
        );
        if hash == id {
            return format!("?system={slug}");
        }
    }
    format!("?case={id}")
}

pub fn route(item: &Value, profiles: &Value, destination: &str) -> Option<String> {
    let id = value(item, "case_id", "");
    if !valid_id(id) {
        return None;
    }
    let (path, hash) = match destination {
        "rea" => ("rea", "#target"),
        "procedure" => ("hardware", "#procedure"),
        "lab" | "timeline" | "hardware" | "help" | "home" | "library" => (destination, ""),
        _ => return None,
    };
    let driver = if path == "rea" && recovery(item, profiles).is_some() {
        match value(item, "profile_id", "") {
            "ddcs-expert-m350-motiondev" => "&driver=expert-m350#kernel",
            "ddcs-v41-motiondev" => "&driver=ddcs-v41#kernel",
            _ => "",
        }
    } else {
        ""
    };
    Some(format!(
        "/{path}/{}{}",
        device_query(id),
        if driver.is_empty() { hash } else { driver }
    ))
}

pub fn machine_type(value: &str) -> &'static str {
    match value {
        "cnc" => "CNC",
        "3d-printer" => "3D printer",
        "robot-arm" => "Robot arm",
        "instrument" => "Instrument",
        _ => "Unknown type",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn foreign_evidence_cannot_enable_research_or_driver_routes() {
        let item = json!({"case_id": DDCS_SHA,"profile_id":"ddcs-expert-2020","library":{}});
        let foreign = json!({"ddcs-expert-2020":{"sha256":"wrong","compared":true,"lab_contract":{"firmware_sha256":DDCS_SHA}}});
        assert_eq!(support(&item, &foreign), "Investigation only");
        assert_eq!(progress(&item, &foreign).0, 0);
        let mut correct = foreign;
        correct["ddcs-expert-2020"]["sha256"] = DDCS_SHA.into();
        assert_eq!(support(&item, &correct), "Research emulation");
        assert_eq!(progress(&item, &correct).0, 300);
    }

    #[test]
    fn package_identity_does_not_collapse_distinct_controller_drivers() {
        let a = json!({"profile_id":"fanuc-crx-v940p90-mpplan","library":{}});
        let b = json!({"profile_id":"fanuc-crx-v940p90-ioscan","library":{}});
        assert_eq!(info(&a).group, info(&b).group);
        let a = json!({"profile_id":"ddcs-v41-motiondev","library":{}});
        let b = json!({"profile_id":"ddcs-expert-m350-motiondev","library":{}});
        assert_ne!(info(&a).group, info(&b).group);
        assert_eq!(info(&a).format, "Linux kernel module · ELF");
        assert!(route(&a, &json!({}), "lab").is_none());
        let graph = json!({"case_id":"a".repeat(64),"artifact_type":"dotdog","library":{"model":"connections.json"}});
        assert_eq!(info(&graph).version, "Evidence graph");
        assert_eq!(info(&graph).format, "Dotdog 1.0 JSON");
        assert_eq!(info(&graph).vendor, "User import");
        assert_eq!(info(&json!({})).version, "Version not identified");
    }
    #[test]
    fn original_route_aliases_and_driver_anchors_are_preserved() {
        let item = json!({"case_id":DDCS_SHA,"profile_id":"ddcs-expert-2020"});
        assert_eq!(
            route(&item, &json!({}), "rea").unwrap(),
            "/rea/?system=ddcs-expert#target"
        );
        assert_eq!(
            route(&item, &json!({}), "procedure").unwrap(),
            "/hardware/?system=ddcs-expert#procedure"
        );
        let id = "6e309c6368aaacf4a2b406f645f8f631f43e2a6778c934ed2ed67f7d684a4eee";
        let item = json!({"case_id":id,"profile_id":"ddcs-v41-motiondev"});
        let profiles = json!({"ddcs-v41-motiondev":{"sha256":id}});
        assert_eq!(
            route(&item, &profiles, "rea").unwrap(),
            "/rea/?system=ddcs-v4-driver&driver=ddcs-v41#kernel"
        );
        let item = json!({"case_id":"a".repeat(64)});
        assert_eq!(
            route(&item, &json!({}), "lab").unwrap(),
            format!("/lab/?case={}", "a".repeat(64))
        );
    }
}
