use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    #[default]
    Backend,
    Webkit,
    Webview2,
    Rust,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Cause {
    #[serde(default)]
    pub origin: Origin,
    #[serde(default)]
    pub operation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
    #[serde(default)]
    pub detail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Event {
    #[serde(flatten)]
    pub cause: Cause,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub count: u64,
    #[serde(default)]
    pub last_ms: u128,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u64>,
}

#[derive(Default)]
struct Store {
    sequence: u64,
    events: VecDeque<Event>,
}

static EVENTS: OnceLock<Mutex<Store>> = OnceLock::new();

fn bounded(value: &str, limit: usize) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(limit)
        .collect()
}

impl Store {
    fn record(&mut self, mut event: Event) -> String {
        if let Some(previous) = self.events.back_mut()
            && previous.cause == event.cause
        {
            previous.count = previous.count.saturating_add(1);
            previous.last_ms = event.last_ms;
            previous.elapsed_ms = event.elapsed_ms;
            previous.sequence = event.sequence;
            return previous.id.clone();
        }
        self.sequence += 1;
        let id = format!("E{:04}", self.sequence);
        event.id = id.clone();
        event.count = 1;
        if self.events.len() == 64 {
            self.events.pop_front();
        }
        self.events.push_back(event);
        id
    }
}

pub fn record(
    operation: &str,
    port: u16,
    status: Option<u16>,
    code: Option<i32>,
    elapsed_ms: u128,
    detail: &str,
) -> String {
    let event = Event {
        cause: Cause {
            operation: bounded(operation.split('?').next().unwrap_or(operation), 120),
            port: Some(port),
            status,
            code,
            detail: bounded(detail, 320),
            ..Cause::default()
        },
        elapsed_ms: Some(elapsed_ms),
        ..Event::default()
    };
    record_event(event)
}

fn record_event(mut event: Event) -> String {
    event.last_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    super::backend_session::record_diagnostic(&json!(event));
    EVENTS
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .record(event)
}

pub fn client(message: &str) -> bool {
    let Some(event) = super::client_diagnostics::parse(message) else {
        return false;
    };
    let Ok(event) = serde_json::from_value::<Event>(event) else {
        return false;
    };
    record_event(event);
    true
}

pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let detail = info
            .payload()
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| info.payload().downcast_ref::<&str>().copied())
            .unwrap_or("Rust panic");
        let location = info.location();
        // Write directly: a panic may have occurred while the in-memory store was locked.
        super::backend_session::record_diagnostic(&json!({"origin":"rust", "operation":"panic",
            "detail":bounded(detail, 320), "source":location.map(|location| location.file()),
            "line":location.map(|location| location.line())}));
        previous(info);
    }));
}

pub fn events() -> Vec<Event> {
    EVENTS
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .events
        .iter()
        .rev()
        .cloned()
        .collect()
}

pub fn local_report() -> String {
    serde_json::to_string_pretty(
        &json!({"build":super::backend_session::build_identity(),"events":events()}),
    )
    .unwrap()
}

pub fn report(_health: Option<&Value>) -> String {
    use rea_ui_contract::diagnostics::{DiagnosticReport, DiagnosticSummary, ReportOperation};
    let events = events()
        .into_iter()
        .map(|event| DiagnosticSummary {
            id: event.id,
            count: event.count,
            elapsed_ms: event.elapsed_ms.map(|ms| ms.min(u64::MAX as u128) as u64),
            code: event.cause.code,
            status: event.cause.status,
            line: event.cause.line.unwrap_or(0),
            column: event.cause.column.unwrap_or(0),
            operation: match event.cause.operation.as_str() {
                "uncaught" => ReportOperation::Uncaught,
                "rejection" => ReportOperation::Rejection,
                "resource" => ReportOperation::Resource,
                "panic" => ReportOperation::Panic,
                _ => ReportOperation::Error,
            },
        })
        .collect();
    serde_json::to_string_pretty(&DiagnosticReport {
        schema: "reforge-diagnostics-summary/1",
        events,
    })
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_exports_exclude_local_messages_and_location_identifiers() {
        let private = "local-only-diagnostic-marker";
        record_event(Event {
            cause: Cause {
                detail: private.into(),
                path: Some(private.into()),
                operation: private.into(),
                ..Cause::default()
            },
            ..Event::default()
        });
        assert!(local_report().contains(private));
        assert!(!report(None).contains(private));
    }

    #[test]
    fn bounded_deduplicated_diagnostics_keep_the_latest_failure() {
        let mut store = Store::default();
        let event: Event = serde_json::from_value(json!({"operation":"GET /api/library", "status":502,"detail":"metadata missing","last_ms":1})).unwrap();
        assert_eq!(store.record(event.clone()), "E0001");
        assert_eq!(store.record(event), "E0001");
        assert_eq!(store.events[0].count, 2);
        for status in 0..100 {
            store.record(Event {
                cause: Cause {
                    status: Some(status),
                    ..Cause::default()
                },
                ..Event::default()
            });
        }
        assert_eq!(store.events.len(), 64);
        assert_eq!(store.events.back().unwrap().cause.status, Some(99));
        assert_eq!(bounded("a\nb\tc", 2), "ab");
    }

    #[test]
    fn same_message_at_different_actions_is_not_collapsed() {
        let mut store = Store::default();
        let first: Event = serde_json::from_value(
            json!({"operation":"uncaught", "detail":"failed", "action":"load", "last_ms":1}),
        )
        .unwrap();
        assert_eq!(store.record(first.clone()), "E0001");
        assert_eq!(store.record(first), "E0001");
        assert_eq!(
            store.record(
                serde_json::from_value(
                    json!({"operation":"uncaught", "detail":"failed", "action":"play", "last_ms":2})
                )
                .unwrap()
            ),
            "E0002"
        );
    }
}
