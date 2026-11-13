use std::{
    error::Error,
    path::{Path, PathBuf},
};

pub fn project_root() -> Result<PathBuf, Box<dyn Error>> {
    let home = std::env::var_os("HOME").ok_or("Application data directory unavailable")?;
    let root = PathBuf::from(home).join("Library/Application Support/dev.reforge.bench");
    private_directory(&root)?;
    Ok(root)
}

pub(crate) fn private_directory(path: &Path) -> std::io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)?;
    if !std::fs::symlink_metadata(path)?.file_type().is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Application storage must be a directory, not a symlink or file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

pub fn build_identity() -> serde_json::Value {
    serde_json::json!({"application":"reforge-bench", "version":env!("CARGO_PKG_VERSION"),
        "build_number":1,"agent_debug":cfg!(feature="agent-debug"), "transport":"in_process"})
}

pub fn record_diagnostic(event: &serde_json::Value) {
    eprintln!("{event}");
}

pub fn record_metric(message: &str) {
    eprintln!("{message}");
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};

    #[test]
    fn private_storage_rejects_symlinks_without_changing_their_targets() {
        let directory = std::env::temp_dir().join(format!(
            "reforge-storage-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let target = directory.join("target");
        std::fs::create_dir(&target).unwrap();
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o755)).unwrap();
        let link = directory.join("link");
        symlink(&target, &link).unwrap();
        assert!(private_directory(&link).is_err());
        assert_eq!(
            std::fs::metadata(&target).unwrap().permissions().mode() & 0o777,
            0o755
        );
        let regular = directory.join("file");
        std::fs::write(&regular, b"unchanged").unwrap();
        assert!(private_directory(&regular).is_err());
        let storage = directory.join("storage");
        private_directory(&storage).unwrap();
        assert_eq!(
            std::fs::metadata(&storage).unwrap().permissions().mode() & 0o777,
            0o700
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
}
