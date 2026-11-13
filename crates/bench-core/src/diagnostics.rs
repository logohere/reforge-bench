//! Bounded local diagnostics and a metadata-only report for sharing.
//!
//! Local messages and stacks can contain private user content. The shareable
//! report deliberately excludes them instead of relying on heuristic redaction.

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

const MAX_EVENTS: usize = 64;
const MAX_MESSAGE_BYTES: usize = 16_384;

/// The supported sources of a client-side failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientPhase {
    /// An uncaught client exception.
    Uncaught,
    /// An unhandled promise rejection.
    Rejection,
    /// A failed resource load.
    Resource,
}

/// A normalized client failure. Text fields are private local data.
#[derive(Debug, PartialEq, Eq)]
pub struct ClientError {
    /// Accepted failure phase.
    pub phase: ClientPhase,
    /// Local route without query or fragment.
    pub path: String,
    /// Bounded action identifier.
    pub action: String,
    /// Bounded failure description.
    pub detail: String,
    /// Local source route without query or fragment.
    pub source: String,
    /// Source line, or zero when unavailable.
    pub line: u64,
    /// Source column, or zero when unavailable.
    pub column: u64,
    /// Bounded stack text.
    pub stack: String,
    /// Bounded producer session identifier.
    pub session: String,
    /// Producer-defined sequence, not a timestamp.
    pub sequence: u64,
}

/// Failure details retained for local inspection, never included in shared text.
#[derive(Debug, PartialEq, Eq)]
pub enum Failure {
    /// A normalized client envelope.
    Client(ClientError),
    /// An application failure.
    Application {
        /// Local operation label.
        operation: String,
        /// Bounded description.
        detail: String,
    },
    /// A local transport failure.
    Transfer {
        /// Local request label without query parameters.
        operation: String,
        /// Local endpoint port.
        port: u16,
        /// HTTP status, when available.
        status: Option<u16>,
        /// Process or transport code, when available.
        code: Option<i32>,
        /// Bounded description.
        detail: String,
    },
}

impl Failure {
    fn same_failure(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Client(a), Self::Client(b)) => {
                a.phase == b.phase
                    && a.path == b.path
                    && a.action == b.action
                    && a.detail == b.detail
                    && a.source == b.source
                    && a.line == b.line
                    && a.column == b.column
                    && a.stack == b.stack
                    && a.session == b.session
            }
            _ => self == other,
        }
    }
}

/// One private local failure and its deduplication metadata.
#[derive(Debug)]
pub struct DiagnosticEvent {
    /// Collector-generated event identity.
    pub id: String,
    /// Adjacent occurrences of this failure.
    pub count: u64,
    /// Host recording time in Unix milliseconds, not the instrument clock.
    pub last_ms: u64,
    /// Latest transfer duration in milliseconds, when applicable.
    pub elapsed_ms: Option<u64>,
    /// Private details, inspected explicitly rather than serialized for sharing.
    pub failure: Failure,
}

/// Closed categories permitted in shared reports; arbitrary labels cannot leak.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportOperation {
    /// An uncaught client exception.
    Uncaught,
    /// An unhandled promise rejection.
    Rejection,
    /// A failed resource load.
    Resource,
    /// An explicitly recorded panic.
    Panic,
    /// Any other failure, without its private operation label.
    Error,
}

/// Allowlisted metadata for one failure, without private details or locations.
#[derive(Debug, Serialize)]
pub struct DiagnosticSummary {
    /// Process or transport code, when available.
    pub code: Option<i32>,
    /// Source column, or zero when unavailable.
    pub column: u64,
    /// Adjacent occurrence count.
    pub count: u64,
    /// Latest transfer duration, when applicable.
    pub elapsed_ms: Option<u64>,
    /// Collector-generated identity.
    pub id: String,
    /// Source line, or zero when unavailable.
    pub line: u64,
    /// Allowlisted failure category.
    pub operation: ReportOperation,
    /// HTTP status, when available.
    pub status: Option<u16>,
}

/// A serializable metadata-only report, newest failures first.
#[derive(Debug, Serialize)]
pub struct DiagnosticReport {
    /// Allowlisted failure summaries.
    pub events: Vec<DiagnosticSummary>,
    /// Persisted report format identifier.
    pub schema: &'static str,
}

fn bounded(value: &str, limit: usize) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(limit)
        .collect()
}

fn text(value: &Value, key: &str, limit: usize) -> String {
    bounded(value[key].as_str().unwrap_or_default(), limit)
}

fn path(value: &Value, key: &str) -> String {
    let value = text(value, key, 240);
    let value = value.split(['?', '#']).next().unwrap_or_default();
    if value.starts_with('/') && !value.starts_with("//") && !value.contains('\\') {
        value.into()
    } else {
        String::new()
    }
}

