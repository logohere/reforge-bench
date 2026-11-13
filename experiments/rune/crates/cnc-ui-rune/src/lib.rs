#[cfg(unix)]
mod agent_debug;
#[path = "../../../../../desktop/src/session.rs"]
mod backend_session;
mod catalog;
#[path = "../../../../../desktop/src/client_diagnostics.rs"]
mod client_diagnostics;
mod controller;
mod diagnostics;
mod dotdog;
mod embedded;
mod graph;
mod graph_model;
mod hardware;
mod hardware_host;
mod host;
mod inspector;
mod library;
mod local_http;
mod preferences;
mod provider;
mod theme;
mod timeline;
mod timeline_model;
mod workspace;

use std::io::Read;
use std::path::Path;

use rea_ui_contract::{BenchSnapshot, SourceKind};
use rune::{App, AppSpec, Chrome, Cx, El, ScrollSpec, col, row, text};
use rune_kit::{Button, ButtonVariant};
use sha2::{Digest, Sha256};
use theme::{color, font, size};

const IDLE: &[u8] = include_bytes!("../../../tests/fixtures/machine-idle.json");
const DISCONNECTED: &[u8] = include_bytes!("../../../tests/fixtures/instrument-disconnected.json");
const MAX_FIXTURE_BYTES: usize = 1_048_576;

#[cfg(unix)]
pub use agent_debug::inspect;

pub struct State {
    pub snapshot: BenchSnapshot,
    pub name: String,
    pub sha256: String,
    replay: Option<usize>,
    host: host::Connection,
    page: host::Page,
    ui: workspace::Workspace,
    preferences: Option<preferences::Preferences>,
}

pub(crate) fn sha256(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut hex = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(&mut hex, "{byte:02x}").expect("writing to a String cannot fail");
    }
    hex
}

