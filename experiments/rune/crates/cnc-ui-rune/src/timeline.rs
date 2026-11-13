use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};

use rune::{
    Cx, El, FileDialogOptions, Image, ImageCache, Key, KeyEvent, Phase, ScrollSpec, Tx, col, el,
    row,
};
use rune_kit::{Button, ButtonVariant};
use serde_json::{Value, json};

use super::{State, catalog, host, label, secondary, theme, timeline_model as model, workspace};
use catalog::value;
use theme::{color, font, size};
use workspace::{badge, command};

type Reply = Result<model::History, String>;

#[derive(Default)]
pub struct Timeline {
    pub app: bool,
    case: Option<String>,
    data: Option<model::History>,
    receiver: Option<Receiver<Reply>>,
    error: Option<String>,
    selection: HashMap<(String, bool), String>,
    due: Option<f64>,
    scrub: Option<f64>,
    image: Option<Image>,
    image_error: Option<String>,
    image_receiver: Option<Receiver<Result<Vec<u8>, String>>>,
    image_key: Option<String>,
    image_ready: bool,
}

impl Timeline {
    fn index(&self) -> usize {
        let Some(data) = &self.data else { return 0 };
        self.selection
            .get(&(self.case.clone().unwrap_or_default(), self.app))
            .and_then(|id| data.moments.iter().position(|moment| moment["id"] == *id))
            .unwrap_or(data.moments.len().saturating_sub(1))
    }

    fn select(&mut self, index: usize) {
        self.due = None;
        self.scrub = None;
        if let Some(data) = &self.data
            && let Some(moment) = data
                .moments
                .get(index.min(data.moments.len().saturating_sub(1)))
        {
            self.selection.insert(
                (self.case.clone().unwrap_or_default(), self.app),
                value(moment, "id", "").into(),
            );
        }
    }

    fn step(&mut self, delta: i32) {
        self.select(self.index().saturating_add_signed(delta as isize));
    }

    fn seek(&mut self, position: f64) {
        let Some(data) = &self.data else { return };
        let times = data.times();
        let (start, end) = model::bounds(&times);
        let time = start + position.clamp(0.0, 1.0) * (end - start);
        let index = times
            .iter()
            .rposition(|stamp| *stamp as f64 <= time)
            .unwrap_or(0);
        self.select(index);
        self.scrub = Some(position);
    }

    pub fn stop(&mut self) {
        self.due = None;
    }

    pub fn refresh(
        &mut self,
        port: u16,
        case: Option<String>,
        waker: Option<rune::Waker>,
        expected_health: Option<Value>,
    ) {
        self.stop();
        self.scrub = None;
        if self.case != case {
            self.data = None;
            self.case = case.clone();
        }
        self.error = None;
        self.receiver = None;
        let Some(case) = case else { return };
        let app = self.app;
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        std::thread::spawn(move || {
            let result = (|| {
                let health = host::read(port, "/api/desktop/health")?;
                if expected_health.as_ref() != Some(&health) || health["ok"] != true {
                    return Err("Host identity changed. Refresh the library before reading timeline evidence.".into());
                }
                let data = host::read(port, "/process/timeline.json")?;
                let visuals = host::read(port, "/timeline/evidence.json").unwrap_or(Value::Null);
                let (history, experiments) = if app {
                    (Value::Null, Value::Null)
                } else {
                    (
                        host::read(port, &format!("/api/library/history?case_id={case}"))?,
                        host::read(port, &format!("/api/library/experiments?case_id={case}"))?,
                    )
                };
                if health != host::read(port, "/api/desktop/health")? {
                    return Err("Host build changed during refresh. Refresh again.".into());
                }
                model::History::parse(data, visuals, history, experiments, &case, app)
            })();
            let _ = sender.send(result);
            if let Some(wake) = waker {
                wake();
            }
        });
    }

