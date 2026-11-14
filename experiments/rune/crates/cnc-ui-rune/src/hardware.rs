use std::sync::mpsc::{self, Receiver};

use rune::{Cx, El, FileDialogOptions, LineCap, ScrollSpec, Shape, Tx, canvas, col, row};
use rune_kit::{Button, ButtonVariant, Checkbox, Collapsible};
use serde_json::{Value, json};

use super::{State, hardware_host as api, host, label, secondary, theme, workspace};
use theme::{color, font, size};
use workspace::{badge, command};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum Source {
    #[default]
    Scope,
    Logic,
    Pico,
}

impl Source {
    fn id(self) -> &'static str {
        match self {
            Self::Scope => "scope",
            Self::Logic => "logic",
            Self::Pico => "pico",
        }
    }
    fn title(self) -> &'static str {
        match self {
            Self::Scope => "Hantek CH1",
            Self::Logic => "Logic CH0 / CH1",
            Self::Pico => "Pico GP2",
        }
    }
}

enum Reply {
    Status(Value),
    Action(Value),
    Exported,
}

#[derive(Default)]
pub struct Hardware {
    source: Source,
    captures: bool,
    trash: bool,
    status: Value,
    frame: Option<Value>,
    saved: Value,
    selected: Option<String>,
    setup: Option<Value>,
    connection_details: bool,
    details: bool,
    wiring: bool,
    live: bool,
    visible: bool,
    zoom: f64,
    cursor: Option<f64>,
    error: Option<String>,
    message: Option<String>,
    confirm: Option<bool>,
    receiver: Option<Receiver<Result<Reply, String>>>,
    request_context: (Source, bool),
    due: f64,
}

impl Hardware {
    fn current_frame(&self) -> bool {
        self.frame.as_ref().is_some_and(|frame| {
            self.status
                .get("frame")
                .or_else(|| self.status.get("capture"))
                .is_some_and(|current| {
                    current["sha256"].is_string() && current["sha256"] == frame["sha256"]
                })
        })
    }

    fn start(
        &mut self,
        port: u16,
        wake: Option<rune::Waker>,
        work: impl FnOnce(u16) -> Result<Reply, String> + Send + 'static,
    ) {
        if self.receiver.is_some() {
            return;
        }
        self.error = None;
        self.request_context = (self.source, self.captures);
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        std::thread::spawn(move || {
            let _ = sender.send(work(port));
            if let Some(wake) = wake {
                wake();
            }
        });
    }

    fn refresh(&mut self, port: u16, wake: Option<rune::Waker>) {
        let path = if self.captures {
            "/api/captures"
        } else {
            match self.source {
                Source::Scope => "/api/scope/status",
                Source::Logic => "/api/logic/status",
                Source::Pico => "/api/pico/status",
            }
        };
        self.start(port, wake, move |port| {
            api::request(port, path, None).map(Reply::Status)
        });
    }

    fn accept(&mut self, reply: Reply) -> Result<(), String> {
        if !matches!(reply, Reply::Exported) && self.request_context != (self.source, self.captures)
        {
            self.due = 0.0;
            return Ok(());
        }
        match reply {
            Reply::Status(value) => {
                if self.captures {
                    self.saved = value;
                } else {
                    if let Some(frame) = value
                        .get("frame")
                        .or_else(|| value.get("capture"))
                        .filter(|frame| frame.is_object())
                    {
                        api::validate_frame(frame)?;
                        self.frame = Some(frame.clone());
                    }
                    self.status = value;
                }
            }
            Reply::Action(value) => {
                if value["configured"] == false {
                    self.message = Some(
                        value["message"]
                            .as_str()
                            .unwrap_or("No acquisition adapter configured")
                            .into(),
                    );
                    return Ok(());
                }
                if value["handoff"].is_string() {
                    self.setup = Some(value);
                } else if let Some(frame) = value
                    .get("frame")
                    .or_else(|| value.get("capture"))
                    .filter(|frame| frame.is_object())
                {
                    api::validate_frame(frame)?;
                    self.frame = Some(frame.clone());
                    if let Some(id) = value["entry"]["id"].as_str() {
                        self.selected = Some(id.into());
                    }
                    self.message = Some(
                        if self.captures {
                            "Saved replay loaded"
                        } else {
                            "Capture received"
                        }
                        .into(),
                    );
                } else if value["detected"].is_boolean() {
                    self.message = Some(
                        if value["detected"] == true {
                            "Analyzer detected · eight channels"
                        } else {
                            "Analyzer not detected. Check USB and close other capture apps."
                        }
                        .into(),
                    );
                } else {
                    self.message = Some(
                        if value["entry"].is_object() {
                            "Capture saved in app"
                        } else {
                            "Operation completed"
                        }
                        .into(),
                    );
                }
                self.due = 0.0;
            }
            Reply::Exported => {
                self.message = Some("Original capture exported".into());
            }
        }
        Ok(())
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        if !visible {
            self.live = false;
        }
    }
}