impl State {
    pub fn from_bytes(name: &str, bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_FIXTURE_BYTES {
            return Err("Snapshot exceeds the 1 MiB limit".into());
        }
        let snapshot: BenchSnapshot = serde_json::from_slice(bytes)
            .map_err(|error| format!("Invalid snapshot JSON: {error}"))?;
        snapshot.validate().map_err(str::to_owned)?;
        if snapshot
            .instruments
            .iter()
            .any(|item| item.device_id.trim().is_empty())
        {
            return Err("Instrument identity is empty".into());
        }
        Ok(Self {
            snapshot,
            name: name.into(),
            sha256: sha256(bytes),
            ui: workspace::Workspace::default(),
            preferences: None,
            host: host::Connection::default(),
            page: host::Page::Replay,
            replay: if bytes == IDLE {
                Some(0)
            } else if bytes == DISCONNECTED {
                Some(1)
            } else {
                None
            },
        })
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        let file =
            std::fs::File::open(path).map_err(|error| format!("Cannot open snapshot: {error}"))?;
        let mut bytes = Vec::new();
        file.take((MAX_FIXTURE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| format!("Cannot read snapshot: {error}"))?;
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Imported snapshot");
        Self::from_bytes(name, &bytes)
    }

    pub fn idle() -> Self {
        Self::from_bytes("machine-idle.json", IDLE).expect("embedded idle snapshot must validate")
    }

    pub fn disconnected() -> Self {
        Self::from_bytes("instrument-disconnected.json", DISCONNECTED)
            .expect("embedded disconnected snapshot must validate")
    }

    fn select_system(&mut self, id: String) {
        let destination = if self.page == host::Page::Workspace {
            self.ui.embedded.destination().map(str::to_owned)
        } else {
            None
        };
        self.host.selected = Some(id);
        self.ui.library.files = false;
        self.ui.library.evidence = false;
        self.ui.library.provenance = false;
        let step = self.ui.inspector.step;
        self.ui.inspector = inspector::Inspector::default();
        self.ui.inspector.step = step;
        controller::preload(self, false);
        if self.page == host::Page::Rea {
            workspace::open_existing(self, "rea");
        }
        if self.page == host::Page::Timeline {
            timeline::refresh(self);
        }
        if let Some(destination) = destination {
            workspace::open_existing(self, &destination);
        }
        self.persist_preferences();
    }

    fn persist_preferences(&mut self) {
        if let Some(preferences) = &mut self.preferences
            && let Err(error) = preferences.save(self.host.selected.as_deref(), self.ui.collapsed)
        {
            self.ui.notice = Some(error);
        }
    }

    fn select_replay(&mut self, disconnected: bool) {
        let next = if disconnected {
            Self::disconnected()
        } else {
            Self::idle()
        };
        self.snapshot = next.snapshot;
        self.name = next.name;
        self.sha256 = next.sha256;
        self.replay = next.replay;
        self.page = host::Page::Replay;
    }
}

fn label(value: impl Into<String>) -> El {
    text(value)
        .font(font("--rb-text-body", 400.0))
        .color(color("--rb-color-text"))
        .wrap()
}

fn secondary(value: impl Into<String>) -> El {
    text(value)
        .font(font("--rb-text-secondary", 400.0))
        .color(color("--rb-color-secondary"))
        .wrap()
}

fn measurement(key: &str, name: &str, value: Option<f64>, unit: &str) -> El {
    let value = value.map_or_else(|| "Unknown".into(), |value| format!("{value:.2} {unit}"));
    row()
        .w_full()
        .justify_between()
        .py(size("--rb-space-4"))
        .border_sides([0.0, 0.0, 1.0, 0.0], color("--rb-color-divider"))
        .child(secondary(name))
        .child(
            label(value)
                .key(key)
                .font(font("--rb-text-body", 500.0).mono()),
        )
}

pub fn view(state: &State, cx: &mut Cx<State>) -> El {
    if state.page != host::Page::Replay {
        return workspace::view(state, cx);
    }
    let kit = theme::kit();
    let machine = &state.snapshot.machine;
    let source = match state.snapshot.source {
        SourceKind::Simulated => "Simulated snapshot",
        SourceKind::Recorded => "Recorded snapshot",
        SourceKind::Live => "Imported live snapshot · not authenticated",
    };
    let choices = row()
        .gap(size("--rb-space-3"))
        .center_y()
        .child(secondary("Replay"))
        .child(
            Button::new("Machine idle")
                .variant(if state.replay == Some(0) {
                    ButtonVariant::Contrast
                } else {
                    ButtonVariant::Neutral
                })
                .on_press(|state: &mut State| state.select_replay(false))
                .build(&kit)
                .key("fixture-idle")
                .h(size("--rb-control-primary")),
        )
        .child(
            Button::new("Instrument disconnected")
                .variant(if state.replay == Some(1) {
                    ButtonVariant::Contrast
                } else {
                    ButtonVariant::Neutral
                })
                .on_press(|state: &mut State| state.select_replay(true))
                .build(&kit)
                .key("fixture-disconnected")
                .h(size("--rb-control-primary")),
        )
        .child(
            Button::new("Bench")
                .on_press(|state: &mut State| {
                    state.page = host::Page::Home;
                    if state.host.snapshot.is_none() {
                        state.host.refresh();
                    }
                })
                .build(&kit)
                .key("nav-bench")
                .h(size("--rb-control-primary")),
        );
    let machine_panel = col()
        .w_full()
        .gap(size("--rb-space-2"))
        .child(label("Machine").font(font("--rb-text-body", 500.0)))
        .child(measurement("x-value", "X position", machine.x_mm, "mm"))
        .child(measurement("z-value", "Z position", machine.z_mm, "mm"))
        .child(measurement(
            "spindle-value",
            "Spindle speed",
            machine.spindle_rpm,
            "rpm",
        ))
        .child(
            row()
                .w_full()
                .justify_between()
                .py(size("--rb-space-4"))
                .child(secondary("Program state"))
                .child(label(&machine.program_state).key("program-state")),
        );
    let mut instruments = col()
        .w_full()
        .gap(size("--rb-space-4"))
        .child(label("Instruments").font(font("--rb-text-body", 500.0)));
    if state.snapshot.instruments.is_empty() {
        instruments = instruments
            .child(secondary("No instrument samples in this snapshot.").key("instrument-empty"));
    }
    for (index, item) in state.snapshot.instruments.iter().enumerate() {
        let status = if item.stale {
            "Stale · no current sample"
        } else {
            "Retained sample · not a live connection"
        };
        let captured = item.captured_at_ms.map_or_else(
            || "Capture time unknown".into(),
            |value| format!("Captured at {value} ms since Unix epoch"),
        );
        instruments = instruments.child(
            col()
                .gap(size("--rb-space-2"))
                .child(label(&item.device_id))
                .child(
                    label(status)
                        .key(format!("instrument-status-{index}"))
                        .color(color("--rb-color-warning")),
                )
                .child(secondary(captured)),
        );
    }
    let mut content = col()
        .w_full()
        .gap(size("--rb-gap"))
        .child(
            text("Machine status")
                .font(font("--rb-text-title", 500.0))
                .color(color("--rb-color-text")),
        )
        .child(secondary(
            "Inspect a saved snapshot. No devices are connected or controlled.",
        ))
        .child(choices)
        .child(
            col()
                .w_full()
                .padding(size("--rb-space-5"))
                .gap(size("--rb-space-2"))
                .radius(size("--rb-radius-panel"))
                .fill(color("--rb-color-info-bg"))
                .child(
                    label(source)
                        .key("source-kind")
                        .color(color("--rb-color-info")),
                )
                .child(
                    secondary(format!(
                        "{} · schema {} · tick {}",
                        state.name, state.snapshot.schema_version, state.snapshot.tick
                    ))
                    .key("snapshot-identity"),
                ),
        )
        .child(machine_panel)
        .child(instruments);
    for (index, warning) in state.snapshot.warnings.iter().enumerate() {
        content = content.child(
            label(warning)
                .key(format!("warning-{index}"))
                .color(color("--rb-color-warning")),
        );
    }
    content = content.child(
        secondary(format!("Snapshot SHA-256  {}", state.sha256))
            .key("snapshot-hash")
            .font(font("--rb-text-caption", 400.0).mono())
            .wrap_anywhere(),
    );
    col()
        .w_full()
        .h_full()
        .fill(color("--rb-color-canvas"))
        .child(header())
        .child(
            col()
                .key("content")
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .padding(size("--rb-gutter"))
                .child(content),
        )
}

fn header() -> El {
    row()
        .h(size("--rb-shell-height"))
        .px(size("--rb-gutter"))
        .center_y()
        .justify_between()
        .fill(color("--rb-color-surface"))
        .child(label("REA Bench").font(font("--rb-text-body", 500.0)))
        .child(secondary("Rune · read-only"))
}

pub fn app(state: State) -> App<State> {
    let port = state.host.port;
    let window_size = if state.page == host::Page::Replay {
        (1000.0, 760.0)
    } else {
        (1440.0, 800.0)
    };
    let spec = AppSpec {
        min_size: Some((560.0, 480.0)),
        families: ("system-ui, Inter", "ui-monospace, Inter"),
        scenes: &[("disconnected", |state| *state = State::disconnected())],
        setup: Some(|state, waker| {
            state.ui.assets(waker.clone());
            state.host.waker = Some(waker);
            if state.page != host::Page::Replay {
                state.host.refresh();
            }
        }),
        on_key: Some(timeline::key),
        poll: Some(|state, clock, tx| {
            let changed = state.host.poll();
            let inspector_changed = state.ui.inspector.poll(state.host.selected.as_deref());
            let graph_changed = state.ui.graph.poll(state.host.selected.as_deref());
            let hardware_changed = hardware::poll(state, clock, tx);
            if changed {
                controller::preload(state, false);
                state.persist_preferences();
                if state.page == host::Page::Rea {
                    workspace::open_existing(state, "rea");
                }
                if state.page == host::Page::Timeline {
                    timeline::refresh(state);
                }
            }
            let timeline_changed = if state.page == host::Page::Timeline {
                state
                    .ui
                    .timeline
                    .poll(state.host.port, state.host.waker.clone(), clock, tx)
            } else {
                state.ui.timeline.stop();
                false
            };
            if state.host.imported.take().is_some() {
                state.ui.library.clear();
                state.ui.library.import = None;
                workspace::open_existing(state, "rea");
            }
            let controller_changed = controller::poll(state);
            changed
                || inspector_changed
                || graph_changed
                || timeline_changed
                || hardware_changed
                || controller_changed
        }),
        ..AppSpec::new("Reforge Bench", window_size, State::idle, view)
    };
    let app = App::new(spec)
        .init(move || state)
        .chrome(Chrome {
            full_size_content: false,
            ..Chrome::default()
        })
        .tab_visits_controls(true)
        .on_visibility(|state, visible, _| {
            state.ui.hardware.set_visible(visible);
            if !visible {
                state.ui.timeline.stop();
            }
        })
        .standard_menus();
    embedded::attach(workspace::menus(app), port)
}

pub fn set_app_icon() -> Result<(), String> {
    rune::set_app_icon(include_bytes!("../../../../../desktop/assets/app-icon.png").to_vec())
}

pub fn run(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    diagnostics::install_panic_hook();
    let mut args = args.into_iter();
    let mut rune_args = Vec::new();
    let mut state = State::idle();
    state.page = host::Page::Home;
    let mut loaded = false;
    let port = 0;
    while let Some(arg) = args.next() {
        if arg == "--backend-probe" {
            let started = std::time::Instant::now();
            let library = host::read(0, "/api/library")?;
            println!(
                "{}",
                serde_json::json!({"transport":"in_process","ready_ms":started.elapsed().as_millis(),"health":host::read(0,"/api/desktop/health")?,"library_cases":library["cases"].as_array().map(Vec::len)})
            );
            return Ok(());
        } else if arg == "--replay" {
            state.page = host::Page::Replay;
        } else if arg == "--library" {
            state.page = host::Page::Library;
        } else if arg == "--rea" {
            state.page = host::Page::Rea;
        } else if arg == "--timeline" {
            state.page = host::Page::Timeline;
        } else if arg == "--hardware" {
            state.page = host::Page::Hardware;
            state.ui.hardware.set_visible(true);
        } else if arg == "--fixture" {
            if loaded {
                return Err("Use --fixture only once".into());
            }
            let path = args.next().ok_or("--fixture requires a JSON file path")?;
            state = State::from_file(Path::new(&path))?;
            loaded = true;
        } else {
            rune_args.push(arg);
        }
    }
    state.host.port = port;
    let headless = rune::tooling::Args::parse_from(rune_args.clone()).headless();
    if state.page != host::Page::Replay
        && !headless
        && let Some(path) = preferences::path()
    {
        match preferences::Preferences::load(path) {
            Ok((preferences, (selected, collapsed))) => {
                state.host.selected = selected;
                state.ui.collapsed = collapsed;
                state.preferences = Some(preferences);
            }
            Err(error) => state.ui.notice = Some(error),
        }
    }

    app(state).args(rune_args).try_run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Key, KeyEvent, Mods, Runtime};

