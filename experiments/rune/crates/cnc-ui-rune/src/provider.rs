use std::{
    io::{Read, Write},
    path::{Component, Path},
};

use serde_json::{Value, json};

const EXAMPLE: &[u8] =
    include_bytes!("../../../../../crates/bench-cli/examples/hardware-rea/summary.json");

pub(crate) fn example() -> Value {
    json!({"case_id":crate::sha256(EXAMPLE),
        "profile_id":"hardware-rea-example", "filename":"summary.json", "size":EXAMPLE.len(),
        "library":{"vendor":"Hantek", "model":"DSO2D15", "version":"Recorded example",
            "artifact_role":"Hardware investigation summary", "machine_type":"instrument"},
        "emulation":"unavailable", "last_runs":{}})
}

fn library(directory: &Path) -> Result<Value, String> {
    let mut cases = vec![example()];
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(json!({"ok":true,"cases":cases}));
        }
        Err(_) => return Err("Cannot read the local library".into()),
    };
    for entry in entries {
        let entry = entry.map_err(|_| "Cannot read a library entry")?;
        let path = entry.path();
        let Some(id) = path
            .file_stem()
            .and_then(|name| name.to_str())
            .filter(|id| super::catalog::valid_id(id))
        else {
            continue;
        };
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let mut bytes = Vec::new();
        std::fs::File::open(&path)
            .map_err(|_| "Cannot read library metadata")?
            .take(16_385)
            .read_to_end(&mut bytes)
            .map_err(|_| "Cannot read library metadata")?;
        if bytes.len() > 16_384 {
            return Err("Library metadata exceeds 16 KiB".into());
        }
        let case: Value = serde_json::from_slice(&bytes).map_err(|_| "Invalid library metadata")?;
        if case["case_id"] != id || !case["library"].is_object() {
            return Err("Library identity does not match its metadata".into());
        }
        if case["case_id"] != cases[0]["case_id"] {
            cases.push(case);
        }
    }
    cases[1..].sort_by(|a, b| a["case_id"].as_str().cmp(&b["case_id"].as_str()));
    Ok(json!({"ok":true,"cases":cases}))
}

pub fn import(path: &Path, bytes: &[u8], labels: &[(String, String)]) -> Result<String, String> {
    let root =
        super::backend_session::project_root().map_err(|_| "Application data unavailable")?;
    import_into(&root.join("library"), path, bytes, labels)
}