pub fn open(state: &mut State, captures: bool) {
    state.page = host::Page::Hardware;
    state.ui.navigation_open = false;
    state.ui.hardware.captures = captures;
    state.ui.hardware.visible = true;
    state.ui.hardware.due = 0.0;
    state
        .ui
        .hardware
        .refresh(state.host.port, state.host.waker.clone());
}

fn operate(state: &mut State, path: &'static str, body: Value) {
    state
        .ui
        .hardware
        .start(state.host.port, state.host.waker.clone(), move |port| {
            api::request(port, path, Some(&body)).map(Reply::Action)
        });
}

pub fn poll(state: &mut State, clock: f64, tx: &mut Tx) -> bool {
    let hw = &mut state.ui.hardware;
    let reply = hw
        .receiver
        .as_ref()
        .and_then(|receiver| receiver.try_recv().ok());
    let changed = reply.is_some();
    if let Some(reply) = reply {
        hw.receiver = None;
        if let Err(error) = reply.and_then(|reply| hw.accept(reply)) {
            hw.error = Some(error);
            hw.status = Value::Null;
            hw.live = false;
        }
    }
    if state.page == host::Page::Hardware && hw.visible {
        if hw.receiver.is_none() && clock >= hw.due {
            hw.due = clock + 1.0;
            if hw.live && !hw.captures && hw.source == Source::Logic {
                hw.start(state.host.port, state.host.waker.clone(), |port| {
                    api::request(
                        port,
                        "/api/logic/capture",
                        Some(&json!({"profile":"preview"})),
                    )
                    .map(Reply::Action)
                });
            } else {
                hw.refresh(state.host.port, state.host.waker.clone());
            }
        }
        tx.wake_at(hw.due.max(clock + 0.1));
    } else {
        hw.live = false;
    }
    changed
}

fn button(
    title: impl Into<String>,
    key: &str,
    primary: bool,
    disabled: bool,
    action: impl Fn(&mut State) + 'static,
) -> El {
    let title = title.into();
    let stop = title.starts_with("Stop ") || title == "Disconnect" || title == "Delete permanently";
    let selection = matches!(
        key,
        "hardware-acquire" | "hardware-captures" | "captures-library" | "captures-trash"
    );
    let selected = selection && primary;
    let button = command(title, primary)
        .variant(if stop {
            ButtonVariant::Danger
        } else if primary && !selection {
            ButtonVariant::Contrast
        } else {
            ButtonVariant::Outline
        })
        .disabled(disabled)
        .on_press(action)
        .build(&theme::kit())
        .key(key)
        .h(size("--rb-control-primary"));
    if selected {
        button.fill(color("--rb-color-selected"))
    } else {
        button
    }
}

fn export(state: &mut State, tx: &mut Tx, csv: bool) {
    let hw = &state.ui.hardware;
    let source = hw.source.id().to_owned();
    let id = if hw.captures {
        hw.selected.clone()
    } else {
        None
    };
    let extension = if csv {
        "csv"
    } else if hw
        .frame
        .as_ref()
        .is_some_and(|frame| frame["counts"].is_array())
    {
        "reascope"
    } else {
        "sr"
    };
    let file_name = format!("{}.{}", id.as_deref().unwrap_or(&source), extension);
    tx.save_file(
        FileDialogOptions {
            title: "Export original capture".into(),
            file_name: Some(file_name),
            ..Default::default()
        },
        move |state: &mut State, result, _| match result {
            Ok(Some(path)) => {
                let id = id.clone();
                let source = source.clone();
                state
                    .ui
                    .hardware
                    .start(state.host.port, state.host.waker.clone(), move |port| {
                        let bytes = api::export(port, id.as_deref(), &source, csv)?;
                        std::fs::write(path, bytes)
                            .map_err(|error| format!("Could not export capture: {error}"))?;
                        Ok(Reply::Exported)
                    });
            }
            Ok(None) => {}
            Err(error) => state.ui.hardware.error = Some(error.to_string()),
        },
    );
}

