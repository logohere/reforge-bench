# Local CLI

Run from source with `cargo run --locked -- COMMAND`. Use `--help` for syntax.
Commands are read-only apart from stdout/stderr. They do not install providers,
open a browser, start a server or control a running desktop or hardware.

| Command | Input | Output |
| --- | --- | --- |
| `demo [--json]` | Bundled recorded example | Human-readable summary or original JSON |
| `validate-snapshot FILE [--json]` | Saved bench snapshot | Validation result and counts |
| `diagnostics FILE` | JSON array of client errors | Metadata-only report, newest failures first |
| `schema [commands\|snapshot\|diagnostics]` | Schema selection | JSON Schema |
| `--request FILE` | Schema-described command JSON | The selected command's output |

For a programmatic request, put this object in a JSON file and pass its path to
`--request`:

```json
{"command":"demo","json":true}
```

Unknown command fields and unsupported commands are rejected. Every selected
file must be a regular file no larger than 1 MiB. Invalid input exits nonzero;
errors do not echo the document, its field values or a full local path.

## Snapshot validation

`schema snapshot` describes the shared machine/instrument document. Validation
checks the schema version, finite numeric values and non-negative spindle speed.
Unknown fields are ignored for compatibility. A `live` label is a producer claim,
not authentication, current connectivity or calibration.

Try the included case's state envelope, then adapt a copy for your own adapter:

```sh
cargo run --locked -- validate-snapshot crates/bench-cli/examples/hardware-rea/snapshot.json --json
```

Coordinates and capture time are unavailable in this envelope. Its local device
identity is an example label, not a serial number. The recorded measurements and
findings remain in the case's `summary.json`; no raw waveform is included.

## Diagnostic input

`schema diagnostics` describes the existing client-error envelope. The input is
an array of at most 1,024 entries. Each serialized entry is limited to 16 KiB.
Only `uncaught`, `rejection` and `resource` phases are accepted. Invalid entries
fail the command rather than silently disappearing from a partial report.

The collector retains at most 64 distinct failures. Adjacent duplicate failures
increment a count. Shared output omits messages, stacks, routes, source paths,
actions, sessions and raw payloads. See [diagnostics](diagnostics.md).
