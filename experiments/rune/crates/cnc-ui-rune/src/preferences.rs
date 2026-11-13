use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::catalog::valid_id;

type Selection = (Option<String>, bool);

pub struct Preferences {
    path: PathBuf,
    saved: Option<Selection>,
}

impl Preferences {
    pub fn load(path: PathBuf) -> Result<(Self, Selection), String> {
        let selection = match std::fs::File::open(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (None, false),
            Err(error) => return Err(format!("Cannot read preferences: {error}")),
            Ok(file) => {
                let mut bytes = Vec::new();
                file.take(8193)
                    .read_to_end(&mut bytes)
                    .map_err(|error| error.to_string())?;
                if bytes.len() > 8192 {
                    return Err("Preferences exceed 8 KiB".into());
                }
                let value: Value = serde_json::from_slice(&bytes)
                    .map_err(|error| format!("Invalid preferences: {error}"))?;
                if value["schema"] != 1 || !value["navigation_collapsed"].is_boolean() {
                    return Err("Unsupported preferences".into());
                }
                let id = match &value["selected_firmware"] {
                    Value::Null => None,
                    Value::String(id) if valid_id(id) => Some(id.clone()),
                    _ => return Err("Saved firmware identity is invalid".into()),
                };
                (id, value["navigation_collapsed"].as_bool().unwrap())
            }
        };
        Ok((
            Self {
                path,
                saved: Some(selection.clone()),
            },
            selection,
        ))
    }

    pub fn save(&mut self, selected: Option<&str>, collapsed: bool) -> Result<(), String> {
        if selected.is_some_and(|id| !valid_id(id)) {
            return Err("Cannot save an invalid firmware identity".into());
        }
        let selection = (selected.map(str::to_owned), collapsed);
        if self.saved.as_ref() == Some(&selection) {
            return Ok(());
        }
        let bytes = serde_json::to_vec_pretty(
            &json!({"schema":1,"selected_firmware":selected,"navigation_collapsed":collapsed}),
        )
        .map_err(|error| error.to_string())?;
        write(&self.path, &bytes).map_err(|error| format!("Cannot save preferences: {error}"))?;
        self.saved = Some(selection);
        Ok(())
    }
}

pub(crate) fn write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("json.{}.next", std::process::id()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    std::fs::rename(temporary, path)
}

pub fn path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home).join("Library/Application Support/dev.reforge.bench/preferences.json")
    })
}

#[cfg(test)]
mod tests {
    use super::super::catalog::DDCS_SHA;
    use super::*;
    #[test]
    fn relaunch_retains_exact_image_and_navigation_without_accepting_a_damaged_identity() {
        let directory = std::env::temp_dir().join(format!(
            "rune-preferences-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = directory.join("preferences.json");
        let (mut preferences, selection) = Preferences::load(path.clone()).unwrap();
        assert_eq!(selection, (None, false));
        preferences.save(Some(DDCS_SHA), true).unwrap();
        let (_, selection) = Preferences::load(path.clone()).unwrap();
        assert_eq!(selection, (Some(DDCS_SHA.into()), true));
        assert!(preferences.save(Some("wrong-image"), false).is_err());
        assert_eq!(Preferences::load(path.clone()).unwrap().1, selection);
        #[cfg(unix)]
        {
            let target = directory.join("unrelated.json");
            std::fs::write(&target, b"unchanged").unwrap();
            let temporary = path.with_extension(format!("json.{}.next", std::process::id()));
            std::os::unix::fs::symlink(&target, &temporary).unwrap();
            assert!(write(&path, b"replacement").is_err());
            assert_eq!(std::fs::read(&target).unwrap(), b"unchanged");
            assert_eq!(Preferences::load(path.clone()).unwrap().1, selection);
            std::fs::remove_file(temporary).unwrap();
        }
        std::fs::write(
            &path,
            b"{\"schema\":1,\"selected_firmware\":\"foreign\",\"navigation_collapsed\":true}",
        )
        .unwrap();
        assert!(Preferences::load(path).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