fn import_into(
    directory: &Path,
    path: &Path,
    bytes: &[u8],
    labels: &[(String, String)],
) -> Result<String, String> {
    if bytes.is_empty() || bytes.len() > 64 * 1024 * 1024 {
        return Err("Choose a nonempty artifact file of at most 64 MiB.".into());
    }
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Invalid artifact filename")?;
    let id = crate::sha256(bytes);
    let mut case = json!({"case_id":id,"filename":filename,"size":bytes.len(),"profile_id":id,
        "library":{"artifact_role":"Imported artifact", "model":filename}, "emulation":"unavailable", "last_runs":{}});
    if bytes.len() <= 16 * 1_048_576
        && let Ok(data) = serde_json::from_slice::<Value>(bytes)
        && data["nodes"].is_array()
        && (data["version"] == "1.0" || (data["project"].is_string() && data["edges"].is_array()))
    {
        super::dotdog::parse(bytes, &id)?;
        case["artifact_type"] = json!("dotdog");
        case["library"]["artifact_role"] = json!("Dotdog evidence graph");
    }
    for (key, value) in labels {
        if !["vendor", "model", "version", "machine_type", "parent"].contains(&key.as_str())
            || value.chars().count() > 512
        {
            return Err("Invalid artifact labels".into());
        }
        case["library"][key] = json!(value);
    }
    std::fs::create_dir_all(directory).map_err(|_| "Cannot create the local library")?;
    // Exclusive creation never replaces an existing content-addressed artifact.
    let artifact = directory.join(format!("{id}.artifact"));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    match options.open(&artifact) {
        Ok(mut file) => {
            if file.write_all(bytes).and_then(|_| file.sync_all()).is_err() {
                let _ = std::fs::remove_file(&artifact);
                return Err("Cannot retain imported artifact".into());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if artifact
                .metadata()
                .map_err(|_| "Cannot verify retained artifact")?
                .len()
                != bytes.len() as u64
                || std::fs::read(&artifact).map_err(|_| "Cannot verify retained artifact")? != bytes
            {
                return Err("Retained artifact identity is damaged".into());
            }
        }
        Err(_) => return Err("Cannot retain imported artifact".into()),
    }
    let metadata = directory.join(format!("{id}.json"));
    super::preferences::write(
        &metadata,
        &serde_json::to_vec(&case).map_err(|_| "Invalid artifact metadata")?,
    )
    .map_err(|_| "Cannot save artifact metadata")?;
    Ok(id)
}

pub fn request(path: &str, body: Option<&Value>) -> Result<Vec<u8>, String> {
    if body.is_some() {
        return Err(
            "No acquisition adapter is configured. Imported data and bench views remain available."
                .into(),
        );
    }
    let root =
        super::backend_session::project_root().map_err(|_| "Application data unavailable")?;
    let value = match path.split('?').next().unwrap_or(path) {
        "/api/desktop/health" => {
            json!({"ok":true,"version":env!("CARGO_PKG_VERSION"),"frontend":serde_json::from_slice::<Value>(&asset("/build.json")?).map_err(|_|"Invalid frontend build")?["id"],"transport":"in_process"})
        }
        "/version.json" => json!({"version":env!("CARGO_PKG_VERSION")}),
        "/desktop-build.json" => super::backend_session::build_identity(),
        "/api/scope/setup" => {
            json!({"ok":true,"configured":false,"message":"Configure your own acquisition adapter. No remote account, key or connection is bundled."})
        }
        "/api/library" => library(&root.join("library"))?,
        "/library/recovery-status.json" => json!({"profiles":{}}),
        "/api/hardware/status" => {
            json!({"ok":true,"pico":{"connected":false,"enabled":false},"scope":{"connected":false,"enabled":false,"recording":false,"frame":null},"logic":{"enabled":false,"installed":false,"capture":null}})
        }
        "/api/captures/baseline" => {
            let summary: Value =
                serde_json::from_slice(EXAMPLE).map_err(|_| "Example unavailable")?;
            let data = &summary["reference_measurement_summary"];
            json!({"ok":true,"available":true,"frames":data["captures"],"frequency_hz":data["frequency_hz"],"clipped_frames":data["clipped_captures"]})
        }
        "/api/ddcs/status" => {
            json!({"ok":true,"enabled":false,"busy":false,"comparison":null,"session":null})
        }
        "/api/captures" => json!({"ok":true,"enabled":false,"entries":[],"captures":[],"trash":[]}),
        "/api/scope/status" | "/api/logic/status" | "/api/pico/status" => {
            json!({"ok":true,"connected":false,"enabled":false,"installed":false,"available":false,"status":"disconnected","error":null,"frame":null})
        }
        "/api/library/history" | "/api/library/experiments" => {
            let id = path
                .split_once("?case_id=")
                .map(|(_, id)| id)
                .filter(|id| super::catalog::valid_id(id))
                .ok_or("Invalid artifact identity")?;
            json!({"ok":true,"firmware_sha256":id,"checkpoints":[],"moments":[],"groups":[]})
        }
        "/examples/hardware-rea/summary.json" => return Ok(EXAMPLE.to_vec()),
        _ => return asset(path),
    };
    serde_json::to_vec(&value).map_err(|_| "Cannot encode local data".into())
}

pub(crate) fn map(id: &str) -> Result<Value, String> {
    let bytes = if example()["case_id"] == id {
        asset("/hardware/workbench-map.json")?
    } else {
        if !super::catalog::valid_id(id) {
            return Err("Invalid artifact identity".into());
        }
        let path = super::backend_session::project_root()
            .map_err(|_| "Application data unavailable")?
            .join("library")
            .join(format!("{id}.artifact"));
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|_| "No graph imported for this artifact")?
            .take(16 * 1_048_576 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "Cannot read graph export")?;
        if crate::sha256(&bytes) != id {
            return Err("Graph export identity changed".into());
        }
        bytes
    };
    super::dotdog::parse(&bytes, id)
}