    pub fn poll(&mut self, port: u16, waker: Option<rune::Waker>, clock: f64, tx: &mut Tx) -> bool {
        let mut changed = false;
        if let Some(reply) = self
            .receiver
            .as_ref()
            .and_then(|receiver| receiver.try_recv().ok())
        {
            self.receiver = None;
            match reply {
                Ok(data) => {
                    self.data = Some(data);
                    self.error = None;
                }
                Err(error) => self.error = Some(error),
            }
            changed = true;
        }
        if let Some(due) = self.due {
            if clock >= due {
                self.step(1);
                let len = self.data.as_ref().map_or(0, |data| data.moments.len());
                if self.index() + 1 < len {
                    self.due = Some(clock + 1.0);
                }
                changed = true;
            }
            if let Some(due) = self.due {
                tx.wake_at(due);
            }
        }
        let image = self
            .data
            .as_ref()
            .and_then(|data| {
                data.moments
                    .get(self.index())
                    .map(|moment| data.snapshot(moment))
            })
            .and_then(|snapshot| {
                snapshot["images"]
                    .as_array()
                    .and_then(|images| images.first())
                    .cloned()
            });
        let key = image
            .as_ref()
            .map(|image| format!("{}:{}", value(image, "src", ""), value(image, "sha256", "")));
        if self.image_key != key {
            self.image_key = key;
            self.image = None;
            self.image_ready = false;
            self.image_error = None;
            self.image_receiver = None;
            if let Some(image) = image {
                let (sender, receiver) = mpsc::channel();
                self.image_receiver = Some(receiver);
                let wake = waker.clone();
                std::thread::spawn(move || {
                    let result =
                        host::read_bytes(port, value(&image, "src", "")).and_then(|bytes| {
                            if crate::sha256(&bytes) != value(&image, "sha256", "") {
                                Err("Retained image hash does not match its checkpoint.".into())
                            } else {
                                Ok(bytes)
                            }
                        });
                    let _ = sender.send(result);
                    if let Some(wake) = wake {
                        wake();
                    }
                });
            }
            changed = true;
        }
        if let Some(reply) = self
            .image_receiver
            .as_ref()
            .and_then(|receiver| receiver.try_recv().ok())
        {
            self.image_receiver = None;
            match reply {
                Ok(bytes) => {
                    self.image = Some(
                        ImageCache::new(waker.unwrap_or_else(|| std::sync::Arc::new(|| {})))
                            .load(bytes),
                    )
                }
                Err(error) => self.image_error = Some(error),
            }
            changed = true;
        }
        let ready = self
            .image
            .as_ref()
            .is_some_and(|image| image.size().is_some() || image.error().is_some());
        if ready != self.image_ready {
            self.image_ready = ready;
            changed = true;
        }
        changed
    }
}

pub fn refresh(state: &mut State) {
    state.ui.timeline.refresh(
        state.host.port,
        state.host.selected.clone(),
        state.host.waker.clone(),
        state
            .host
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.health.clone()),
    );
}

pub fn open(state: &mut State) {
    state.page = host::Page::Timeline;
    state.ui.navigation_open = false;
    refresh(state);
}

pub fn key(state: &mut State, event: &KeyEvent, _: &mut Tx) -> bool {
    if state.page != host::Page::Timeline
        || state.ui.chooser
        || state.ui.navigation_open
        || event.mods.cmd
        || event.mods.ctrl
        || event.mods.alt
    {
        return false;
    }
    let timeline = &mut state.ui.timeline;
    match event.key {
        Key::Left => timeline.step(-1),
        Key::Right => timeline.step(1),
        Key::Home => timeline.select(0),
        Key::End => timeline.select(usize::MAX),
        _ => return false,
    }
    true
}

fn save(state: &mut State, tx: &mut Tx) {
    let timeline = &mut state.ui.timeline;
    timeline.stop();
    let Some(data) = &timeline.data else { return };
    let Some(moment) = data.moments.get(timeline.index()) else {
        return;
    };
    let mut bytes = serde_json::to_vec_pretty(
        &json!({"schema":1,"moment":moment,"snapshot":data.snapshot(moment)}),
    )
    .unwrap();
    bytes.push(b'\n');
    let options = FileDialogOptions {
        title: "Save checkpoint".into(),
        file_name: Some(format!("checkpoint-{}.json", value(moment, "id", ""))),
        ..Default::default()
    }
    .filter("JSON checkpoint", &["json"]);
    tx.save_file(options, move |state: &mut State, result, _| match result {
        Ok(Some(path)) => {
            state.ui.notice = Some(match std::fs::write(path, bytes) {
                Ok(()) => "Checkpoint saved.".into(),
                Err(error) => format!("Could not save checkpoint: {error}"),
            });
        }
        Ok(None) => {}
        Err(error) => state.ui.notice = Some(format!("Could not save checkpoint: {error}")),
    });
}