fn export_button(title: &str, key: &str, csv: bool, disabled: bool) -> El {
    Button::new(title)
        .variant(ButtonVariant::Outline)
        .disabled(disabled)
        .on_press_tx(move |state: &mut State, tx| export(state, tx, csv))
        .build(&theme::kit())
        .key(key)
        .h(44.0)
}

fn waveform(hw: &Hardware, width: f32) -> El {
    let Some(frame) = &hw.frame else {
        return col()
            .key("hardware-waveform-empty")
            .w_full()
            .min_h(240.0)
            .center_y()
            .padding(24.0)
            .child(secondary("No capture yet"))
            .fill(color("--rb-color-field"));
    };
    let time = frame["time"].as_array().expect("validated timebase");
    let full_span = time
        .last()
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        .max(1e-12);
    let zoom = if hw.zoom > 0.0 { hw.zoom } else { 1.0 };
    let center = hw.cursor.unwrap_or(0.5);
    let left = (center - zoom / 2.0).clamp(0.0, 1.0 - zoom);
    let start = left * full_span;
    let span = zoom * full_span;
    let digital = frame["channels"].is_array();
    let mut shape = Shape::new();
    for index in 0..=8 {
        let x = index as f32 * 100.0;
        shape = shape.line(
            [x, 0.0],
            [x, 240.0],
            0.5,
            LineCap::Butt,
            color("--rb-color-divider"),
        );
    }
    for index in 0..=4 {
        let y = index as f32 * 60.0;
        shape = shape.line(
            [0.0, y],
            [800.0, y],
            0.5,
            LineCap::Butt,
            color("--rb-color-divider"),
        );
    }
    let channels: Vec<&Value> = if digital {
        frame["channels"].as_array().unwrap().iter().collect()
    } else {
        vec![&frame["counts"]]
    };
    for (channel, values) in channels.into_iter().enumerate() {
        let mut points = Vec::new();
        for (stamp, value) in time.iter().zip(values.as_array().unwrap()) {
            let t = stamp.as_f64().unwrap();
            let x = ((t - start) / span * 800.0) as f32;
            let value = value.as_f64().unwrap() as f32;
            let y = if digital {
                100.0 + channel as f32 * 120.0 - value * 70.0
            } else {
                240.0 - (value + 128.0) / 255.0 * 240.0
            };
            if digital && let Some(previous) = points.last().copied() {
                let [_px, py]: [f32; 2] = previous;
                points.push([x, py]);
            }
            points.push([x, y]);
        }
        shape = shape.stroke(
            points,
            1.5,
            color(if channel == 0 {
                "--rb-color-warning"
            } else {
                "--rb-color-info"
            }),
        );
    }
    if let Some(cursor) = hw.cursor {
        let x = ((cursor - left) / zoom * 800.0) as f32;
        shape = shape.line(
            [x, 0.0],
            [x, 240.0],
            1.0,
            LineCap::Butt,
            color("--rb-color-text"),
        );
    }
    let axis = if digital {
        "CH0 / CH1 · logic state"
    } else {
        "CH1 · ADC counts (−128 … 127)"
    };
    col()
        .w_full()
        .gap(8.0)
        .child(secondary(axis))
        .child(
            canvas(shape.view_box(800.0, 240.0))
                .key("hardware-waveform")
                .role("img")
                .label(axis)
                .w_full()
                .h(240.0)
                .clip_rounded(0.0)
                .fill(color("--rb-color-field"))
                .on_drag(move |state: &mut State, drag, _| {
                    state.ui.hardware.cursor = Some(
                        (left
                            + (f64::from(drag.x) / f64::from(width.max(1.0))).clamp(0.0, 1.0)
                                * zoom)
                            .clamp(0.0, 1.0),
                    );
                }),
        )
        .child(
            row()
                .w_full()
                .justify_between()
                .child(secondary(format!("{:.3} ms", start * 1000.0)))
                .child(secondary(format!("{:.3} ms", (start + span) * 1000.0))),
        )
}

