# Contributing

Use a focused branch and pull request against `develop`. Preserve existing
behavior and component ownership. Keep user configuration and data separate from
reusable implementations.

## Checks

```sh
npm ci --ignore-scripts
npm run model:check
bun install --cwd web --frozen-lockfile --ignore-scripts
bun run --cwd web build
bun test --cwd web
python3 scripts/check-distribution.py
cargo fmt --check
cargo test --locked --workspace
CARGO_BUILD_WARNINGS=deny cargo clippy --locked --workspace --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps
```

On Windows, use PowerShell `$env:NAME="value"` syntax for environment variables.
See [desktop development](docs/desktop.md#windows) for MSVC/WebView2 prerequisites
and the packaged-app smoke check. CI checks Windows x64 and both macOS architectures.

Use Rustfmt and Clippy, not another formatting framework. Keep Rust APIs readable
and document units, provenance, failure behavior and authority boundaries. Verify
behavior before claiming an optimization; do not trade correctness for a smaller
diff or an unsupported performance claim.

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
where relevant. Borrow data when ownership is unnecessary; keep justified clones.
Measure representative release workloads before and after performance changes.
Keep existing privacy and input limits intact. Workspace lints reject debug
macros and unfinished-code placeholders; do not enable every restriction lint.
Use structs and enums for internal state; keep JSON at interchange boundaries.

Keep tests that establish behavior or prevent regressions. Avoid tests that only
mirror the implementation. Report checks actually run and unresolved gaps; a
build is not rendered UI, hardware or security acceptance.

## REA mapping

Dotdog's product role is preparing and querying firmware/hardware evidence
graphs. See [REA maps](docs/rea-maps.md). Do not treat a repo navigation graph as
an investigation or execution trace.

## Repository map

```sh
npm run model:refresh
npm run model:query -- BenchSnapshot
npm run model:trace -- DiagnosticReport
npm run model:path -- Command DiagnosticReport
```

Edit contracts and file bindings in `specs/bench/data-model.dog`, not generated
graphs. The npm refresh/check commands copy that reviewed file into Dotdog's
local semantic layer before rebuilding. Queries then include the authored Rust
contracts and links to source, schemas and tests. Preparation checks that mapped
files exist; CI checks two required paths.
Generated `.doghouse/` and `.dag` files remain ignored. This is an authored map,
not automatic Rust call-graph extraction or proof that tests cover every behavior.

See the [Dotdog guide](https://specdog.github.io/handbook) for the distinction
between intended contracts, observed code and runtime verification.

## Data and dependencies

Do not include credentials, serial numbers, machine bindings, raw captures,
firmware, decompiled code or private analysis procedures. Use the single reviewed
example for demonstrations. New assets need source, license and redistribution
permission. Preserve required third-party attribution.

Dotdog is required tooling. Commit human-owned specs, not generated `.dag` files.
Its graph checks do not establish Rust runtime coverage or hardware correctness.
Keep dependency lockfiles and third-party notices current. Dependency updates go
through PRs, not automatic merges. Incompatible stored formats need an explicit
version/migration decision.

The initial version remains `0.1.0-alpha.1` until released; do not bump it for each
unshipped commit. Record user-visible changes in [CHANGELOG](CHANGELOG.md).

See [SECURITY](SECURITY.md) for private reports and
[publishing](docs/publishing.md) for distribution checks.
