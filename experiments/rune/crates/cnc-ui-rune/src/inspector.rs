use std::sync::mpsc::{self, Receiver};

use rune::{Cx, El, ScrollSpec, col, el, row, text};
use rune_kit::{Button, ButtonVariant, Collapsible};
use serde_json::Value;

use super::{State, catalog, host, label, secondary, theme, workspace};
use theme::{color, font, size};

const STEPS: [(&str, &str); 14] = [
    ("target", "Target binary"),
    ("boot", "OEM display"),
    ("trace", "Device trace"),
    ("kernel", "Kernel driver"),
    ("hardware", "Hardware evidence"),
    ("decompile", "Decompilation"),
    ("firmware-map", "Firmware map"),
    ("map", "Recovery map"),
    ("rebuild", "Reconstruction"),
    ("compare", "Comparison"),
    ("firmware", "Native build"),
    ("rust", "Rust interface"),
    ("cut", "Virtual cut"),
    ("workflow", "Workflow"),
];

type Reply = (String, Result<Value, String>);

#[derive(Default)]
pub struct Inspector {
    pub step: usize,
    data: Option<Value>,
    pub error: Option<String>,
    loading: bool,
    receiver: Option<Receiver<Reply>>,
    routine: usize,
    inspect: bool,
    bytes: bool,
    method: bool,
    instructions: bool,
    source: bool,
    steps: bool,
    identity: bool,
}

impl Inspector {
    pub fn poll(&mut self, selected: Option<&str>) -> bool {
        let Some((id, result)) = self.receiver.as_ref().and_then(|r| r.try_recv().ok()) else {
            return false;
        };
        self.receiver = None;
        self.loading = false;
        if selected != Some(&id) {
            return true;
        }
        match result {
            Ok(data) => {
                self.data = Some(data);
                self.error = None;
                self.routine = self.routine.min(
                    self.data.as_ref().unwrap()["functions"]
                        .as_array()
                        .map_or(0, |items| items.len().saturating_sub(1)),
                );
            }
            Err(error) => self.error = Some(error),
        }
        true
    }

    fn select(&mut self, index: usize) {
        self.routine = index;
        self.bytes = false;
        self.method = false;
        self.instructions = false;
        self.source = false;
    }
}

fn field<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or_default()
}

fn hex_bytes(value: &str) -> bool {
    !value.is_empty()
        && value.len().is_multiple_of(2)
        && value.bytes().all(|c| c.is_ascii_hexdigit())
}

fn parse(data: Value, expected: &str) -> Result<Value, String> {
    if !catalog::valid_id(expected) || data["target_sha256"] != expected {
        return Err("Binary evidence does not match the selected firmware.".into());
    }
    let file_size = data["file_size"]
        .as_u64()
        .filter(|n| *n > 0)
        .ok_or("Invalid binary file size.")?;
    let sections = data["sections"]
        .as_array()
        .ok_or("Binary sections are unavailable.")?;
    for section in sections {
        let offset = section["offset"]
            .as_u64()
            .ok_or("Invalid section offset.")?;
        let length = section["size"].as_u64().ok_or("Invalid section size.")?;
        if field(section, "name").is_empty()
            || !section["executable"].is_boolean()
            || offset.checked_add(length).is_none_or(|end| end > file_size)
        {
            return Err("Binary section exceeds the original file.".into());
        }
    }
    let routines = data["functions"]
        .as_array()
        .filter(|items| !items.is_empty())
        .ok_or("No recovered routine evidence.")?;
    for routine in routines {
        let bytes = field(routine, "bytes");
        let offset = routine["file_offset"]
            .as_u64()
            .ok_or("Invalid routine offset.")?;
        let address = u64::from_str_radix(field(routine, "address").trim_start_matches("0x"), 16)
            .map_err(|_| "Invalid routine address.")?;
        if !hex_bytes(bytes)
            || address.checked_add((bytes.len() / 2) as u64).is_none()
            || offset
                .checked_add((bytes.len() / 2) as u64)
                .is_none_or(|end| end > file_size)
            || !sections.iter().any(|section| {
                section["name"] == routine["section"]
                    && section["executable"] == true
                    && offset >= section["offset"].as_u64().unwrap()
                    && offset + (bytes.len() / 2) as u64
                        <= section["offset"].as_u64().unwrap() + section["size"].as_u64().unwrap()
            })
            || [
                "label",
                "why",
                "how",
                "source",
                "recovered_c",
                "proof",
                "proof_scope",
            ]
            .iter()
            .any(|key| field(routine, key).is_empty())
            || ["proof_sha256", "source_sha256"]
                .iter()
                .any(|key| !catalog::valid_id(field(routine, key)))
            || routine["comparison_cases"].as_u64().is_none()
        {
            return Err("Invalid recovered routine evidence.".into());
        }
        if let Some(evidence) = routine.get("instruction_evidence")
            && (!field(evidence, "evidence_id")
                .strip_prefix("ev_")
                .is_some_and(catalog::valid_id)
                || field(evidence, "path").is_empty()
                || !evidence["instructions"].as_array().is_some_and(|items| {
                    items.iter().all(|item| {
                        hex_bytes(field(item, "bytes"))
                            && ["address", "instruction", "meaning"]
                                .iter()
                                .all(|key| !field(item, key).is_empty())
                    })
                }))
        {
            return Err("Invalid original instruction evidence.".into());
        }
    }
    Ok(data)
}

