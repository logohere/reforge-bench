# Reforge Bench

A native Rust/Rune workbench with typed state, hardware inspection, local
diagnostics and one recorded hardware/REA example. The desktop uses a native
workspace shell, with hardware views running inside the app using embedded
WebKit. It does not open a browser or require a local server.

**Status: `0.1.0-alpha.1`, unreleased source preview.** No live acquisition
adapter, OEM firmware or private firmware-analysis engine is bundled. The
desktop preview is macOS-only; no public binary release is available.

[Architecture](docs/architecture.md) · [Desktop](docs/desktop.md) ·
[REA maps](docs/rea-maps.md) · [AI guide](docs/ai.md) · [Contributing](CONTRIBUTING.md)

For your own artifacts, equipment and captures, start with
[Use Reforge for your project](docs/your-project.md). Included hardware is an
example preset; editable equipment records do not imply pin or driver compatibility.

## macOS app

Build on macOS with Xcode Command Line Tools, Python 3, Rustup and Bun 1.3.14.
Cargo uses the pinned Rust 1.100.0 toolchain. From the repository root:

```sh
python3 scripts/package-macos.py
open "dist/Reforge Bench.app"
```

No Apple developer account is needed to build locally. The packager creates a
locally ad-hoc-signed app, not a notarized distribution.
See [architecture-specific builds](docs/desktop.md) and the
[release checklist](docs/publishing.md) before distributing a build.

| Platform | Current verification |
| --- | --- |
| Apple silicon macOS | Native build, tests and rendered UI checked |
| Intel macOS | Desktop compile-checked, including developer features; execution not yet verified |

CI defines separate macOS architecture jobs. Hosted execution remains unverified;
the table describes local checks, not CI results. The macOS
bundle declares macOS 13 as its minimum; execution on that minimum is not yet
qualified.

Library imports your own files into the app's local storage. Hardware provides
2D/3D reference models, pin inspection and wiring guides. Lab uses the independent
pulse fixture; its saved RTL simulation is explicitly separate from measured
hardware. The recorded scope example contains summaries, not raw captures.

Tools → Diagnostics retains native/client failures locally and exports only
allowlisted metadata. For a developer build with WebKit tools and the local
agent inspector, use `--debug --agent-debug`; normal builds do not enable that
endpoint. See [desktop development](docs/desktop.md).

## Use it with your agent

Start your agent with [AGENTS.md](AGENTS.md) and [the AI guide](docs/ai.md).
Use existing native/headless inspection tools, then verify changed embedded views
in the running app. Preserve the UI and evidence boundaries rather than inventing
device data to make a check pass.

## Evidence mapping

[Dotdog](https://github.com/specdog/dotdog) prepares and queries REA graphs.
Import saved JSON exports into the native REA Tool to inspect relationships,
identities and limits. See [REA mapping](docs/rea-maps.md). Repository navigation
is a separate, secondary use of Dotdog.

## Try it

With Rustup installed, Cargo uses the toolchain pinned in `rust-toolchain.toml`.
From the repository root:

```sh
cargo run --locked -- demo
cargo run --locked -- validate-snapshot crates/bench-cli/examples/hardware-rea/snapshot.json --json
cargo run --locked -- schema snapshot
cargo test --locked --workspace
```

The first command prints selected recorded findings. The second validates the
same case's example state envelope; unknown readings remain null. Neither opens a
connection. The CLI needs no Node installation or hardware setup.

## Reuse the core

In another Rust project's `Cargo.toml`, point to your copy of the core:

```toml
[dependencies]
reforge-bench-core = { path = "../reforge-bench/crates/bench-core" }
serde_json = "1"
```

```rust
use reforge_bench_core::diagnostics::Diagnostics;

fn main() -> Result<(), serde_json::Error> {
    let mut diagnostics = Diagnostics::default();
    diagnostics.record_failure("import", "Selected file is unavailable");
    let report = diagnostics.shareable_report();
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
```

Local failures are typed Rust values. Only allowlisted metadata goes into the
serializable report; private messages are excluded. `BenchSnapshot` similarly
separates typed state from JSON interchange. See [diagnostics](docs/diagnostics.md).

## Included example

The [hardware/REA example](crates/bench-cli/examples/hardware-rea/README.md) is a curated summary
of a real scope investigation, selected measured results and the Rust bridge's
role. It is saved demo information, not a live connection or fabricated signal.
No device, remote login or external analysis installation is needed to read it.

## Local CLI

The CLI can validate saved snapshots and export metadata-only diagnostic
summaries. It reads local files; it does not control hardware or a running app.
See [CLI commands](docs/cli.md) and [diagnostics](docs/diagnostics.md).
The desktop's separate [developer inspector CLI](docs/desktop.md#inspector)
can query an explicitly enabled developer build; it is off in normal builds.

## Contributing

Dotdog 1.0.0 is required contributor tooling, pinned in `package-lock.json`;
it is not a Rust runtime dependency. See [contributing](CONTRIBUTING.md) for
setup, formatting, linting and distribution checks.
The [repository map](CONTRIBUTING.md#repository-map) supports targeted queries
for contracts and their implementation, schema and test links.

## Release boundary

Only reusable bench code, documented data formats and the reviewed example
belong here. Original firmware, raw captures, device serial numbers and user
project files remain outside the repository. No proprietary analysis engine or
private service configuration is bundled. Selected factual summaries are distinct
from permission to redistribute the underlying software or assets.

Retain source provenance as simulated, recorded or live. Users supply their own
analysis tools and hardware configuration; do not include remote access, account
configuration, private analysis examples or personal notes.

## Credits

Built with [Rune and Rune Kit](https://github.com/anomalyco/rune), created by
Anomaly and licensed under MIT, © 2026 Anomaly. Their native UI and agent-oriented
tooling are core foundations of Reforge. The full [Rune MIT notice](experiments/rune/licenses/Rune-MIT.txt)
is retained in the source and macOS package.

[Dotdog](https://github.com/specdog/dotdog) provides evidence-graph tooling.
Carbon, Three.js, uPlot and the other components and hardware assets retain their
own attribution and licenses; see [third-party notices](THIRD_PARTY_NOTICES.md).
Reforge is an independent project; credits do not imply upstream endorsement.

## License

See [LICENSE](LICENSE) for this project's MIT license, covering its source and
original example content. Dependencies retain their own licenses and notices.
See [third-party notices](THIRD_PARTY_NOTICES.md).

See [contributing](CONTRIBUTING.md), [security](SECURITY.md),
[changelog](CHANGELOG.md) and [publishing](docs/publishing.md).