fn frame_details(hw: &Hardware) -> El {
    let Some(frame) = &hw.frame else { return col() };
    let mut details = frame.clone();
    for field in ["counts", "time", "channels", "sample_indices"] {
        details.as_object_mut().unwrap().remove(field);
    }
    let rate = frame["sample_rate_hz"].as_f64().unwrap_or(0.0);
    let clock = if rate >= 1_000_000.0 {
        format!("{:.2} MS/s", rate / 1_000_000.0)
    } else {
        format!("{rate:.0} samples/s")
    };
    let mut body = col()
        .w_full()
        .gap(12.0)
        .child(label(format!("{} samples · {clock}", frame["samples"])));
    if let Some(metrics) = frame["metrics"].as_array() {
        for metric in metrics {
            body = body.child(secondary(format!(
                "{} · {} rising edges · {} Hz",
                metric["name"].as_str().unwrap_or("Channel"),
                metric["rising_edges"],
                metric["frequency_hz"]
            )));
        }
    } else if let Some(frequency) = frame["frequency_hz"].as_f64() {
        body = body.child(label(format!("{frequency:.2} Hz")));
    }
    body.child(
        Collapsible::new("Capture identity & settings", hw.details)
            .on_toggle(|state: &mut State| state.ui.hardware.details = !state.ui.hardware.details)
            .child(
                secondary(serde_json::to_string_pretty(&details).unwrap())
                    .font(font("--rb-text-caption", 400.0).mono())
                    .wrap_anywhere(),
            )
            .build(&theme::kit()),
    )
}

fn status(hw: &Hardware) -> (String, &'static str) {
    if let Some(error) = hw.status["error"].as_str() {
        return (error.into(), "danger");
    }
    if hw.status["enabled"] != true {
        return ("No acquisition adapter configured".into(), "warning");
    }
    if hw.source == Source::Scope {
        if hw
            .frame
            .as_ref()
            .is_some_and(|frame| frame["origin"] == "replay")
        {
            return ("Replay · not live scope input".into(), "info");
        }
        if hw.status["connected"] == true {
            let age = hw.status["age_ms"].as_f64().unwrap_or(0.0) / 1000.0;
            return (
                if hw.frame.is_none() {
                    "Acquisition sender connected · waiting for capture".into()
                } else {
                    format!("Connected · capture {age:.1} s old")
                },
                if age > 10.0 { "warning" } else { "success" },
            );
        }
        return (
            if hw.frame.is_some() {
                "Disconnected · last capture"
            } else {
                "Acquisition sender disconnected"
            }
            .into(),
            "secondary",
        );
    }
    if hw.source == Source::Pico {
        return (
            if hw.status["connected"] == true
                && hw.status["age_ms"].as_u64().is_some_and(|age| age < 2500)
            {
                if hw.status["device"]["output_enabled"] == true {
                    "Pico connected · output enabled"
                } else {
                    "Pico connected · output off"
                }
            } else {
                "Pico disconnected"
            }
            .into(),
            "info",
        );
    }
    (
        if hw.status["busy"] == true {
            "Analyzer acquiring"
        } else if hw.status["installed"] == true {
            "Analyzer ready · USB recognition requires Scan"
        } else {
            "sigrok-cli unavailable"
        }
        .into(),
        "info",
    )
}