pub fn refresh(state: &mut State) {
    if state.ui.inspector.step == 6 {
        super::graph::refresh(state);
        return;
    }
    if state.host.selected.as_deref() != Some(catalog::DDCS_SHA) {
        let Some(id) = state.host.selected.clone() else {
            return;
        };
        if state.ui.inspector.loading {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        state.ui.inspector.loading = true;
        state.ui.inspector.receiver = Some(receiver);
        let wake = state.host.waker.clone();
        std::thread::spawn(move || {
            let result =
                super::provider::overview(&id).map(|text| serde_json::json!({"overview":text}));
            let _ = sender.send((id, result));
            if let Some(wake) = wake {
                wake();
            }
        });
        return;
    }

    if state.ui.inspector.step == 1 {
        super::controller::preload(state, true);
        return;
    }
    if matches!(state.ui.inspector.step, 6 | 7) {
        super::graph::refresh(state);
        return;
    }
    if state.ui.inspector.loading || state.host.selected.as_deref() != Some(catalog::DDCS_SHA) {
        return;
    }
    let id = state.host.selected.clone().unwrap();
    let port = state.host.port;
    let wake = state.host.waker.clone();
    let expected_health = state
        .host
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.health.clone());
    let (sender, receiver) = mpsc::channel();
    state.ui.inspector.loading = true;
    state.ui.inspector.error = None;
    state.ui.inspector.receiver = Some(receiver);
    std::thread::spawn(move || {
        let result = (|| {
            let before = host::read(port, "/api/desktop/health")?;
            if before["ok"] != true || expected_health.as_ref() != Some(&before) {
                return Err(
                    "Host identity changed. Refresh the library before reading binary evidence."
                        .into(),
                );
            }
            let data = host::read(port, "/process/binary.json")?;
            if before != host::read(port, "/api/desktop/health")? {
                return Err("Host build changed during inspection. Refresh and retry.".into());
            }
            parse(data, &id)
        })();
        let _ = sender.send((id, result));
        if let Some(wake) = wake {
            wake();
        }
    });
}

fn open_step(state: &mut State, step: &str) {
    let path = state.host.snapshot.as_ref().and_then(|snapshot| {
        workspace::selected(state).and_then(|item| catalog::route(item, &snapshot.recovery, "rea"))
    });
    if let Some(path) = path {
        workspace::open_path(
            state,
            &format!("{}#{step}", path.split('#').next().unwrap()),
        );
    } else {
        state.ui.notice = Some("Select a system with a valid firmware identity first.".into());
    }
}