pub(crate) fn overview(id: &str) -> Result<String, String> {
    if example()["case_id"] == id {
        return String::from_utf8(EXAMPLE.to_vec()).map_err(|_| "Invalid example".into());
    }
    if !super::catalog::valid_id(id) {
        return Err("Invalid artifact identity".into());
    }
    let path = super::backend_session::project_root()
        .map_err(|_| "Application data unavailable")?
        .join("library")
        .join(format!("{id}.artifact"));
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|_| "Imported artifact unavailable")?
        .take(128)
        .read_to_end(&mut bytes)
        .map_err(|_| "Cannot read imported artifact")?;
    Ok(bytes
        .chunks(16)
        .enumerate()
        .map(|(index, bytes)| {
            format!(
                "{:08x}  {}",
                index * 16,
                bytes
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

pub fn asset(path: &str) -> Result<Vec<u8>, String> {
    let path = path
        .split('?')
        .next()
        .unwrap_or(path)
        .strip_prefix('/')
        .ok_or("Invalid resource path")?;
    let path = if path.ends_with('/') {
        format!("{path}index.html")
    } else {
        path.to_owned()
    };
    if Path::new(&path)
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
        || path.contains('%')
        || path.contains('\\')
    {
        return Err("Invalid resource path".into());
    }
    let resources: Vec<&str> =
        serde_json::from_str(include_str!("../../../../../web/resources.json"))
            .map_err(|_| "Invalid resource manifest")?;
    let generated = path.starts_with("dist/")
        && matches!(
            Path::new(&path)
                .extension()
                .and_then(|value| value.to_str()),
            Some("js" | "css" | "wasm")
        );
    if path != "build.json" && !generated && !resources.contains(&path.as_str()) {
        return Err("Resource not included".into());
    }
    let packaged = std::env::current_exe().ok().and_then(|exe| {
        #[cfg(target_os = "windows")]
        return exe.parent().map(|root| root.join("resources/web"));
        #[cfg(not(target_os = "windows"))]
        exe.parent()?
            .parent()
            .map(|root| root.join("Resources/web"))
    });
    let root = packaged
        .filter(|root| root.is_dir())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().join("web"));
    let root = root
        .canonicalize()
        .map_err(|_| "Application resources unavailable")?;
    let file = root
        .join(path)
        .canonicalize()
        .map_err(|_| "Resource not included")?;
    if !file.starts_with(&root) || !file.is_file() {
        return Err("Invalid resource path".into());
    }
    let metadata = file.metadata().map_err(|_| "Resource unavailable")?;
    if metadata.len() > 32 * 1024 * 1024 {
        return Err("Resource exceeds 32 MiB".into());
    }
    std::fs::read(file).map_err(|_| "Resource unavailable".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_are_local_content_addressed_and_do_not_claim_analysis() {
        let directory =
            std::env::temp_dir().join(format!("reforge-library-{}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let id = import_into(&directory, Path::new("example.bin"), b"owned example", &[]).unwrap();
        assert_eq!(id, crate::sha256(b"owned example"));
        let cases = library(&directory).unwrap();
        assert_eq!(cases["cases"].as_array().unwrap().len(), 2);
        assert_eq!(cases["cases"][1]["emulation"], "unavailable");
        assert!(import_into(&directory, Path::new("bad"), b"", &[]).is_err());
        assert!(
            import_into(
                &directory,
                Path::new("future.json"),
                br#"{"version":"2.0","project":"example","nodes":[],"edges":[]}"#,
                &[],
            )
            .is_err()
        );
        assert_eq!(
            library(&directory).unwrap()["cases"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        for path in ["/../Cargo.toml", "/%2e%2e/Cargo.toml", "/api/run"] {
            assert!(request(path, None).is_err());
        }
        assert!(request("/api/scope/connect", Some(&json!({}))).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