    fn runtime(state: State, size: (f32, f32)) -> Runtime<State> {
        let mut runtime = app(state).headless_runtime(
            size,
            Env {
                system_dark: true,
                scale: 2.0,
            },
        );
        runtime.frame(0.0);
        runtime
    }

    fn value(runtime: &Runtime<State>, key: &str) -> String {
        runtime
            .elements()
            .into_iter()
            .find(|element| element.key.as_deref() == Some(key))
            .unwrap()
            .text
            .unwrap()
    }

    #[test]
    fn snapshots_preserve_unknowns_and_exact_source_bytes() {
        assert_eq!(
            sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let state = State::disconnected();
        assert_eq!(state.snapshot.machine.x_mm, None);
        assert_eq!(state.sha256, sha256(DISCONNECTED));
        let runtime = runtime(state, (1000.0, 760.0));
        assert_eq!(value(&runtime, "x-value"), "Unknown");
        assert_eq!(value(&runtime, "spindle-value"), "Unknown");
        assert!(value(&runtime, "instrument-status-0").starts_with("Stale"));
        assert_eq!(value(&runtime, "source-kind"), "Simulated snapshot");
    }

    #[test]
    fn pointer_and_keyboard_can_switch_and_recover_replays() {
        let mut runtime = runtime(State::idle(), (1000.0, 760.0));
        let (x, y) = runtime.find("fixture-disconnected").unwrap();
        runtime.pointer_down(x, y);
        runtime.pointer_up(x, y);
        runtime.frame(0.0);
        assert_eq!(value(&runtime, "x-value"), "Unknown");
        runtime.key(KeyEvent {
            key: Key::Tab,
            mods: Mods {
                shift: true,
                ..Mods::default()
            },
        });
        runtime.frame(0.0);
        runtime.key(KeyEvent {
            key: Key::Text(" ".into()),
            mods: Mods::default(),
        });
        runtime.frame(0.0);
        assert_eq!(runtime.state.name, "machine-idle.json");
        assert_eq!(value(&runtime, "x-value"), "12.50 mm");
    }

    #[test]
    fn rejects_malformed_unsupported_and_oversized_snapshots() {
        assert!(State::from_bytes("bad", b"{").is_err());
        assert!(State::from_bytes("large", &vec![b' '; MAX_FIXTURE_BYTES + 1]).is_err());
        let mut snapshot: serde_json::Value = serde_json::from_slice(IDLE).unwrap();
        snapshot["schema_version"] = 99.into();
        assert!(State::from_bytes("future", &serde_json::to_vec(&snapshot).unwrap()).is_err());
        snapshot["schema_version"] = 1.into();
        snapshot["machine"]["spindle_rpm"] = (-1).into();
        assert!(State::from_bytes("invalid", &serde_json::to_vec(&snapshot).unwrap()).is_err());
    }

    #[test]
    fn replay_selection_preserves_the_host_session() {
        let mut state = State::idle();
        state.host.port = 12345;
        state.host.error = Some("Host unavailable".into());
        state.select_replay(true);
        assert_eq!(state.host.port, 12345);
        assert_eq!(state.host.error.as_deref(), Some("Host unavailable"));
        assert_eq!(state.name, "instrument-disconnected.json");
    }

    #[test]
    fn completed_import_opens_native_evidence_instead_of_lab() {
        let mut runtime = runtime(State::idle(), (1000.0, 760.0));
        runtime.state.host.imported = Some("0".repeat(64));
        runtime.state.ui.library.import = Some("example.json".into());
        runtime.frame(0.0);
        assert!(matches!(runtime.state.page, host::Page::Rea));
        assert!(runtime.state.ui.library.import.is_none());
        assert!(runtime.state.host.imported.is_none());
    }

    #[test]
    fn replay_actions_remain_visible_at_supported_widths() {
        for viewport in [(1000.0, 760.0), (560.0, 480.0)] {
            let runtime = runtime(State::disconnected(), viewport);
            for key in [
                "fixture-idle",
                "fixture-disconnected",
                "x-value",
                "source-kind",
            ] {
                let element = runtime
                    .elements()
                    .into_iter()
                    .find(|element| element.key.as_deref() == Some(key))
                    .unwrap();
                let [x, y, width, height] = element.rect;
                assert!(
                    x >= 0.0
                        && y >= 0.0
                        && x + width <= viewport.0 + 1.0
                        && y + height <= viewport.1 + 1.0,
                    "{key}: {:?}",
                    element.rect
                );
            }
        }
    }
}
