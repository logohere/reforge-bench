use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    time::Duration,
};

const LIMIT: usize = 2_097_152;

fn socket_path() -> Result<PathBuf, String> {
    Ok(super::backend_session::project_root()
        .map_err(|error| error.to_string())?
        .join("agent-debug/desktop.sock"))
}

fn schema() -> Value {
    serde_json::from_str(include_str!("../../../schemas/inspector.json"))
        .expect("embedded inspector schema must be valid JSON")
}

pub fn inspect(arguments: &[String]) -> Result<(), String> {
    if arguments == ["--schema"] {
        println!("{}", serde_json::to_string_pretty(&schema()).unwrap());
        return Ok(());
    }
    let Some(method) = arguments.first() else {
        return Err("Use inspect snapshot|diagnostics|web.snapshot|web.evaluate EXPRESSION|web.inspector or inspect --schema".into());
    };
    let mut request = json!({"id":1,"method":method});
    if let Some(expression) = arguments.get(1) {
        request["expression"] = json!(expression);
    }
    validate(&request)?;
    if arguments.len() > if method == "web.evaluate" { 2 } else { 1 } {
        return Err("Unexpected inspector arguments".into());
    }
    let path = socket_path()?;
    let mut stream = UnixStream::connect(&path).map_err(|error| {
        format!(
            "{}: {error}. Start an agent-debug build; normal app builds do not enable the inspector.",
            path.display()
        )
    })?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    writeln!(stream, "{request}").map_err(|error| error.to_string())?;
    let mut response = String::new();
    std::io::Read::take(BufReader::new(stream), LIMIT as u64 + 1)
        .read_line(&mut response)
        .map_err(|error| error.to_string())?;
    if response.len() > LIMIT || response.is_empty() {
        return Err("Invalid inspector response".into());
    }
    let value: Value = serde_json::from_str(&response).map_err(|error| error.to_string())?;
    if value["schema"] != "reforge-inspector/1" || value["id"] != request["id"] {
        return Err("Inspector response does not match this protocol/request".into());
    }
    println!("{}", serde_json::to_string_pretty(&value).unwrap());
    if value["ok"] == true {
        Ok(())
    } else {
        Err("Inspector command failed; see the structured response".into())
    }
}

fn validate(request: &Value) -> Result<(), String> {
    let fields = request
        .as_object()
        .ok_or("Expected an inspector request object")?;
    if fields
        .keys()
        .any(|key| !matches!(key.as_str(), "id" | "method" | "expression"))
        || request["id"].as_u64().is_none()
    {
        return Err("Invalid inspector request fields".into());
    }
    match request["method"].as_str() {
        Some("snapshot" | "diagnostics" | "web.snapshot" | "web.inspector")
            if !fields.contains_key("expression") =>
        {
            Ok(())
        }
        Some("web.evaluate")
            if request["expression"].as_str().is_some_and(|expression| {
                !expression.trim().is_empty() && expression.len() <= 8192
            }) =>
        {
            Ok(())
        }
        _ => Err("Unknown inspector method or invalid expression".into()),
    }
}

#[cfg(all(feature = "agent-debug", target_os = "macos"))]
mod live {
    use super::*;
    use rune::{AppHandle, compose::Frame};
    use std::{
        collections::VecDeque,
        os::unix::{
            fs::{FileTypeExt, PermissionsExt},
            net::UnixListener,
        },
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        thread,
    };

    type Pending = (Value, mpsc::SyncSender<Value>);

    pub struct Inspector {
        pending: Arc<Mutex<VecDeque<Pending>>>,
        stop: Arc<AtomicBool>,
        worker: Option<thread::JoinHandle<()>>,
        path: PathBuf,
    }

