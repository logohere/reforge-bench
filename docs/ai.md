# AI contributor guide

Read [AGENTS.md](../AGENTS.md) first. This is a codebase guide, not a separate
agent runtime or permission grant.

## Find the code

- Layout, stable control keys and menus: `workspace.rs`.
- Lifecycle, fixture replay and Rune tooling: `lib.rs`.
- Local imports, limits and storage: `provider.rs`.
- REA mapping: `dotdog.rs`, `graph_model.rs`, `graph.rs`.
- Hardware: `web/hardware/layout.js`, `tabletop.js`, `wires.js`.
- User equipment records: `web/hardware/equipment-config.js`, `equipment.schema.json`.
- Typed state and safe reports: `crates/bench-core`.
- CLI schemas: `crates/bench-cli/schemas`; developer requests:
  `experiments/rune/schemas/inspector.json`.

See [architecture](architecture.md) for clickable paths and
[REA mapping](rea-maps.md) for Dotdog's product role. The secondary repository
map is not a firmware investigation.

## Verify with existing tools

```sh
bun install --cwd web --frozen-lockfile --ignore-scripts
bun run --cwd web build
bun test --cwd web
cargo fmt --all --check
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
```

The native app forwards Rune's headless arguments:

```sh
cargo run --locked -p reforge-bench-desktop -- --replay --tree /tmp/reforge-tree.json
cargo run --locked -p reforge-bench-desktop -- --replay --snap /tmp/reforge.png --size 1000x760
```

Headless replay verifies the native renderer, not embedded webviews or physical hardware.
Use [the running-app inspector](desktop.md#inspector) and inspect changed embedded
views too. The socket inspector is macOS-only; normal builds do not start it.
Windows packaged-app checks use `scripts/check-windows.ps1`.

For live inspection, read `inspect --schema` before sending commands; it works
without a running app. Start with `inspect snapshot` and `inspect diagnostics`.
Use `inspect web.snapshot` only after opening an embedded view. Results are JSON
on stdout; failures return a nonzero exit code and an explanation on stderr.
Treat a failed or unavailable query as unknown, not as a passing check.

The CLI embeds `experiments/rune/schemas/inspector.json` as its single request
schema. Requests have a 16 KiB wire limit; responses have a 2 MiB limit. This is
an opt-in developer interface, not a bundled AI service or a hardware controller.

Keep keys stable and scoped. Bound growing scroll areas; use `min_h(0.0)` through
nested flex columns. Prefer one scroll owner. Preserve keyboard navigation,
unknown values and simulated/recorded/live distinctions.

## Changes and evidence

Trace actions to real handlers and data sources. Preserve producer identities,
coverage and limitations. A path means a supplied graph relationship, not a
measured causal chain. Never invent captured data, analysis results, connected
adapters or passing checks. Keep generated maps, local logs and private user
data out of commits.

Rune's [README](https://github.com/anomalyco/rune) and
[core docs](https://github.com/anomalyco/rune/blob/dev/crates/rune/README.md)
informed this workflow. Reforge pins its own tested revision; current upstream
examples may use newer APIs. Follow Reforge's contribution policy.
