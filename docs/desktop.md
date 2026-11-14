# Desktop development

The application shell is Rust/Rune. The retained hardware framework uses Three,
Carbon web components and an embedded WebKit (macOS) or WebView2 (Windows) view. Both share the app design
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

## Windows

The same desktop crate builds for `x86_64-pc-windows-msvc`. Its historical
`experiments/rune/apps/macos` path is not a platform restriction. Windows ARM64,
32-bit Windows and GNU-toolchain packages are not currently qualified.

Use Windows 11 x64 with:

- Visual Studio 2022 or newer C++ Build Tools, with **Desktop development with C++**
  and a Windows SDK. Rustup's Windows installer can help install these tools.
- Rustup with the `x86_64-pc-windows-msvc` host toolchain. Cargo selects the pinned Rust version.
- Python 3 and Bun 1.3.14 available as `python` and `bun` on PATH.
- The [WebView2 Evergreen Runtime](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution).
  It is a runtime prerequisite, not bundled by the packager.

```powershell
python scripts/package-windows.py
& ".\dist\Reforge Bench\reforge-bench-desktop.exe"
```

This builds the native Rune shell and the existing embedded hardware views,
then writes `dist/Reforge Bench` and a versioned Windows x64 ZIP. Extract and
keep the entire folder together. Run it from any working directory; packaged
resources are resolved beside the executable. Rebuilding replaces the package;
close the app first. `--debug` uses a debug Rust build. `--agent-debug` enables
WebView2 developer tools, not the macOS-only socket inspector.

Library imports, preferences and the isolated WebView2 profile live under
`%LOCALAPPDATA%\dev.reforge.bench`, not beside the executable or in the source
checkout. Source references open with the Windows default browser, and artifact
import uses Ctrl+O. Release builds do not open a console window; fatal startup
errors appear in a native dialog.

Local packages are unsigned and have no installer. Windows may warn before
running an unsigned app. Code signing, clean-machine runtime qualification and
an approved release remain separate distribution gates. CI uses Windows Server
2025 x64; it does not qualify every Windows version or hardware adapter.

The Windows smoke check launches the packaged app outside the checkout, invokes
the native hardware command through UI Automation, checks 2D/3D resources and the
in-process API, and captures embedded-view screenshots. Its temporary WebView2
debugging port is test-only; normal builds do not start a debug endpoint.

## Inspector

The private Unix-socket inspector is currently macOS-only.

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