    impl Inspector {
        pub fn start(handle: AppHandle<super::super::State>) -> Result<Option<Self>, String> {
            if super::super::backend_session::build_identity()["agent_debug"] != true
                && std::env::var("REFORGE_AGENT_DEBUG").as_deref() != Ok("1")
            {
                return Ok(None);
            }
            let path = socket_path()?;
            let directory = path.parent().unwrap();
            super::super::backend_session::private_directory(directory)
                .map_err(|error| error.to_string())?;
            if let Ok(metadata) = std::fs::symlink_metadata(&path) {
                if !metadata.file_type().is_socket() || UnixStream::connect(&path).is_ok() {
                    return Err("Inspector path is already occupied".into());
                }
                std::fs::remove_file(&path).map_err(|error| error.to_string())?;
            }
            let listener = UnixListener::bind(&path).map_err(|error| error.to_string())?;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                .map_err(|error| error.to_string())?;
            listener
                .set_nonblocking(true)
                .map_err(|error| error.to_string())?;
            let pending = Arc::new(Mutex::new(VecDeque::new()));
            let stop = Arc::new(AtomicBool::new(false));
            let (requests, stopped) = (pending.clone(), stop.clone());
            let worker = thread::spawn(move || {
                while !stopped.load(Ordering::Relaxed) {
                    let (mut stream, _) = match listener.accept() {
                        Ok(connection) => connection,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(20));
                            continue;
                        }
                        Err(_) => break,
                    };
                    if stream.set_nonblocking(false).is_err() {
                        continue;
                    }
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
                    let mut raw = String::new();
                    let parsed = std::io::Read::take(BufReader::new(&stream), 16_385)
                        .read_line(&mut raw)
                        .map_err(|error| error.to_string())
                        .and_then(|_| {
                            if raw.len() > 16_384 {
                                return Err("Inspector request exceeds limit".into());
                            }
                            serde_json::from_str::<Value>(&raw).map_err(|error| error.to_string())
                        })
                        .and_then(|request| validate(&request).map(|()| request));
                    let (id, result) = match parsed {
                        Ok(request) => {
                            let id = request["id"].clone();
                            let (sender, receiver) = mpsc::sync_channel(1);
                            let mut pending =
                                requests.lock().unwrap_or_else(|error| error.into_inner());
                            if pending.len() == 8 {
                                pending.pop_front();
                            }
                            pending.push_back((request, sender));
                            drop(pending);
                            handle.update(|_, _| {});
                            let result = receiver.recv_timeout(Duration::from_secs(4)).unwrap_or_else(|_| json!({"ok":false,"error":"Inspector timed out; the main thread or web view may be stalled"}));
                            (id, result)
                        }
                        Err(error) => (Value::Null, json!({"ok":false,"error":error})),
                    };
                    let mut response = json!({"schema":"reforge-inspector/1","id":id,"build":super::super::backend_session::build_identity(),"ok":result["ok"],"result":result["result"],"error":result["error"]}).to_string();
                    if response.len() > LIMIT {
                        response = json!({"schema":"reforge-inspector/1","id":id,"ok":false,"error":"Inspector response exceeds 2 MiB; narrow the query"}).to_string();
                    }
                    let _ = writeln!(stream, "{response}");
                }
            });
            Ok(Some(Self {
                pending,
                stop,
                worker: Some(worker),
                path,
            }))
        }

        pub fn poll(&mut self, frame: &Frame<'_, super::super::State>, web: Option<&wry::WebView>) {
            let request = self
                .pending
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .pop_front();
            let Some((request, sender)) = request else {
                return;
            };
            let result = match request["method"].as_str().unwrap() {
                "snapshot" => {
                    let elements: Vec<_> = frame.rt.elements().into_iter().map(|element| json!({
                        "key":element.key,"path":element.path,"role":element.role,"label":element.label,
                        "text":element.text,"rect":element.rect,"visible":element.visible,"target":element.target,
                        "focusable":element.focusable,"focused":element.focused,"inert":element.inert,
                        "moving":element.moving,"parent":element.parent,"help":element.help})).collect();
                    json!({"ok":true,"result":{"renderer":"rune","clock":frame.rt.clock,"update_ms":frame.update_ms,
                        "size":frame.size,"selected_case":frame.rt.state.host.selected,"elements":elements,
                        "web_available":web.is_some()}})
                }
                "diagnostics" => {
                    json!({"ok":true,"result":serde_json::from_str::<Value>(&super::super::diagnostics::report(frame.rt.state.host.snapshot.as_ref().map(|snapshot| &snapshot.health))).unwrap()})
                }
                "web.inspector" => {
                    if let Some(web) = web {
                        web.open_devtools();
                        json!({"ok":true,"result":{"requested":true}})
                    } else {
                        json!({"ok":false,"error":"Open an embedded workspace first"})
                    }
                }
                method => {
                    let Some(web) = web else {
                        let _ = sender
                            .send(json!({"ok":false,"error":"Open an embedded workspace first"}));
                        return;
                    };
                    let expression = if method == "web.snapshot" {
                        "window.__reaAgent.snapshot()"
                    } else {
                        request["expression"].as_str().unwrap()
                    };
                    let script = format!(
                        "(()=>{{try{{const result=({expression});if(result&&typeof result.then==='function')throw new Error('Only synchronous expressions are supported');return JSON.stringify({{ok:true,result:result===undefined?null:result}})}}catch(error){{return JSON.stringify({{ok:false,error:String(error).slice(0,1600)}})}}}})()"
                    );
                    let callback = sender.clone();
                    if let Err(error) = web.evaluate_script_with_callback(&script, move |raw| {
                        let result = serde_json::from_str::<Value>(&raw)
                            .ok()
                            .and_then(|value| match value {
                                Value::String(value) => serde_json::from_str(&value).ok(),
                                Value::Object(_) => Some(value),
                                _ => None,
                            })
                            .unwrap_or_else(
                                || json!({"ok":false,"error":"Web evaluation returned no result"}),
                            );
                        let _ = callback.send(result);
                    }) {
                        let _ = sender.send(json!({"ok":false,"error":error.to_string()}));
                    }
                    return;
                }
            };
            let _ = sender.send(result);
        }
    }

    impl Drop for Inspector {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

#[cfg(all(feature = "agent-debug", target_os = "macos"))]
pub use live::Inspector;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspector_rejects_unknown_commands_and_unbounded_expressions() {
        assert!(validate(&json!({"id":1,"method":"snapshot"})).is_ok());
        assert!(
            validate(&json!({"id":2,"method":"web.evaluate","expression":"document.title"}))
                .is_ok()
        );
        for request in [
            json!({"id":1,"method":"shell"}),
            json!({"id":1,"method":"snapshot","extra":true}),
            json!({"method":"snapshot"}),
            json!({"id":1,"method":"snapshot","expression":"sideEffect()"}),
            json!({"id":1,"method":"web.evaluate","expression":"x".repeat(8193)}),
            json!({"id":1,"method":"web.evaluate","expression":""}),
            json!({"id":1,"method":"web.evaluate","expression":"  \n"}),
            json!({"id":-1,"method":"snapshot"}),
            json!({"id":1.5,"method":"snapshot"}),
        ] {
            assert!(validate(&request).is_err());
        }
    }
}