/// Parse the existing client-error envelope without retaining arbitrary payloads.
///
/// The bounded message and stack remain private local data. Use
/// [`Diagnostics::shareable_report`] when producing a report for others.
pub fn parse_client_error(message: &str) -> Option<ClientError> {
    if message.len() > MAX_MESSAGE_BYTES {
        return None;
    }
    let value: Value = serde_json::from_str(message).ok()?;
    if value["kind"] != "client-error" {
        return None;
    }
    let phase = match value["phase"].as_str()? {
        "uncaught" => ClientPhase::Uncaught,
        "rejection" => ClientPhase::Rejection,
        "resource" => ClientPhase::Resource,
        _ => return None,
    };
    let action = text(&value, "action", 96);
    let action = if action
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "_.:-".contains(c))
    {
        action
    } else {
        String::new()
    };
    Some(ClientError {
        phase,
        path: path(&value, "path"),
        action,
        detail: text(&value, "message", 320),
        source: path(&value, "source"),
        line: value["line"].as_u64().unwrap_or(0),
        column: value["column"].as_u64().unwrap_or(0),
        stack: text(&value, "stack", 1600),
        session: text(&value, "session", 64),
        sequence: value["sequence"].as_u64().unwrap_or(0),
    })
}

/// The latest 64 distinct local failures; adjacent duplicates share an event ID.
///
/// This collector does not install hooks, write files or upload data.
///
/// ```
/// use reforge_bench_core::diagnostics::{Diagnostics, ReportOperation};
///
/// let mut diagnostics = Diagnostics::default();
/// diagnostics.record_failure("import", "Selected file is unavailable");
/// let report = diagnostics.shareable_report();
/// assert_eq!(report.events[0].operation, ReportOperation::Error);
/// assert_eq!(report.events[0].count, 1);
/// let json = serde_json::to_string_pretty(&report)?;
/// assert!(!json.contains("Selected file"));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Default)]
pub struct Diagnostics {
    sequence: u64,
    events: VecDeque<DiagnosticEvent>,
}

impl Diagnostics {
    /// Record an accepted client envelope, returning its local event ID.
    pub fn record_client(&mut self, message: &str) -> Option<String> {
        Some(self.push(Failure::Client(parse_client_error(message)?), None))
    }

    /// Record a bounded application failure for private local inspection.
    pub fn record_failure(&mut self, operation: &str, detail: &str) -> String {
        self.push(
            Failure::Application {
                operation: bounded(operation, 120),
                detail: bounded(detail, 320),
            },
            None,
        )
    }

    /// Record a local transport failure, keeping request details out of exports.
    pub fn record_transfer(
        &mut self,
        operation: &str,
        port: u16,
        status: Option<u16>,
        code: Option<i32>,
        elapsed_ms: u128,
        detail: &str,
    ) -> String {
        self.push(
            Failure::Transfer {
                operation: bounded(operation.split('?').next().unwrap_or(operation), 120),
                port,
                status,
                code,
                detail: bounded(detail, 320),
            },
            Some(elapsed_ms.min(u64::MAX as u128) as u64),
        )
    }

    /// Read private local events, newest first, without cloning their contents.
    pub fn local_events(&self) -> impl Iterator<Item = &DiagnosticEvent> {
        self.events.iter().rev()
    }

    /// Export IDs, categories, counts, coordinates and numeric transfer results.
    ///
    /// Messages, stacks, routes, actions, sessions and source paths are omitted.
    pub fn shareable_report(&self) -> DiagnosticReport {
        let events = self
            .local_events()
            .map(|event| {
                let mut summary = DiagnosticSummary {
                    code: None,
                    column: 0,
                    count: event.count,
                    elapsed_ms: event.elapsed_ms,
                    id: event.id.clone(),
                    line: 0,
                    operation: ReportOperation::Error,
                    status: None,
                };
                match &event.failure {
                    Failure::Client(client) => {
                        summary.operation = match client.phase {
                            ClientPhase::Uncaught => ReportOperation::Uncaught,
                            ClientPhase::Rejection => ReportOperation::Rejection,
                            ClientPhase::Resource => ReportOperation::Resource,
                        };
                        summary.line = client.line;
                        summary.column = client.column;
                    }
                    Failure::Application { operation, .. }
                    | Failure::Transfer { operation, .. } => {
                        summary.operation = match operation.as_str() {
                            "uncaught" => ReportOperation::Uncaught,
                            "rejection" => ReportOperation::Rejection,
                            "resource" => ReportOperation::Resource,
                            "panic" => ReportOperation::Panic,
                            _ => ReportOperation::Error,
                        };
                        if let Failure::Transfer { status, code, .. } = &event.failure {
                            summary.status = *status;
                            summary.code = *code;
                        }
                    }
                }
                summary
            })
            .collect();
        DiagnosticReport {
            events,
            schema: "reforge-diagnostics-summary/1",
        }
    }