fn acquire(state: &State, width: f32) -> El {
    let hw = &state.ui.hardware;
    let busy = hw.receiver.is_some();
    let enabled = hw.status["enabled"] == true;
    let mut instruments = row().w_full().flex_wrap().gap(8.0);
    for source in [Source::Scope, Source::Logic, Source::Pico] {
        instruments = instruments.child(
            Button::new(source.title())
                .variant(ButtonVariant::Neutral)
                .disabled(busy)
                .on_press(move |state: &mut State| {
                    let hw = &mut state.ui.hardware;
                    hw.source = source;
                    hw.frame = None;
                    hw.status = Value::Null;
                    hw.live = false;
                    hw.error = None;
                    hw.setup = None;
                    hw.zoom = 1.0;
                    hw.cursor = None;
                    hw.due = 0.0;
                })
                .build(&theme::kit())
                .key(format!("hardware-source-{}", source.id()))
                .fill(color(if hw.source == source {
                    "--rb-color-selected"
                } else {
                    "--rb-color-surface"
                }))
                .h(40.0),
        );
    }
    let (status, tone) = status(hw);
    let mut body = col()
        .w_full()
        .gap(16.0)
        .child(instruments)
        .child(badge(status, tone).key("hardware-status"));
    let mut actions = row().w_full().flex_wrap().gap(8.0);
    match hw.source {
        Source::Scope => {
            let recording = hw.status["recording"] == true;
            actions = actions
                .child(button(
                    if recording {
                        "Stop recording"
                    } else {
                        "Record 30 s"
                    },
                    "hardware-record",
                    true,
                    busy || !enabled || (!recording && hw.status["connected"] != true),
                    move |state| {
                        operate(
                            state,
                            "/api/scope/recording",
                            json!({"op":if recording {"stop"} else {"start"}}),
                        )
                    },
                ))
                .child(button(
                    "Disconnect",
                    "hardware-disconnect",
                    false,
                    busy || hw.status["connected"] != true,
                    |state| operate(state, "/api/scope/disconnect", json!({"op":"disconnect"})),
                ))
                .child(button(
                    "Adapter setup",
                    "hardware-setup",
                    false,
                    busy,
                    |state| {
                        state
                            .ui
                            .hardware
                            .start(state.host.port, state.host.waker.clone(), |port| {
                                api::request(port, "/api/scope/setup", None).map(Reply::Action)
                            })
                    },
                ));
        }
        Source::Logic => {
            let blocked =
                busy || !enabled || hw.status["installed"] != true || hw.status["busy"] == true;
            actions = actions
                .child(button(
                    if hw.live {
                        "Stop preview"
                    } else {
                        "Start preview"
                    },
                    "hardware-live",
                    true,
                    blocked && !hw.live,
                    |state| {
                        let hw = &mut state.ui.hardware;
                        hw.live = !hw.live;
                        hw.due = 0.0;
                    },
                ))
                .child(button(
                    "Record 10 s",
                    "hardware-record",
                    false,
                    blocked,
                    |state| {
                        state.ui.hardware.live = false;
                        operate(state, "/api/logic/capture", json!({"profile":"pulse-loop"}));
                    },
                ))
                .child(button(
                    "Scan USB",
                    "hardware-scan",
                    false,
                    blocked,
                    |state| {
                        state.ui.hardware.live = false;
                        operate(state, "/api/logic/scan", json!({"op":"scan"}));
                    },
                ));
        }
        Source::Pico => {
            body = body.child(
                Checkbox::new(hw.wiring)
                    .label("Powered 3.3 V receivers and common ground confirmed")
                    .on_toggle(|state: &mut State| {
                        state.ui.hardware.wiring = !state.ui.hardware.wiring
                    })
                    .build(&theme::kit())
                    .key("hardware-wiring"),
            );
            let connected = hw.status["connected"] == true
                && hw.status["age_ms"].as_u64().is_some_and(|age| age < 2500);
            actions = actions
                .child(button(
                    "Start 1 kHz pulse",
                    "hardware-pico-start",
                    true,
                    busy || !enabled
                        || !connected
                        || hw.status["device"]["profile"] == "ddcs-uart/1"
                        || !hw.wiring
                        || hw.status["device"]["output_enabled"] == true,
                    |state| {
                        operate(
                            state,
                            "/api/pico/control",
                            json!({"op":"start","wiring_confirmed":true}),
                        )
                    },
                ))
                .child(button(
                    "Stop output",
                    "hardware-pico-stop",
                    false,
                    busy || !enabled,
                    |state| operate(state, "/api/pico/control", json!({"op":"stop"})),
                ));
            body = body.child(secondary(
                "Requested PWM settings are not independent measurements.",
            ));
            if let Some(device) = hw.status["device"].as_object() {
                body = body.child(label(format!(
                    "GP2 · {} Hz requested · lease {} ms",
                    device.get("frequency_hz").unwrap_or(&Value::Null),
                    device.get("lease_timeout_ms").unwrap_or(&Value::Null)
                )));
            }
        }
    }
    if hw.frame.is_some() && hw.source != Source::Pico {
        actions = actions
            .child(button(
                "Save capture",
                "hardware-save",
                false,
                busy || !hw.current_frame(),
                |state| {
                    let hw = &state.ui.hardware;
                    let key = if hw.source == Source::Scope {
                        hw.status["key"].as_str()
                    } else {
                        hw.frame.as_ref().and_then(|frame| frame["sha256"].as_str())
                    }
                    .map(str::to_owned);
                    if let Some(key) = key {
                        operate(
                            state,
                            "/api/captures",
                            json!({"op":"save","source":state.ui.hardware.source.id(),"key":key}),
                        );
                    }
                },
            ))
            .child(export_button(
                "Export raw",
                "hardware-export",
                false,
                busy || !hw.current_frame(),
            ));
        if hw.source == Source::Scope {
            actions = actions.child(export_button(
                "Export CSV",
                "hardware-export-csv",
                true,
                busy || !hw.current_frame(),
            ));
        }
    }
    body = body.child(actions);
    if hw.source != Source::Pico {
        body = body.child(waveform(hw, width)).child(frame_details(hw));
    }
    if let Some(setup) = &hw.setup {
        body = body
            .child(label(if setup["remote_login_running"] == true {
                "The configured transport is available."
            } else {
                "Configure and verify your acquisition transport before connecting."
            }))
            .child(
                Button::new("Copy connection instructions")
                    .on_press_tx(|state: &mut State, tx| {
                        if let Some(instructions) = state
                            .ui
                            .hardware
                            .setup
                            .as_ref()
                            .and_then(|setup| setup["handoff"].as_str())
                        {
                            tx.copy(instructions);
                        }
                    })
                    .build(&theme::kit())
                    .h(40.0),
            )
            .child(
                Collapsible::new("Connection details", hw.connection_details)
                    .on_toggle(|state: &mut State| {
                        state.ui.hardware.connection_details = !state.ui.hardware.connection_details
                    })
                    .child(
                        secondary(
                            setup["handoff"]
                                .as_str()
                                .unwrap_or("Connection instructions unavailable"),
                        )
                        .wrap_anywhere(),
                    )
                    .build(&theme::kit()),
            );
    }
    body
}

