# Changelog

## 0.1.0-alpha.1 — Unreleased

- Added Windows x64 source builds, embedded WebView2 hardware views, per-user storage and ZIP packaging.
- Added Windows build/package/UI smoke CI; kept the developer socket inspector macOS-only.
- Fixed repeated preference saves on Windows and used the platform import shortcut and default browser.
- Shared desktop resource packaging and fixed Bun availability during CI packaging.
- Reject pre-existing symlink storage directories and temporary-write collisions.
- Clarify local macOS source builds without Apple signing credentials.
- Include frontend dependencies in update checks and dependency audits.
- Generalized example host labels and removed stale shared-session messaging.
- Removed non-rendering icon provenance metadata without changing pixels.
- Fixed Cargo target-directory resolution and added explicit macOS architecture packaging.
- Added Intel/Apple silicon macOS CI jobs.
- Clarified that diagnostic exports contain summaries, not local error text.
- Updated Wry to 0.57.2 and SHA-2 to 0.11.1; preserved lowercase artifact hashes.
- Moved library milestones into evidence details and identified instrument records.
- Fixed board-pin layout overlap and removed an unavailable reference-image control.
- Pruned unmounted backend-only widgets; kept models, wiring, parsers and plots.
- Reject non-file artifact imports and unsupported Dotdog graph versions.
- Unified the native inspector request schema and removed an unsupported catalog URL.
- Refined hardware navigation, example setup guidance and waveform import.
- Added portable, editable equipment lists with bounded import/export and a schema.
- Generalized artifact import labels and removed duplicate overview navigation.
- Kept sample geometry/pin mappings separate from user equipment records.
- Added native Dotdog export import and evidence relationship inspection.
- Added architecture, AI contributor and REA mapping guides, with upstream credits.
- Added the native shell, Carbon icons, macOS app icon and hardware UI.
- Added isolated, in-process resources and local content-addressed imports; no private host or fixed port.
- Retained device geometry, reference pins, wiring and the independent pulse-fixture framework.
- Included a separate macOS packager and opt-in native/WebKit developer inspector.
- Open completed imports in native evidence inspection rather than Lab.
- Label imported evidence graphs separately from unidentified firmware.
- Fix the missing font token that caused imported-file previews to exit.
- Document native Rust/embedded-view boundaries and incremental parity checks.

- Fixed parallel CLI test directory collisions using atomic IDs and exclusive creation.
- Linked authored contracts to implementation, schema and test files in the Dotdog map.
- Consolidated the public Rust crates under `crates/`.
- Added local CLI commands and JSON Schemas for commands, snapshots and client errors.
- Extracted bounded diagnostics with metadata-only sharing and no automatic uploads.
- Typed local diagnostic records and shared reports; JSON formats remain unchanged.
- Added a reusable-core quickstart, editable example snapshot and CLI regression checks.
- Reused diagnostic-import buffers while preserving input limits and exported fields.
- Extracted the shared bench state contract and existing design tokens.
- Retained one curated hardware/REA example with recorded aggregate results.
- Required Dotdog 1.0.0 tooling and Rust 1.100.0.
- Added publication-boundary checks, secret scanning and source-only packaging.
- Added MIT package metadata, contribution guidance and security reporting policy.

Live acquisition and firmware-analysis adapters must be supplied by the user.
The alpha is not a qualified hardware-control system or a public binary release.
