use std::{
    fs, io,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

use serde_json::{Value, json};

struct Files(PathBuf);

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

impl Files {
    fn new() -> Self {
        loop {
            let unique = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("reforge-test-{}-{unique}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("Cannot create test directory: {error}"),
            }
        }
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> String {
        let path = self.0.join(name);
        fs::write(&path, contents).unwrap();
        path.to_str().unwrap().into()
    }
}

impl Drop for Files {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn cli(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_reforge-bench"))
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn recorded_example_and_schema_commands_work_from_an_unrelated_directory() {
    let files = Files::new();
    let output = Command::new(env!("CARGO_BIN_EXE_reforge-bench"))
        .args(["demo", "--json"])
        .current_dir(&files.0)
        .output()
        .unwrap();
    assert!(output.status.success());
    let bundled: Value =
        serde_json::from_slice(include_bytes!("../examples/hardware-rea/summary.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        bundled
    );

    let snapshot = files.write(
        "snapshot.json",
        include_bytes!("../examples/hardware-rea/snapshot.json"),
    );
    let direct = cli(&["validate-snapshot", &snapshot, "--json"]);
    assert!(direct.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&direct.stdout).unwrap()["instruments"],
        1
    );
    let request = files.write(
        "request.json",
        json!({"command":"validate-snapshot","path":snapshot,"json":true}).to_string(),
    );
    let requested = cli(&["--request", &request]);
    assert!(requested.status.success());
    assert_eq!(requested.stdout, direct.stdout);

    for schema in ["commands", "snapshot", "diagnostics"] {
        let output = cli(&["schema", schema]);
        assert!(output.status.success());
        assert!(serde_json::from_slice::<Value>(&output.stdout).unwrap()["$schema"].is_string());
    }
    let unknown = files.write("unknown.json", br#"{"command":"demo","unexpected":true}"#);
    assert!(!cli(&["--request", &unknown]).status.success());
}

#[test]
fn diagnostic_import_preserves_privacy_deduplication_and_input_limits() {
    let files = Files::new();
    let private = "private-marker-must-remain-local";
    let event = json!({"kind":"client-error","phase":"uncaught","message":private,
        "stack":private,"path":format!("/{private}"),"source":format!("/{private}"),
        "action":private,"session":private,"line":7});
    let input = files.write("errors.json", json!([event, event]).to_string());
    let output = cli(&["diagnostics", &input]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(!String::from_utf8_lossy(&output.stdout).contains(private));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema"], "reforge-diagnostics-summary/1");
    assert_eq!(report["events"][0]["count"], 2);
    assert_eq!(report["events"][0]["line"], 7);

    for invalid in [
        "[".to_owned(),
        "null".into(),
        json!([{"kind":"client-error","phase":"unsupported"}]).to_string(),
        json!([{"kind":"client-error","phase":"uncaught","message":"🛠".repeat(4096)}]).to_string(),
        serde_json::to_string(&vec![event; 1025]).unwrap(),
        " ".repeat(1_048_577),
    ] {
        let input = files.write("invalid.json", invalid);
        let output = cli(&["diagnostics", &input]);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(!error.contains(private));
        assert!(!error.contains(&input));
    }
}
