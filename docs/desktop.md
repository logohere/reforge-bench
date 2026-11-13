# Desktop development

The application shell is Rust/Rune. The retained hardware framework uses Three,
Carbon web components and an embedded WebKit view. Both share the app design
tokens and icons. Packaged resources use an app-owned custom URL scheme; no
TCP port, browser handoff or private backend is needed.

## Build and verify

```sh
bun install --cwd web --frozen-lockfile --ignore-scripts
bun run --cwd web build
bun test --cwd web
cargo test --locked --workspace
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
python3 scripts/package-macos.py --debug --agent-debug
```

The macOS bundle has its own identifier, app icon, resources and data directory.
Imports, preferences and developer sockets never use another bench installation.
The packager signs locally with an ad-hoc signature; it does not notarize or
publish the app.

The default build uses the host Mac's architecture. For a specific architecture,
install the matching Rust target and pass it to the packager:

```sh
rustup target add x86_64-apple-darwin
python3 scripts/package-macos.py --target x86_64-apple-darwin
```

Use `aarch64-apple-darwin` for Apple silicon. Each command replaces the one app
in `dist`; quit that app before rebuilding. The build receipt records the target.
Cross-compilation does not verify execution on the other architecture. Hardware
and acquisition support still depends on the user's chosen equipment and tools.

## Inspector

Run against an explicitly enabled developer build:

```sh
"dist/Reforge Bench.app/Contents/MacOS/reforge-bench-desktop" inspect --schema
"dist/Reforge Bench.app/Contents/MacOS/reforge-bench-desktop" inspect snapshot
"dist/Reforge Bench.app/Contents/MacOS/reforge-bench-desktop" inspect diagnostics
"dist/Reforge Bench.app/Contents/MacOS/reforge-bench-desktop" inspect web.snapshot
"dist/Reforge Bench.app/Contents/MacOS/reforge-bench-desktop" inspect web.inspector
```

The inspector uses a private local Unix socket, bounded newline-delimited JSON
and a closed method list. `web.evaluate EXPRESSION` executes developer-provided
JavaScript in the embedded view. It is not a hardware-control protocol. Inspect
responses can contain private local details; use the metadata-only diagnostic
export when sharing a report.

## Data boundaries

The default library contains one real scope investigation summary. User imports
are content-addressed local files, not uploaded firmware. Import does not claim
analysis or emulation. Acquisition controls remain disconnected without a user
adapter; there are no fabricated hardware frames or bundled access credentials.

Hardware models and factual reference pin mappings are retained. Vendor firmware,
decompiled routines, private analysis providers, raw private captures and unit
serials are not distributed. The Pico CAD and Digilent pin reference retain their
upstream licenses. OEM screenshots and unlicensed vendor photography are omitted.