fn steps(state: &State) -> El {
    let mut list = col().w_full().gap(0.0);
    for (index, (id, title)) in STEPS.into_iter().enumerate() {
        let button = Button::new(format!("{} · {title}", index + 1))
            .variant(ButtonVariant::Ghost)
            .label_with(move |_, ink| {
                row()
                    .w_full()
                    .gap(8.0)
                    .center_y()
                    .child(
                        label(format!("{}", index + 1))
                            .key(format!("rea-step-number-{index}"))
                            .w(24.0)
                            .font(font("--rb-text-secondary", 400.0))
                            .color(ink),
                    )
                    .child(
                        label(title)
                            .key(format!("rea-step-label-{index}"))
                            .font(font("--rb-text-secondary", 400.0))
                            .color(ink),
                    )
            })
            .help(
                if matches!(id, "target" | "boot" | "firmware-map" | "map") {
                    "Native saved investigation evidence"
                } else {
                    "Open investigation inside the app"
                },
            )
            .on_press(move |state: &mut State| {
                if index == 1 {
                    state.ui.inspector.step = index;
                    super::controller::preload(state, false);
                } else if matches!(index, 0 | 6 | 7) {
                    state.ui.inspector.step = index;
                    refresh(state);
                } else {
                    open_step(state, id);
                }
            })
            .build(&theme::kit())
            .key(format!("rea-step-{id}"))
            .w_full()
            .h(40.0)
            .px(16.0)
            .radius(0.0);
        list = list.child(if index == state.ui.inspector.step {
            button.fill(color("--rb-color-selected"))
        } else {
            button
        });
    }
    list
}

