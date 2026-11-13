use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};

use super::catalog;
use rune::Waker;
use serde_json::Value;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Library,
    Rea,
    Timeline,
    Hardware,
    Workspace,
    Replay,
}

pub struct Snapshot {
    pub health: Value,
    pub recovery: Value,
    pub cases: Vec<Value>,
}

impl Snapshot {
    pub(crate) fn parse(health: Value, library: Value, recovery: Value) -> Result<Self, String> {
        if health["ok"] != true
            || health["version"].as_str().is_none()
            || health["frontend"].as_str().is_none()
        {
            return Err("The local service is not a recognized Reforge provider".into());
        }
        if library["ok"] != true || !recovery["profiles"].is_object() {
            return Err("The host returned invalid library evidence".into());
        }
        let cases = library["cases"]
            .as_array()
            .ok_or("The host returned an invalid artifact library")?
            .clone();
        let mut ids = std::collections::HashSet::new();
        if cases.iter().any(|case| {
            !catalog::valid_id(case["case_id"].as_str().unwrap_or_default())
                || !ids.insert(case["case_id"].as_str())
                || !case["library"].is_object()
        }) {
            return Err("The host returned ambiguous artifact identities".into());
        }
        Ok(Self {
            recovery: recovery["profiles"].clone(),
            health,
            cases,
        })
    }
}

type Reply = Result<(Snapshot, Option<String>), String>;

#[derive(Default)]
pub struct Connection {
    pub port: u16,
    pub snapshot: Option<Snapshot>,
    pub error: Option<String>,
    pub selected: Option<String>,
    pub loading: bool,
    pub waker: Option<Waker>,
    pub imported: Option<String>,
    receiver: Option<Receiver<Reply>>,
}

pub(crate) fn read(port: u16, path: &str) -> Result<Value, String> {
    serde_json::from_slice(&read_bytes(port, path)?)
        .map_err(|error| format!("Invalid host response: {error}"))
}

pub(crate) fn read_bytes(port: u16, path: &str) -> Result<Vec<u8>, String> {
    let history = [
        "/api/library/history?case_id=",
        "/api/library/experiments?case_id=",
    ]
    .iter()
    .any(|prefix| path.strip_prefix(prefix).is_some_and(catalog::valid_id));
    let image = path
        .strip_prefix("/timeline/artifacts/")
        .and_then(|name| name.strip_suffix(".png"))
        .is_some_and(catalog::valid_id);
    if !matches!(
        path,
        "/api/desktop/health"
            | "/api/library"
            | "/library/recovery-status.json"
            | "/process/binary.json"
            | "/process/timeline.json"
            | "/timeline/evidence.json"
            | "/process/firmware-map.json"
            | "/process/recovery-map.json"
    ) && !history
        && !image
    {
        return Err("Unsupported read-only host operation".into());
    }
    let large = history || path == "/process/firmware-map.json";
    let limit = if large { 16_777_216 } else { 1_048_576 };
    super::local_http::transfer(port, path, None, limit, if large { 10 } else { 3 })
}

fn fetch(port: u16) -> Result<Snapshot, String> {
    let health = read(port, "/api/desktop/health")?;
    let library = read(port, "/api/library")?;
    let recovery = read(port, "/library/recovery-status.json")?;
    let after = read(port, "/api/desktop/health")?;
    if health != after {
        return Err("Host build changed during refresh. Refresh again.".into());
    }
    Snapshot::parse(health, library, recovery)
}

