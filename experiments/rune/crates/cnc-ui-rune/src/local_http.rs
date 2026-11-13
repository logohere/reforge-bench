use serde_json::Value;

pub fn transfer(
    port: u16,
    path: &str,
    body: Option<&Value>,
    limit: usize,
    _timeout: u16,
) -> Result<Vec<u8>, String> {
    let started = std::time::Instant::now();
    let result = super::provider::request(path, body).and_then(|bytes| {
        if bytes.len() > limit {
            Err("Local response exceeds its size limit".into())
        } else {
            Ok(bytes)
        }
    });
    result.map_err(|error| {
        let id = super::diagnostics::record(
            "Local request",
            port,
            None,
            None,
            started.elapsed().as_millis(),
            &error,
        );
        format!("{error} [{id}]")
    })
}