fn bytes_text(routine: &Value) -> String {
    let start =
        u64::from_str_radix(field(routine, "address").trim_start_matches("0x"), 16).unwrap();
    field(routine, "bytes")
        .as_bytes()
        .chunks(32)
        .enumerate()
        .map(|(index, chunk)| {
            let bytes = chunk
                .chunks(2)
                .map(|pair| std::str::from_utf8(pair).unwrap())
                .collect::<Vec<_>>()
                .join(" ");
            format!("{:08x}  {bytes}", start + (index * 16) as u64)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn code(title: &str, source: &str, language: Option<&str>, key: &str) -> El {
    let spans = theme::code_spans(source, language);
    col()
        .w_full()
        .gap(12.0)
        .padding(16.0)
        .fill(color("--rb-color-field"))
        .child(secondary(title))
        .child(
            text(source)
                .key(key)
                .spans(spans)
                .font(font("--rb-text-secondary", 400.0).mono())
                .color(color("--rb-color-text"))
                .wrap()
                .selectable(true)
                .w_full(),
        )
}

fn section_map(data: &Value, routine: &Value, width: f32) -> El {
    let total = data["file_size"].as_u64().unwrap() as f32;
    let mut map = el().key("binary-map").instant_changes().size(width, 32.0).role("img")
        .label("Binary file layout. Orange: executable sections. Grey: data and metadata sections. Dark: headers and gaps. Blue: selected routine, enlarged for visibility. Not recovery progress.")
        .fill(color("--rb-color-raised"));
    for section in data["sections"].as_array().unwrap() {
        map = map.child(
            el().absolute(
                section["offset"].as_u64().unwrap() as f32 / total * width,
                0.0,
            )
            .size(
                section["size"].as_u64().unwrap() as f32 / total * width,
                32.0,
            )
            .fill(color(if section["executable"] == true {
                "--rb-color-action"
            } else {
                "--rb-color-control-edge"
            }))
            .help(format!(
                "{} · file offset 0x{:x} · {} bytes",
                field(section, "name"),
                section["offset"].as_u64().unwrap(),
                section["size"]
            )),
        );
    }
    map.child(
        el().key("binary-selection")
            .absolute(
                (routine["file_offset"].as_u64().unwrap() as f32 / total * width).min(width - 4.0),
                0.0,
            )
            .size(
                (field(routine, "bytes").len() as f32 / 2.0 / total * width).max(4.0),
                32.0,
            )
            .border(2.0, color("--rb-color-info"))
            .help("Selected routine; marker enlarged for visibility, not its proportional size"),
    )
}

fn routine_name(routine: &Value) -> &str {
    field(routine, "label")
}

fn process(data: &Value, routine: &Value, width: f32) -> El {
    let compact = width < 760.0;
    let mut flow = if compact { col() } else { row().center_y() }
        .key("binary-process")
        .w_full()
        .gap(if compact { 4.0 } else { 12.0 });
    let stages = [
        (
            "Original binary",
            format!(
                "Selected artifact · {:.2} MB",
                data["file_size"].as_u64().unwrap() as f64 / 1_000_000.0
            ),
            "catalog",
        ),
        ("Find one routine", routine_name(routine).to_owned(), "flow"),
        (
            "Rebuild in C",
            field(routine, "source")
                .rsplit('/')
                .next()
                .unwrap_or_default()
                .to_owned(),
            "tools",
        ),
        (
            "Compare outputs",
            format!("{} suite matches", routine["comparison_cases"]),
            "restart",
        ),
    ];
    for (index, (title, value, symbol)) in stages.into_iter().enumerate() {
        if index > 0 {
            flow = flow.child(secondary(if compact { "↓" } else { "→" }).self_center());
        }
        let title = row()
            .center_y()
            .gap(8.0)
            .child(rune::icon(workspace::icon(symbol), 20.0).color(color("--rb-color-secondary")))
            .child(secondary(title));
        let step = if compact { row().center_y() } else { col() }
            .key(format!("binary-stage-{index}"))
            .min_w(0.0)
            .gap(8.0)
            .padding(if compact { 12.0 } else { 16.0 })
            .fill(color("--rb-color-field"))
            .radius(size("--rb-radius-panel"))
            .child(title)
            .child(
                label(value)
                    .key(format!("binary-stage-value-{index}"))
                    .font(font("--rb-text-body", 500.0)),
            );
        flow = flow.child(if compact {
            step.w_full()
        } else {
            step.w(((width - 132.0) / 4.0).max(1.0)).grow(1.0)
        });
    }
    col()
        .w_full()
        .gap(12.0)
        .child(label("How a replacement is checked").font(font("--rb-text-body", 500.0)))
        .child(flow)
        .child(secondary(
            "Saved comparison-suite results, not whole-controller equivalence.",
        ))
}

fn map_legend() -> El {
    let mut legend = row().w_full().gap(24.0).flex_wrap();
    for (title, token) in [
        ("Code", "--rb-color-action"),
        ("Data & metadata", "--rb-color-control-edge"),
        ("Headers & gaps", "--rb-color-raised"),
        ("Selected routine", "--rb-color-info"),
    ] {
        legend = legend.child(
            row()
                .center_y()
                .gap(8.0)
                .child(el().size(12.0, 12.0).fill(color(token)))
                .child(secondary(title)),
        );
    }
    legend
}

fn binary(state: &State, cx: &Cx<State>, width: f32) -> El {
    let inspector = &state.ui.inspector;
    let mut content = col().w_full().gap(16.0);
    if let Some(error) = &inspector.error {
        content = content.child(
            label(format!(
                "{error}{}",
                if inspector.data.is_some() {
                    " Showing retained evidence; refresh failed."
                } else {
                    ""
                }
            ))
            .key("binary-error")
            .color(color("--rb-color-danger")),
        );
    }
    if state.host.selected.as_deref() != Some(catalog::DDCS_SHA) {
        return content
            .child(
                label("Open the target investigation for this system.").key("binary-unsupported"),
            )
            .child(secondary(
                "Continue with the selected system’s investigation.",
            ))
            .child(
                workspace::command("Open investigation", true)
                    .on_press(|state: &mut State| open_step(state, "target"))
                    .build(&theme::kit()),
            );
    }
    let Some(data) = inspector.data.as_ref() else {
        return content.child(
            label(if inspector.loading {
                "Reading saved binary evidence…"
            } else {
                "Binary evidence unavailable. Retry to read the selected host."
            })
            .key("binary-empty"),
        );
    };
    let routines = data["functions"].as_array().unwrap();
    let routine = &routines[inspector.routine];
    let measured_width = cx.layout_size("binary-body").map_or(width, |s| s[0]);
    content = content
        .child(
            label(format!(
                "Binary layout · {} file sections",
                data["sections"].as_array().unwrap().len()
            ))
            .font(font("--rb-text-body", 500.0)),
        )
        .child(section_map(data, routine, measured_width.max(1.0)))
        .child(map_legend())
        .child(process(data, routine, measured_width.max(1.0)));
    let mut toolbar = row().w_full().gap(8.0).flex_wrap();
    for (index, routine) in routines.iter().enumerate() {
        toolbar = toolbar.child(
            Button::new(routine_name(routine))
                .variant(if inspector.routine == index {
                    ButtonVariant::Contrast
                } else {
                    ButtonVariant::Outline
                })
                .on_press(move |state: &mut State| state.ui.inspector.select(index))
                .build(&theme::kit())
                .key(format!("routine-{index}"))
                .help("Inspect this saved routine; does not execute firmware or write hardware")
                .min_h(40.0),
        );
    }
    let mut inspection = col()
        .w_full()
        .gap(16.0)
        .child(toolbar)
        .child(
            label(field(routine, "label"))
                .key("routine-title")
                .font(font("--rb-text-body", 500.0)),
        )
        .child(secondary(format!(
            "ARM address {} · {} · file offset 0x{:x} · {} bytes",
            field(routine, "address"),
            field(routine, "section"),
            routine["file_offset"].as_u64().unwrap(),
            field(routine, "bytes").len() / 2
        )))
        .child(
            Collapsible::new("Original bytes", inspector.bytes)
                .on_toggle(|state: &mut State| state.ui.inspector.bytes = !state.ui.inspector.bytes)
                .child(code(
                    "Original bytes by ARM address",
                    &bytes_text(routine),
                    None,
                    "routine-bytes",
                ))
                .build(&theme::kit())
                .key("routine-bytes-disclosure"),
        );
    let mut method = col().w_full().gap(12.0)
        .child(label(field(routine, "why")))
        .child(label(field(routine, "how")))
        .child(secondary(format!("Proof: {} · SHA-256 {}. Original ARM and compiled C are compared with identical inputs; bounded checks are not full-program equivalence.", field(routine,"proof"),field(routine,"proof_sha256"))));
    if let Some(evidence) = routine.get("instruction_evidence") {
        let mut instructions = col().w_full().gap(12.0);
        for (index, instruction) in evidence["instructions"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            instructions = instructions
                .child(code(
                    field(instruction, "address"),
                    &format!(
                        "{}  {}",
                        field(instruction, "bytes"),
                        field(instruction, "instruction")
                    ),
                    None,
                    &format!("instruction-{index}"),
                ))
                .child(label(field(instruction, "meaning")));
        }
        instructions = instructions.child(secondary(format!(
            "{} {} · saved analysis, not a live instruction pointer. {} · {}",
            field(&evidence["provider"], "name"),
            field(&evidence["provider"], "version"),
            field(evidence, "path"),
            field(evidence, "evidence_id")
        )));
        method = method.child(
            Collapsible::new(
                "Original instructions: why this reconstruction",
                inspector.instructions,
            )
            .on_toggle(|state: &mut State| {
                state.ui.inspector.instructions = !state.ui.inspector.instructions
            })
            .child(instructions)
            .build(&theme::kit())
            .key("routine-instructions"),
        );
    }
    method = method.child(
        Collapsible::new("Recovered C and comparison scope", inspector.source)
            .on_toggle(|state: &mut State| state.ui.inspector.source = !state.ui.inspector.source)
            .child(
                col()
                    .w_full()
                    .gap(12.0)
                    .child(code(
                        field(routine, "source"),
                        field(routine, "recovered_c"),
                        Some("c"),
                        "routine-source",
                    ))
                    .child(secondary(format!(
                        "SHA-256 {} · {} matching cases in this comparison suite. {}",
                        field(routine, "source_sha256"),
                        routine["comparison_cases"],
                        field(routine, "proof_scope")
                    ))),
            )
            .build(&theme::kit())
            .key("routine-source-disclosure"),
    );
    inspection = inspection.child(
        Collapsible::new("Routine evidence", inspector.method)
            .on_toggle(|state: &mut State| state.ui.inspector.method = !state.ui.inspector.method)
            .child(method)
            .build(&theme::kit())
            .key("routine-evidence"),
    );
    content = content.child(
        Collapsible::new("Inspect routines", inspector.inspect)
            .on_toggle(|state: &mut State| state.ui.inspector.inspect = !state.ui.inspector.inspect)
            .child(inspection)
            .build(&theme::kit())
            .key("binary-inspect"),
    );
    content.child(Collapsible::new("File identity & limits", inspector.identity)
        .on_toggle(|state: &mut State| state.ui.inspector.identity = !state.ui.inspector.identity)
        .child(col().w_full().gap(8.0)
            .child(secondary(format!("Selected firmware SHA-256 · {}",field(data,"target_sha256"))).key("binary-identity"))
            .child(label(field(data,"scope")))
            .child(secondary("Saved file bytes and bounded comparisons; not live execution, a complete source rebuild or physical motion.")))
        .build(&theme::kit()).key("binary-identity-disclosure"))
}

fn overview(state: &State) -> El {
    let mut body = col()
        .w_full()
        .gap(16.0)
        .padding(size("--rb-gutter"))
        .child(label("Artifact overview").font(font("--rb-text-title", 500.0)));
    let Some(item) = workspace::selected(state) else {
        return body.child(secondary("Choose an artifact in Library."));
    };
    let info = catalog::info(item);
    body = body
        .child(label(format!("{} {}", info.vendor, info.name)))
        .child(secondary(info.purpose))
        .child(workspace::badge(
            if item["profile_id"] == "hardware-rea-example" {
                "Recorded summary"
            } else {
                "Local import"
            },
            "info",
        ));
    if let Some(error) = &state.ui.inspector.error {
        body = body.child(secondary(error));
    }
    if let Some(text) = state
        .ui
        .inspector
        .data
        .as_ref()
        .and_then(|data| data["overview"].as_str())
    {
        if item["profile_id"] == "hardware-rea-example" {
            if let Ok(summary) = serde_json::from_str::<Value>(text) {
                let measurement = &summary["reference_measurement_summary"];
                if let Some(mean) = measurement["frequency_hz"]["mean"].as_f64() {
                    body = body.child(label(format!(
                        "{} captures · {mean:.3} Hz mean",
                        measurement["captures"]
                    )));
                }
                body = body.child(
                    Collapsible::new("Findings & limits", state.ui.inspector.method)
                        .on_toggle(|state: &mut State| {
                            state.ui.inspector.method = !state.ui.inspector.method
                        })
                        .child(col().gap(8.0)
                            .child(secondary("ARMv5 / Linux · temporary display drawing and normal return observed"))
                            .child(secondary("Persistent firmware modification not demonstrated. Voltage scaling unqualified."))
                            .child(secondary(format!("Frequency sample SD · {} Hz", measurement["frequency_hz"]["sample_standard_deviation"]))))
                        .build(&theme::kit()),
                );
            }
        } else {
            body = body.child(
                Collapsible::new("File preview · first 128 bytes", state.ui.inspector.bytes)
                    .on_toggle(|state: &mut State| {
                        state.ui.inspector.bytes = !state.ui.inspector.bytes
                    })
                    .child(text_element(text))
                    .build(&theme::kit()),
            );
        }
    } else if state.ui.inspector.loading {
        body = body.child(secondary("Reading local evidence…"));
    }
    if item["profile_id"] == "hardware-rea-example" || item["artifact_type"] == "dotdog" {
        body = body.child(
            row().w_full().child(
                workspace::command("Evidence map", false)
                    .on_press(|state: &mut State| {
                        state.ui.inspector.step = 6;
                        super::graph::refresh(state);
                    })
                    .build(&theme::kit())
                    .w(180.0)
                    .h(40.0),
            ),
        );
    }
    body = body.child(
        Collapsible::new("Artifact identity", state.ui.inspector.identity)
            .on_toggle(|state: &mut State| {
                state.ui.inspector.identity = !state.ui.inspector.identity
            })
            .child(secondary(format!(
                "SHA-256 · {}",
                catalog::value(item, "case_id", "")
            )))
            .build(&theme::kit()),
    );
    col()
        .w_full()
        .h_full()
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .child(body)
}

fn text_element(value: &str) -> El {
    text(value).font(font("--rb-text-secondary", 400.0).mono())
}

pub fn view(state: &State, cx: &Cx<State>, width: f32) -> El {
    if state.host.selected.as_deref() != Some(catalog::DDCS_SHA) && state.ui.inspector.step == 6 {
        let sidebar = if width <= 1000.0 {
            0.0
        } else if state.ui.collapsed {
            56.0
        } else {
            192.0
        };
        let available = cx
            .layout_size("evidence-map-panel")
            .map_or(width - sidebar, |size| size[0]);
        return col()
            .key("evidence-map-panel")
            .w_full()
            .min_w(0.0)
            .h_full()
            .scroll(ScrollSpec {
                stick_bottom: false,
            })
            .padding(size("--rb-gutter"))
            .child(
                workspace::command("Artifact overview", false)
                    .on_press(|state: &mut State| state.ui.inspector.step = 0)
                    .build(&theme::kit()),
            )
            .child(super::graph::view(state, cx, (available - 48.0).max(1.0)));
    }

    if state.host.selected.as_deref() != Some(catalog::DDCS_SHA) {
        return overview(state);
    }
    let narrow = width <= 1000.0;
    let inspector = &state.ui.inspector;
    let loading = if inspector.step == 1 {
        state.ui.controller.loading()
    } else if matches!(inspector.step, 6 | 7) {
        state.ui.graph.loading()
    } else {
        inspector.loading
    };
    let heading = row()
        .w_full()
        .center_y()
        .gap(12.0)
        .flex_wrap()
        .child(label(STEPS[inspector.step].1).font(font("--rb-text-title", 500.0)))
        .child(workspace::badge("Saved evidence", "info"))
        .child(
            workspace::command(if loading { "Reading…" } else { "Refresh" }, false)
                .disabled(
                    loading
                        || (inspector.step == 0
                            && state.host.selected.as_deref() != Some(catalog::DDCS_SHA)),
                )
                .on_press(refresh)
                .build(&theme::kit())
                .key("binary-refresh")
                .min_h(40.0),
        );
    let mut body = col()
        .key("binary-body")
        .w_full()
        .min_w(0.0)
        .gap(16.0)
        .child(heading);
    if narrow {
        body = body.child(
            Collapsible::new("Investigation steps", inspector.steps)
                .on_toggle(|state: &mut State| state.ui.inspector.steps = !state.ui.inspector.steps)
                .child(steps(state))
                .build(&theme::kit()),
        );
    }
    let content_width = cx.layout_size("binary-body").map_or(
        (width - if narrow { 48.0 } else { 448.0 }).max(1.0),
        |size| size[0],
    );
    body = body.child(if inspector.step == 1 {
        super::controller::view(state, cx, content_width)
    } else if matches!(inspector.step, 6 | 7) {
        super::graph::view(state, cx, content_width)
    } else {
        binary(state, cx, content_width)
    });
    let content = col()
        .grow(1.0)
        .min_w(0.0)
        .h_full()
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .padding(size("--rb-gutter"))
        .child(body);
    if narrow {
        content
    } else {
        row()
            .w_full()
            .grow(1.0)
            .min_h(0.0)
            .child(
                col()
                    .w(208.0)
                    .h_full()
                    .pt(16.0)
                    .fill(color("--rb-color-surface"))
                    .child(
                        secondary("Investigation")
                            .key("rea-investigation-title")
                            .px(16.0)
                            .py(8.0),
                    )
                    .child(steps(state)),
            )
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn imported_file_preview_renders_with_shared_design_tokens() {
        let mut state = State::idle();
        let id = "a".repeat(64);
        state.page = host::Page::Rea;
        state.host.selected = Some(id.clone());
        state.host.snapshot = Some(host::Snapshot {
            health: json!({}),
            recovery: json!({}),
            cases: vec![json!({"case_id":id,"library":{"model":"example.json"}})],
        });
        state.ui.inspector.data = Some(json!({"overview":"7b 22 76 65 72 73 69 6f 6e"}));
        state.ui.inspector.bytes = true;
        let mut runtime = super::super::app(state).headless_runtime(
            (1000.0, 760.0),
            rune::Env {
                system_dark: true,
                scale: 2.0,
            },
        );
        runtime.frame(0.0);
        assert!(
            runtime
                .elements()
                .iter()
                .any(|element| { element.text.as_deref() == Some("7b 22 76 65 72 73 69 6f 6e") })
        );
    }
}
