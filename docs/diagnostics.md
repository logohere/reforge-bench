# Diagnostics

`reforge_bench_core::diagnostics::Diagnostics` is an in-memory collector extracted
from the bench's error handling. It installs no hooks, writes no logs and uploads
nothing automatically.

- `record_client` accepts bounded uncaught-error, rejection and resource envelopes.
- `record_failure` records a bounded application failure.
- `record_transfer` retains local status, exit code, duration and request context.
- `parse_client_error` normalizes a JSON envelope into a typed `ClientError`.
- `local_events` borrows typed `DiagnosticEvent` records, newest first.
- `shareable_report` returns a typed `DiagnosticReport` with generated IDs, categories, counts,
  source coordinates and numeric transfer results.

`Failure` distinguishes client, application and transfer failures. Match its
variants instead of looking up string keys. `ReportOperation` is a closed enum:
arbitrary private operation labels cannot become report categories.

Local records intentionally do not implement `Serialize`. Their `Debug` output
is private too. Serialize the shared report explicitly with
`serde_json::to_string_pretty(&diagnostics.shareable_report())`. The JSON input
and report formats are unchanged; this pre-release Rust API now returns typed
values instead of JSON objects. The collector's API documentation includes a
compiled usage example (`cargo test --doc --workspace`).

Messages, stacks, paths and session/action labels remain local. Query/fragment
removal and length limits do not make arbitrary free text safe to publish.
Shared reports therefore omit free text rather than trying to scrub every
possible secret pattern.

The ring holds 64 distinct failures. Adjacent identical failures are collapsed;
a different action, request status or source remains a separate record. The local
timestamp is the host's recording time, not the instrument's acquisition clock.

This is collection/reporting code, not a live inspection endpoint. Calling the
API does not start a server, authenticate a device or install a panic hook.