fn date(time: i64, timezone: chrono_tz::Tz, format: &str) -> String {
    chrono::DateTime::from_timestamp_millis(time)
        .unwrap()
        .with_timezone(&timezone)
        .format(format)
        .to_string()
}

fn chart(state: &State, cx: &mut Cx<State>, width: f32) -> El {
    let timeline = &state.ui.timeline;
    let data = timeline.data.as_ref().unwrap();
    let times = data.times();
    let (start, end) = model::bounds(&times);
    let track_width = cx
        .layout_size("timeline-track")
        .map_or((width - 160.0).max(120.0), |size| size[0]);
    let current = timeline.index();
    let cursor = timeline
        .scrub
        .map(|position| position as f32)
        .unwrap_or_else(|| model::ratio(times[current] as f64, start, end));
    let mut ticks = el().key("timeline-track").grow(1.0).min_w(0.0).h(44.0);
    let count = if end - start <= 86_400_000.0 { 3 } else { 5 };
    for i in 0..count {
        let r = i as f32 / (count - 1) as f32;
        let time = start + (end - start) * r as f64;
        let label = secondary(date(
            time as i64,
            data.timezone,
            if track_width < 500.0 {
                "%b %-d"
            } else if end - start < 345_600_000.0 {
                "%b %-d %H:%M"
            } else {
                "%b %-d"
            },
        ))
        .font(font("--rb-text-caption", 400.0));
        ticks = ticks.child(
            label
                .absolute((track_width - 100.0).max(0.0) * r, 0.0)
                .w(100.0),
        );
    }
    let seek = el()
        .key("timeline-seek")
        .w_full()
        .h(44.0)
        .role("slider")
        .focusable(true)
        .label("Scrub investigation dates")
        .a11y_value(format!(
            "{}; {}",
            date(times[current], data.timezone, "%b %-d %H:%M %Z"),
            value(&data.moments[current], "title", "")
        ))
        .range_value(times[current] as f64, start, end)
        .range_step(1.0)
        .disabled(times.len() < 2)
        .on_step(|state: &mut State, delta, _| state.ui.timeline.step(delta))
        .on_step_limit(|state: &mut State, end, _| {
            state.ui.timeline.select(if end { usize::MAX } else { 0 })
        })
        .on_drag(|state: &mut State, drag, _| {
            if state
                .ui
                .timeline
                .data
                .as_ref()
                .is_none_or(|data| data.moments.len() < 2)
            {
                return;
            }
            if matches!(drag.phase, Phase::Up) {
                state.ui.timeline.scrub = None;
            } else {
                state.ui.timeline.seek((drag.x / drag.w.max(1.0)) as f64);
            }
        })
        .child(
            el().absolute(cursor * (track_width - 2.0), 24.0)
                .size(2.0, 20.0)
                .fill(color("--rb-color-info")),
        );
    ticks = ticks.child(seek);
    let mut chart = col().w_full().gap(4.0).child(
        row()
            .w_full()
            .gap(16.0)
            .child(secondary("Recorded").w(112.0))
            .child(ticks),
    );
    for (lane_index, lane) in data.lanes.iter().enumerate() {
        let indices: Vec<_> = data
            .moments
            .iter()
            .enumerate()
            .filter(|(_, moment)| moment[if timeline.app { "lane" } else { "stage" }] == lane["id"])
            .map(|(index, _)| index)
            .collect();
        if indices.is_empty() {
            continue;
        }
        let mut track = el()
            .grow(1.0)
            .min_w(0.0)
            .h(44.0)
            .child(
                el().absolute(0.0, 21.0)
                    .w_full()
                    .h(1.0)
                    .fill(color("--rb-color-divider")),
            )
            .child(
                el().absolute(cursor * (track_width - 2.0), 0.0)
                    .size(2.0, 44.0)
                    .fill(color("--rb-color-info")),
            );
        let lane_color = color(&format!("--rb-chart-{}", lane_index % 6 + 1));
        for (group_index, marker) in model::markers(&indices, &times, start, end, track_width)
            .into_iter()
            .enumerate()
        {
            let selected = marker.indices.contains(&current);
            let name = if marker.indices.len() == 1 {
                let moment = &data.moments[marker.indices[0]];
                format!(
                    "{} · {}",
                    date(times[marker.indices[0]], data.timezone, "%b %-d %H:%M %Z"),
                    value(moment, "title", "")
                )
            } else {
                format!(
                    "{} checkpoints. Activate to cycle through this group.",
                    marker.indices.len()
                )
            };
            let group = marker.indices.clone();
            let amount = marker.indices.len();
            track = track.child(
                Button::new(name)
                    .variant(ButtonVariant::Ghost)
                    .label_with(move |_, _| {
                        let dot = el()
                            .center()
                            .size(
                                if amount > 1 { 20.0 } else { 16.0 },
                                if amount > 1 { 20.0 } else { 16.0 },
                            )
                            .radius(10.0)
                            .fill(lane_color);
                        let dot = if amount > 1 {
                            dot.child(
                                label(amount.to_string())
                                    .font(font("--rb-text-caption", 500.0))
                                    .color(color("--rb-color-canvas")),
                            )
                        } else {
                            dot
                        };
                        row().center().child(dot)
                    })
                    .on_press(move |state: &mut State| {
                        let current = state.ui.timeline.index();
                        let next = group
                            .iter()
                            .position(|index| *index == current)
                            .map_or(0, |position| (position + 1) % group.len());
                        state.ui.timeline.select(group[next]);
                    })
                    .build(&theme::kit())
                    .checked(selected)
                    .key(format!("timeline-point-{lane_index}-{group_index}"))
                    .size(marker.width, 44.0)
                    .absolute(marker.x - marker.width / 2.0, 0.0)
                    .border(
                        if selected { 2.0 } else { 0.0 },
                        color("--rb-color-selection-edge"),
                    )
                    .radius(4.0),
            );
        }
        chart = chart.child(
            row()
                .w_full()
                .gap(16.0)
                .center_y()
                .child(secondary(value(lane, "label", "")).w(112.0))
                .child(track),
        );
    }
    chart
}

