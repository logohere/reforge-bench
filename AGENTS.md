# Working on Reforge Bench

Start with [README](README.md), [architecture](docs/architecture.md) and
[the AI guide](docs/ai.md). Use a focused branch and PR against `develop`.
Do not merge, publish, change visibility or create a release unless requested.

Preserve the native workspace, hardware views, menus and design tokens. Reuse
existing components. Rust owns the shell, imports, validation, diagnostics and
graph inspection; embedded WebKit (macOS) or WebView2 (Windows) hosts JavaScript hardware visualizations.
Do not describe this as an all-Rust renderer.

Dotdog is for REA evidence mapping, relationships and traces. The repo map is
secondary contributor tooling. Preserve artifact identities, provenance,
unknowns and limitations; graph paths are not execution or measurement proof.

Use Rustfmt and Clippy, typed internal state and JSON format boundaries. Avoid
new frameworks and redundant tests. Run relevant checks and inspect changed UI;
headless rendering does not verify embedded views or real devices.

Do not commit credentials, unit serials, raw private captures, proprietary
firmware, decompiled routines, personal notes or workstation configuration.
Preserve asset licenses. Opening files or graphs must not run firmware, analysis
tools or hardware operations.

Release history is independent. Keep `0.1.0-alpha.1`, build `1`, until released;
do not bump it for unshipped changes. Keep changelog and locks current. Report
actual checks and gaps; builds alone do not establish production readiness.