impl Connection {
    fn start(&mut self, operation: &'static str, work: impl FnOnce(u16) -> Reply + Send + 'static) {
        if self.loading {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        let port = self.port;
        let waker = self.waker.clone();
        self.loading = true;
        self.error = None;
        self.receiver = Some(receiver);
        std::thread::spawn(move || {
            let started = std::time::Instant::now();
            let result = work(port).map_err(|error| {
                if error.ends_with(']') && error.contains(" [E") {
                    return error;
                }
                let id = super::diagnostics::record(
                    operation,
                    port,
                    None,
                    None,
                    started.elapsed().as_millis(),
                    &error,
                );
                format!("{error} [{id}]")
            });
            let _ = sender.send(result);
            if let Some(wake) = waker {
                wake();
            }
        });
    }

    pub fn refresh(&mut self) {
        self.start("Library refresh", move |port| {
            fetch(port).map(|snapshot| (snapshot, None))
        });
    }

    pub fn import(&mut self, path: PathBuf, labels: Vec<(String, String)>) {
        self.start("Artifact import", move |port| {
            let id = import(port, &path, &labels)?;
            let snapshot = fetch(port)?;
            if !snapshot.cases.iter().any(|item| item["case_id"] == id) {
                return Err("Imported artifact is not visible. Refresh the library.".into());
            }
            Ok((snapshot, Some(id)))
        });
    }

    pub fn poll(&mut self) -> bool {
        let Some(result) = self
            .receiver
            .as_ref()
            .and_then(|receiver| receiver.try_recv().ok())
        else {
            return false;
        };
        self.loading = false;
        self.receiver = None;
        match result {
            Ok((snapshot, imported)) => {
                if let Some(id) = &imported {
                    self.selected = Some(id.clone());
                }
                self.imported = imported;
                if self.selected.is_none() {
                    self.selected = snapshot
                        .cases
                        .iter()
                        .find(|case| case["case_id"] == catalog::DDCS_SHA)
                        .or_else(|| snapshot.cases.first())
                        .and_then(|case| case["case_id"].as_str())
                        .map(str::to_owned);
                }
                self.snapshot = Some(snapshot);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
        true
    }
}

fn import(_port: u16, path: &Path, labels: &[(String, String)]) -> Result<String, String> {
    if !path.metadata().is_ok_and(|metadata| metadata.is_file()) {
        return Err("Choose a regular artifact file.".into());
    }
    let file = std::fs::File::open(path).map_err(|_| "Cannot read selected artifact")?;
    let mut bytes = Vec::new();
    file.take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Cannot read selected artifact")?;
    super::provider::import(path, &bytes, labels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn failed_refresh_retains_data_and_selection() {
        let snapshot = Snapshot::parse(
            json!({"ok":true,"version":"test","frontend":"test"}),
            json!({"ok":true,"cases":[]}),
            json!({"profiles":{}}),
        )
        .unwrap();
        let (sender, receiver) = mpsc::channel();
        let mut connection = Connection {
            snapshot: Some(snapshot),
            selected: Some(catalog::DDCS_SHA.into()),
            loading: true,
            receiver: Some(receiver),
            ..Default::default()
        };
        sender.send(Err("Host unavailable".into())).unwrap();
        assert!(connection.poll());
        assert!(connection.snapshot.is_some());
        assert_eq!(connection.selected.as_deref(), Some(catalog::DDCS_SHA));
        assert!(!connection.loading);
        assert_eq!(connection.error.as_deref(), Some("Host unavailable"));
    }
    #[test]
    fn rejects_foreign_services_ambiguous_identities_and_commands() {
        let health = json!({"ok":true,"version":"test","frontend":"test"});
        let recovery = json!({"profiles":{}});
        assert!(
            Snapshot::parse(
                json!({"ok":true}),
                json!({"ok":true,"cases":[]}),
                recovery.clone()
            )
            .is_err()
        );
        let item = json!({"case_id":catalog::DDCS_SHA,"library":{}});
        assert!(
            Snapshot::parse(
                health.clone(),
                json!({"ok":true,"cases":[item.clone(),item]}),
                recovery.clone()
            )
            .is_err()
        );
        assert!(
            Snapshot::parse(
                health,
                json!({"ok":true,"cases":[{"case_id":"foreign","library":{}}]}),
                recovery
            )
            .is_err()
        );
        assert!(read(0, "/api/run").is_err());
        assert!(import(0, &std::env::temp_dir(), &[]).is_err());
    }
}