fn captures(state: &State, width: f32) -> El {
    let hw = &state.ui.hardware;
    let busy = hw.receiver.is_some();
    let entries = hw.saved[if hw.trash { "trash" } else { "entries" }].as_array();
    let mut toolbar = row().w_full().flex_wrap().gap(8.0);
    for trash in [false, true] {
        toolbar = toolbar.child(button(
            if trash { "Trash" } else { "Library" },
            if trash {
                "captures-trash"
            } else {
                "captures-library"
            },
            hw.trash == trash,
            busy,
            move |state| state.ui.hardware.trash = trash,
        ));
    }
    toolbar = toolbar.child(button(
        if hw.trash {
            "Empty Trash"
        } else {
            "Clear library"
        },
        "captures-cleanup",
        false,
        busy || entries.is_none_or(Vec::is_empty) || hw.saved["enabled"] != true,
        |state| state.ui.hardware.confirm = Some(state.ui.hardware.trash),
    ));
    let mut list = col().w_full().gap(12.0);
    if let Some(entries) = entries {
        if entries.is_empty() {
            list = list.child(secondary(if hw.trash {
                "Trash is empty"
            } else {
                "No saved captures"
            }));
        }
        for entry in entries {
            let Some(id) = entry["id"].as_str().filter(|id| api::capture_id(id)) else {
                continue;
            };
            let loaded = hw.selected.as_deref() == Some(id);
            let trash = hw.trash;
            let id = id.to_owned();
            let remove_id = id.clone();
            let mut card = col()
                .w_full()
                .padding(16.0)
                .gap(8.0)
                .fill(color(if loaded {
                    "--rb-color-selected"
                } else {
                    "--rb-color-surface"
                }))
                .child(label(if entry["kind"] == "scope" {
                    "Hantek CH1"
                } else {
                    "Logic CH0 / CH1"
                }))
                .child(secondary(
                    entry["saved_at"].as_str().unwrap_or("Date unavailable"),
                ));
            if let Some(error) = entry["error"].as_str() {
                card = card.child(
                    label(format!("Damaged receipt · {error}")).color(color("--rb-color-warning")),
                );
            } else {
                card = card.child(secondary(format!(
                    "{} samples · {} bytes",
                    entry["frame"]["samples"], entry["bytes"]
                )));
            }
            let mut actions = row().flex_wrap().gap(8.0).child(button(
                if trash { "Restore" } else { "Load" },
                &format!("capture-{}", entry["id"].as_str().unwrap()),
                false,
                busy || (!trash && entry["error"].is_string()),
                move |state| {
                    operate(
                        state,
                        "/api/captures",
                        json!({"op":if trash {"restore"} else {"load"},"id":id}),
                    )
                },
            ));
            if !trash {
                actions = actions.child(button(
                    "Move to Trash",
                    &format!("trash-{remove_id}"),
                    false,
                    busy,
                    move |state| {
                        operate(state, "/api/captures", json!({"op":"trash","id":remove_id}))
                    },
                ));
            }
            list = list.child(card.child(actions));
        }
    }
    let mut preview = col().w_full().min_w(0.0).gap(16.0);
    if hw.selected.is_some() && hw.frame.is_some() {
        toolbar = toolbar.child(export_button("Export raw", "hardware-export", false, busy));
        if hw
            .frame
            .as_ref()
            .is_some_and(|frame| frame["counts"].is_array())
        {
            toolbar = toolbar.child(export_button(
                "Export CSV",
                "hardware-export-csv",
                true,
                busy,
            ));
        }
        preview = preview
            .child(badge("Saved replay · not live", "info"))
            .child(waveform(hw, width))
            .child(frame_details(hw));
    } else {
        preview = preview.child(secondary("Load a capture to inspect its original samples"));
    }
    let mut body = col().w_full().gap(16.0).child(toolbar);
    if hw.saved["enabled"] == false {
        body = body.child(label("Capture storage is disabled in this host"));
    }
    if hw.saved["importing"] == true {
        body = body.child(secondary(
            "Importing previous captures · originals preserved",
        ));
    }
    if let Some(purge) = hw.confirm {
        body=body.child(label(if purge {"Permanently delete every capture in Trash?"} else {"Move all saved captures to Trash?"}))
            .child(row().flex_wrap().gap(8.0).child(button("Cancel","captures-cancel",false,busy,|state| state.ui.hardware.confirm=None))
                .child(button(if purge {"Delete permanently"} else {"Move to Trash"},"captures-confirm",true,busy,move |state| {state.ui.hardware.confirm=None;operate(state,"/api/captures",json!({"op":if purge {"purge_trash"} else {"trash_all"},"confirm":true}));})));
    }
    let list = list
        .max_h(if width >= 800.0 { 420.0 } else { 180.0 })
        .scroll(ScrollSpec {
            stick_bottom: false,
        });
    body.child(if width >= 800.0 {
        row()
            .w_full()
            .gap(24.0)
            .child(list.w(320.0))
            .child(preview.grow(1.0).min_w(0.0))
    } else {
        col().w_full().gap(16.0).child(list).child(preview)
    })
}