fn keyframe(state: &State, narrow: bool) -> El {
    let timeline = &state.ui.timeline;
    let data = timeline.data.as_ref().unwrap();
    let moment = &data.moments[timeline.index()];
    let snapshot = data.snapshot(moment);
    let kind = value(moment, "kind", "");
    let lower = kind.to_lowercase();
    let tone = if lower.contains("fail") || lower.contains("stop") {
        "danger"
    } else if lower.contains("correct") || lower.contains("partial") {
        "warning"
    } else if lower.contains("compar") || lower.contains("execut") {
        "success"
    } else {
        "info"
    };
    let mut copy = col()
        .min_w(0.0)
        .grow(1.0)
        .gap(12.0)
        .child(
            row()
                .flex_wrap()
                .gap(12.0)
                .center_y()
                .child(badge(kind, tone))
                .child(secondary(date(
                    model::timestamp(moment).unwrap(),
                    data.timezone,
                    "%b %-d %Y %H:%M %Z",
                ))),
        )
        .child(
            label(value(moment, "title", ""))
                .key("timeline-title")
                .font(font("--rb-text-body", 500.0)),
        )
        .child(label(value(moment, "summary", "")).key("timeline-summary"));
    let mut metrics = row().w_full().flex_wrap().gap(24.0);
    for metric in snapshot["metrics"].as_array().into_iter().flatten() {
        let number = match &metric["value"] {
            Value::Bool(true) => "Yes".into(),
            Value::Bool(false) => "No".into(),
            Value::String(value) => value.clone(),
            Value::Null => "Unknown".into(),
            value => value.to_string(),
        };
        metrics = metrics.child(
            col()
                .max_w(240.0)
                .gap(4.0)
                .child(secondary(value(metric, "label", "")))
                .child(label(number).font(font("--rb-text-title", 500.0))),
        );
    }
    copy = copy
        .child(metrics)
        .child(secondary(value(moment, "limit", "")).key("timeline-limit"));
    let source_count = moment["sources"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(snapshot["sources"].as_array().into_iter().flatten())
        .filter_map(|source| source["path"].as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();
    let mut actions = row()
        .flex_wrap()
        .gap(12.0)
        .child(
            command("Open investigation", false)
                .help("Open workspace inside the app")
                .on_press(|state: &mut State| {
                    state.ui.timeline.stop();
                    let timeline = &state.ui.timeline;
                    if let Some(data) = &timeline.data
                        && let Some(item) = workspace::selected(state)
                        && let Some(snapshot) = &state.host.snapshot
                        && let Some(path) = catalog::route(item, &snapshot.recovery, "rea")
                    {
                        let step = value(&data.moments[timeline.index()], "step", "target");
                        let step = if step
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                        {
                            step
                        } else {
                            "target"
                        };
                        workspace::open_path(
                            state,
                            &format!("{}#{step}", path.split('#').next().unwrap()),
                        );
                    }
                })
                .build(&theme::kit())
                .key("timeline-inspect")
                .h(40.0),
        )
        .child(
            Button::new("Save checkpoint")
                .variant(ButtonVariant::Ghost)
                .on_press_tx(save)
                .build(&theme::kit())
                .key("timeline-save")
                .h(40.0),
        );
    let identity = value(moment, "checkpoint", value(moment, "commit", "Unknown"));
    actions = actions.child(
        secondary(format!(
            "{} · {source_count} {}",
            identity.chars().take(12).collect::<String>(),
            if source_count == 1 {
                "source"
            } else {
                "sources"
            },
        ))
        .font(font("--rb-text-caption", 400.0)),
    );
    copy = copy.child(actions);
    let mut visual = col().min_w(0.0).gap(12.0);
    if let Some(image) = snapshot["images"]
        .as_array()
        .and_then(|images| images.first())
    {
        if let Some(image) = &timeline.image {
            if let Some([width, height]) = image.size() {
                visual = visual.child(
                    el().w_full()
                        .h(if narrow { 200.0 } else { 260.0 })
                        .max_w((if narrow { 200.0 } else { 260.0 }) * width as f32 / height as f32)
                        .pixels(image.pixels())
                        .label(value(
                            &snapshot["images"][0],
                            "label",
                            "Retained checkpoint image",
                        )),
                );
            } else if let Some(error) = image.error() {
                visual = visual.child(secondary(format!("Retained image unavailable: {error}")));
            } else {
                visual = visual.child(secondary("Loading retained image…"));
            }
        } else {
            visual = visual.child(secondary(
                timeline
                    .image_error
                    .as_deref()
                    .unwrap_or("Loading retained image…"),
            ));
        }
        visual = visual.child(secondary(value(image, "label", "")));
    } else if !snapshot["excerpt"].is_null() && snapshot["excerpt"]["language"] != "json" {
        let excerpt = &snapshot["excerpt"];
        let start = excerpt["startLine"].as_u64().unwrap_or(1);
        visual = visual.child(secondary(value(excerpt, "title", "Source excerpt")));
        let source = value(excerpt, "text", "").to_owned();
        let mut code = String::new();
        let mut gutters = Vec::new();
        for (index, line) in source.split('\n').enumerate() {
            if index > 0 {
                code.push('\n');
            }
            let offset = code.len();
            code.push_str(&format!("{:>4}  ", start + index as u64));
            gutters.push(offset..code.len());
            code.push_str(line);
        }
        let spans = theme::code_spans(&code, (excerpt["language"] == "c").then_some("c"));
        let mut excerpt_text = label(code)
            .key("timeline-excerpt")
            .spans(spans)
            .font(font("--rb-text-secondary", 400.0).mono())
            .wrap_anywhere()
            .selectable(true)
            .padding(12.0)
            .fill(color("--rb-color-field"));
        for gutter in gutters {
            excerpt_text = excerpt_text.uncopied(gutter);
        }
        visual = visual.child(excerpt_text).child(secondary(
            if value(excerpt, "path", "").starts_with("data/workbench/") {
                "Retained analysis checkpoint"
            } else {
                value(excerpt, "path", "")
            },
        ));
    }
    let mut contents = if narrow { col() } else { row() };
    contents = contents.w_full().gap(24.0).child(if narrow {
        copy.w_full()
    } else {
        copy.w_frac(0.55)
    });
    if !snapshot["excerpt"].is_null() && snapshot["excerpt"]["language"] != "json"
        || snapshot["images"]
            .as_array()
            .is_some_and(|images| !images.is_empty())
    {
        contents = contents.child(if narrow {
            visual.w_full()
        } else {
            visual.grow(1.0).w_frac(0.45)
        });
    }
    col()
        .key("timeline-keyframe")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .child(contents)
}

pub fn view(state: &State, cx: &mut Cx<State>, width: f32) -> El {
    let timeline = &state.ui.timeline;
    let mut body = col()
        .key("timeline-workspace")
        .w_full()
        .h_full()
        .min_h(0.0)
        .padding(size("--rb-gutter"))
        .gap(16.0);
    let title = if timeline.app {
        "App development"
    } else {
        "Reverse engineering"
    };
    body = body.child(
        row()
            .w_full()
            .justify_between()
            .center_y()
            .gap(16.0)
            .child(label(title).font(font("--rb-text-title", 500.0)))
            .child(
                Button::new(if timeline.receiver.is_some() {
                    "Refreshing…"
                } else {
                    "Refresh"
                })
                .variant(ButtonVariant::Ghost)
                .disabled(timeline.receiver.is_some())
                .on_press(refresh)
                .build(&theme::kit())
                .key("timeline-refresh")
                .h(40.0),
            ),
    );
    if let Some(error) = &timeline.error {
        body = body.child(
            label(format!(
                "{error}{}",
                if timeline.data.is_some() {
                    " Retained timeline; refresh failed."
                } else {
                    ""
                }
            ))
            .key("timeline-error")
            .color(color("--rb-color-danger")),
        );
    }
    if let Some(data) = &timeline.data
        && !data.moments.is_empty()
    {
        let index = timeline.index();
        body = body
            .child(chart(state, cx, width))
            .child(keyframe(state, width < 760.0));
        let controls = row()
            .w_full()
            .flex_wrap()
            .gap(12.0)
            .center_y()
            .child(
                command("Previous", false)
                    .disabled(index == 0)
                    .on_press(|state: &mut State| state.ui.timeline.step(-1))
                    .build(&theme::kit())
                    .key("timeline-previous")
                    .h(44.0),
            )
            .child(
                command(
                    if timeline.due.is_some() {
                        "Pause"
                    } else {
                        "Play events"
                    },
                    true,
                )
                .disabled(data.moments.len() < 2)
                .on_press_tx(|state: &mut State, tx: &mut Tx| {
                    let timeline = &mut state.ui.timeline;
                    if timeline.due.is_some() {
                        timeline.stop();
                        return;
                    }
                    let len = timeline.data.as_ref().map_or(0, |data| data.moments.len());
                    if len < 2 {
                        return;
                    }
                    if timeline.index() + 1 == len {
                        timeline.select(0);
                    }
                    timeline.due = Some(tx.clock() + 1.0);
                    tx.wake_at(tx.clock() + 1.0);
                })
                .build(&theme::kit())
                .key("timeline-play")
                .h(44.0),
            )
            .child(
                command("Next", false)
                    .disabled(index + 1 == data.moments.len())
                    .on_press(|state: &mut State| state.ui.timeline.step(1))
                    .build(&theme::kit())
                    .key("timeline-next")
                    .h(44.0),
            )
            .child(
                secondary(format!("{} / {}", index + 1, data.moments.len()))
                    .key("timeline-counter")
                    .role("status"),
            )
            .child(history_switch(timeline.app));
        body = body.child(controls);
    } else {
        body = body.child(
            label(if timeline.receiver.is_some() {
                "Loading saved checkpoints…"
            } else if timeline.case.is_none() {
                "Choose a system from the firmware library."
            } else if timeline.error.is_some() {
                "Timeline unavailable"
            } else {
                "No dated checkpoints for this system"
            })
            .key("timeline-empty"),
        );
        body = body.child(secondary(
            "The artifact remains selected. Dated evidence will appear here when retained.",
        ));
        body = body.child(history_switch(timeline.app));
    }
    body
}

fn history_switch(app: bool) -> El {
    Button::new(if app {
        "Device investigation"
    } else {
        "App history"
    })
    .variant(ButtonVariant::Ghost)
    .on_press(|state: &mut State| {
        state.ui.timeline.app = !state.ui.timeline.app;
        state.ui.timeline.data = None;
        refresh(state);
    })
    .build(&theme::kit())
    .key("timeline-history")
    .h(44.0)
}

#[cfg(test)]
#[path = "timeline_tests.rs"]
mod tests;