    fn push(&mut self, failure: Failure, elapsed_ms: Option<u64>) -> String {
        let last_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .min(u64::MAX as u128) as u64;
        if let Some(previous) = self.events.back_mut()
            && previous.failure.same_failure(&failure)
        {
            previous.count = previous.count.saturating_add(1);
            previous.last_ms = last_ms;
            previous.elapsed_ms = elapsed_ms;
            return previous.id.clone();
        }
        self.sequence += 1;
        let id = format!("E{:04}", self.sequence);
        if self.events.len() == MAX_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(DiagnosticEvent {
            id: id.clone(),
            count: 1,
            last_ms,
            elapsed_ms,
            failure,
        });
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_only_bounded_client_failures_without_query_or_payload() {
        let message = json!({"kind":"client-error","phase":"rejection","path":"/lab/?private=1#x",
            "source":"https://foreign.invalid/private", "message":"a\nb", "stack":"s".repeat(3000),
            "action":"material", "line":-1, "samples":[1,2,3]})
        .to_string();
        let event = parse_client_error(&message).unwrap();
        assert_eq!(event.phase, ClientPhase::Rejection);
        assert_eq!(event.path, "/lab/");
        assert_eq!(event.source, "");
        assert_eq!(event.detail, "ab");
        assert_eq!(event.line, 0);
        assert_eq!(event.stack.len(), 1600);
        for message in [
            "{}",
            "[]",
            "null",
            "bad",
            &"x".repeat(16385),
            r#"{"kind":"client-error","phase":"arbitrary"}"#,
        ] {
            assert!(parse_client_error(message).is_none());
        }
    }

    #[test]
    fn bounded_deduplicated_diagnostics_keep_the_latest_failure() {
        let mut store = Diagnostics::default();
        assert_eq!(store.record_failure("load", "metadata missing"), "E0001");
        assert_eq!(store.record_failure("load", "metadata missing"), "E0001");
        assert_eq!(store.events[0].count, 2);
        for i in 0..100 {
            store.record_failure("load", &format!("failure {i}"));
        }
        assert_eq!(store.events.len(), 64);
        assert!(matches!(
            &store.events.back().unwrap().failure,
            Failure::Application { detail, .. } if detail == "failure 99"
        ));
    }

    #[test]
    fn same_message_at_different_actions_is_not_collapsed() {
        let mut store = Diagnostics::default();
        let mut event =
            json!({"kind":"client-error","phase":"uncaught","message":"failed","action":"load"});
        assert_eq!(store.record_client(&event.to_string()).unwrap(), "E0001");
        event["sequence"] = json!(2);
        assert_eq!(store.record_client(&event.to_string()).unwrap(), "E0001");
        event["action"] = json!("play");
        assert_eq!(store.record_client(&event.to_string()).unwrap(), "E0002");
        assert_eq!(
            store.record_transfer("GET /api/library", 0, Some(502), None, 1, "failed"),
            "E0003"
        );
        assert_eq!(
            store.record_transfer("GET /api/library", 0, Some(503), None, 1, "failed"),
            "E0004"
        );
        assert_eq!(
            store.record_transfer("GET /api/library", 0, Some(503), None, 9, "failed"),
            "E0004"
        );
        let summary = store.shareable_report();
        assert_eq!(summary.events[0].count, 2);
        assert_eq!(summary.events[0].elapsed_ms, Some(9));
    }

    #[test]
    fn shareable_reports_never_copy_free_text_or_location_identifiers() {
        let mut store = Diagnostics::default();
        let private = "private-marker-must-remain-local";
        let event = json!({"kind":"client-error","phase":"uncaught","message":private,
            "stack":private,"path":format!("/{private}"),"source":format!("/{private}"),
            "session":private,"action":private,"line":7});
        store.record_client(&event.to_string()).unwrap();
        store.record_failure(private, private);
        store.record_transfer(
            &format!("GET /{private}?token={private}"),
            1234,
            Some(502),
            Some(-1),
            u128::MAX,
            private,
        );
        assert!(store.local_events().any(|event| matches!(
            &event.failure,
            Failure::Client(client) if client.detail == private
        )));
        let report = store.shareable_report();
        assert!(!serde_json::to_string(&report).unwrap().contains(private));
        assert_eq!(report.events[0].operation, ReportOperation::Error);
        assert_eq!(report.events[0].status, Some(502));
        assert_eq!(report.events[0].code, Some(-1));
        assert_eq!(report.events[0].elapsed_ms, Some(u64::MAX));
        assert_eq!(report.events[1].operation, ReportOperation::Error);
        assert_eq!(report.events[2].line, 7);
    }
}