pub fn view(state: &State, cx: &Cx<State>, width: f32) -> El {
    let hw = &state.ui.hardware;
    let plot_width = cx
        .layout_size("hardware-waveform")
        .map_or(width, |size| size[0]);
    let toolbar = row()
        .w_full()
        .flex_wrap()
        .gap(8.0)
        .child(button(
            "Acquire",
            "hardware-acquire",
            !hw.captures,
            false,
            |state| {
                state.ui.hardware.captures = false;
                state.ui.hardware.frame = None;
                state.ui.hardware.selected = None;
                state.ui.hardware.due = 0.0;
            },
        ))
        .child(button(
            "Captures",
            "hardware-captures",
            hw.captures,
            false,
            |state| {
                state.ui.hardware.captures = true;
                state.ui.hardware.live = false;
                state.ui.hardware.frame = None;
                state.ui.hardware.selected = None;
                state.ui.hardware.due = 0.0;
            },
        ))
        .child(button(
            "Refresh",
            "hardware-refresh",
            false,
            hw.receiver.is_some() || state.host.loading,
            |state| {
                state.host.refresh();
                state.ui.hardware.due = 0.0;
            },
        ))
        .child(Button::new("Full Hardware · 2D / 3D")
            .variant(ButtonVariant::Ghost)
            .help("Open the remaining bench, experiment, equipment and wiring views in the existing local workspace")
            .on_press(|state: &mut State| workspace::open_existing(state,"hardware"))
            .build(&theme::kit()).key("hardware-full-workspace").h(44.0));
    let mut body = col()
        .w_full()
        .gap(16.0)
        .child(label("Acquire & Captures").font(font("--rb-text-title", 500.0)))
        .child(toolbar);
    if let Some(error) = &hw.error {
        body = body.child(
            col()
                .w_full()
                .gap(8.0)
                .child(
                    label(error)
                        .key("hardware-error")
                        .color(color("--rb-color-danger")),
                )
                .child(
                    command("Diagnostics", false)
                        .on_press(|state: &mut State| state.ui.diagnostics = true)
                        .build(&theme::kit())
                        .key("hardware-diagnostics")
                        .h(44.0),
                ),
        );
    }
    if let Some(message) = &hw.message {
        body = body.child(secondary(message).key("hardware-message"));
    }
    if hw.receiver.is_some() {
        body = body.child(secondary("Updating…").key("hardware-busy"));
    }
    body = body.child(if hw.captures {
        captures(state, plot_width)
    } else {
        acquire(state, plot_width)
    });
    col()
        .w_full()
        .h_full()
        .min_h(0.0)
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .padding(24.0)
        .child(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    fn runtime(width: f32) -> Runtime<State> {
        let mut state = State::idle();
        state.page = host::Page::Hardware;
        state.ui.hardware.status =
            json!({"ok":true,"enabled":true,"connected":false,"recording":false});
        let mut rt = Runtime::new(
            state,
            super::super::view,
            (width, 800.0),
            Env {
                system_dark: true,
                scale: 1.0,
            },
        );
        rt.frame(0.0);
        rt.frame(0.0);
        rt
    }

    #[test]
    fn disconnected_scope_has_no_fake_frame_or_enabled_recording() {
        for width in [1440.0, 1000.0, 560.0] {
            let rt = runtime(width);
            assert!(
                rt.elements()
                    .iter()
                    .any(|element| element.key.as_deref() == Some("hardware-waveform-empty"))
            );
            for key in [
                "hardware-acquire",
                "hardware-captures",
                "hardware-record",
                "hardware-setup",
            ] {
                let elements = rt.elements();
                let element = elements
                    .iter()
                    .find(|element| element.key.as_deref() == Some(key))
                    .unwrap();
                assert!(
                    element.rect[0] >= 0.0 && element.rect[0] + element.rect[2] <= width + 1.0,
                    "{key}: {:?}",
                    element.rect
                );
            }
        }
    }

    #[test]
    fn errors_retain_the_last_frame_and_saved_replay_is_never_live() {
        let mut hw = Hardware::default();
        let frame = json!({"samples":3,"sample_rate_hz":1000,"sha256":"a".repeat(64),"origin":"replay","time":[0,0.001,0.002],"counts":[-1,0,1]});
        hw.accept(Reply::Status(
            json!({"ok":true,"enabled":true,"connected":true,"frame":frame}),
        ))
        .unwrap();
        assert_eq!(status(&hw).0, "Replay · not live scope input");
        assert!(hw.current_frame());
        let original = hw.frame.clone();
        assert!(
            hw.accept(Reply::Status(json!({"ok":true,"frame":{"counts":[3.3]}})))
                .is_err()
        );
        assert_eq!(hw.frame, original);
        hw.accept(Reply::Status(
            json!({"ok":true,"enabled":true,"connected":false}),
        ))
        .unwrap();
        assert_eq!(hw.frame, original);
        assert!(!hw.current_frame());
    }
}
