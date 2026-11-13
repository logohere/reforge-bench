use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use rune::{El, FileDialogOptions, ScrollSpec, TextEdit, Tx, col, row};
use rune_kit::{Button, ButtonVariant, Collapsible, Select, State as ControlState, TextInput};
use serde_json::Value;

use super::{State, catalog, host::Page, label, secondary, theme, workspace};
use catalog::{info, value};
use theme::{color, font, size};
use workspace::{badge, command, icon};

const ALL: [&str; 4] = [
    "All manufacturers",
    "All types",
    "Any support",
    "Recovery evidence first",
];

pub struct Library {
    pub search: TextEdit,
    pub filters: bool,
    pub choices: [String; 4],
    pub open_select: Option<String>,
    pub expanded: HashMap<String, bool>,
    pub files: bool,
    pub evidence: bool,
    pub provenance: bool,
    pub import: Option<PathBuf>,
    pub metadata: bool,
    pub labels: [TextEdit; 3],
    pub machine_type: String,
    pub parent: String,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            search: TextEdit::default(),
            filters: false,
            choices: ALL.map(str::to_owned),
            open_select: None,
            expanded: HashMap::new(),
            files: false,
            evidence: false,
            provenance: false,
            import: None,
            metadata: false,
            labels: std::array::from_fn(|_| TextEdit::default()),
            machine_type: "Use profile / unknown".into(),
            parent: "This is an original image".into(),
        }
    }
}

impl Library {
    pub fn clear(&mut self) {
        self.search = TextEdit::default();
        self.choices = ALL.map(str::to_owned);
    }
    fn dirty(&self) -> bool {
        !self.search.text.is_empty() || self.choices.iter().zip(ALL).any(|(a, b)| a != b)
    }
}

fn natural(a: &str, b: &str) -> std::cmp::Ordering {
    let a = a.to_lowercase();
    let b = b.to_lowercase();
    let (mut a, mut b) = (a.chars().peekable(), b.chars().peekable());
    loop {
        let (Some(&ac), Some(&bc)) = (a.peek(), b.peek()) else {
            return a.count().cmp(&b.count());
        };
        if ac.is_ascii_digit() && bc.is_ascii_digit() {
            let an: String = std::iter::from_fn(|| a.next_if(char::is_ascii_digit)).collect();
            let bn: String = std::iter::from_fn(|| b.next_if(char::is_ascii_digit)).collect();
            let an = an.trim_start_matches('0');
            let bn = bn.trim_start_matches('0');
            let cmp = an.len().cmp(&bn.len()).then_with(|| an.cmp(bn));
            if !cmp.is_eq() {
                return cmp;
            }
        } else {
            a.next();
            b.next();
            let cmp = ac.cmp(&bc);
            if !cmp.is_eq() {
                return cmp;
            }
        }
    }
}

pub fn groups<'a>(cases: &'a [Value], recovery: &Value, library: &Library) -> Vec<Vec<&'a Value>> {
    let query = library.search.text.trim().to_lowercase();
    let mut visible: Vec<_> = cases
        .iter()
        .filter(|item| {
            let meta = &item["library"];
            let haystack = meta
                .as_object()
                .into_iter()
                .flat_map(|v| v.values())
                .filter_map(Value::as_str)
                .chain([
                    value(item, "original_filename", ""),
                    value(item, "case_id", ""),
                ])
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase();
            (library.choices[0] == ALL[0] || meta["vendor"] == library.choices[0])
                && (library.choices[1] == ALL[1]
                    || catalog::machine_type(value(meta, "machine_type", "unknown"))
                        == library.choices[1])
                && (library.choices[2] == ALL[2]
                    || catalog::has_adapter(item) == (library.choices[2] == "Available"))
                && haystack.contains(&query)
        })
        .collect();
    visible.sort_by(|a, b| {
        let cmp = match library.choices[3].as_str() {
            "Recovery evidence first" => catalog::progress(b, recovery)
                .0
                .cmp(&catalog::progress(a, recovery).0),
            "Recently added" => b["imported_at_ms"]
                .as_i64()
                .unwrap_or(0)
                .cmp(&a["imported_at_ms"].as_i64().unwrap_or(0)),
            "Emulator available first" => {
                (a["emulation"] == "unsupported").cmp(&(b["emulation"] == "unsupported"))
            }
            "Manufacturer A–Z" => natural(
                value(&a["library"], "vendor", ""),
                value(&b["library"], "vendor", ""),
            ),
            _ => natural(
                value(&a["library"], "model", ""),
                value(&b["library"], "model", ""),
            ),
        };
        cmp.then_with(|| {
            natural(
                value(&a["library"], "vendor", ""),
                value(&b["library"], "vendor", ""),
            )
        })
        .then_with(|| {
            natural(
                value(&a["library"], "model", ""),
                value(&b["library"], "model", ""),
            )
        })
        .then_with(|| {
            natural(
                value(&b["library"], "version", ""),
                value(&a["library"], "version", ""),
            )
        })
        .then_with(|| natural(value(a, "case_id", ""), value(b, "case_id", "")))
    });
    let mut groups: Vec<Vec<&Value>> = Vec::new();
    for item in visible {
        let group = info(item).group;
        if let Some(existing) = groups
            .iter_mut()
            .find(|items| info(items[0]).group == group)
        {
            existing.push(item);
        } else {
            groups.push(vec![item]);
        }
    }
    groups
}

pub fn choose_file(state: &mut State, tx: &mut Tx) {
    state.page = Page::Library;
    if state.host.loading {
        return;
    }
    tx.open_file(
        FileDialogOptions {
            title: "Import artifact".into(),
            ..Default::default()
        },
        |state: &mut State, result, _| match result {
            Ok(Some(path)) => {
                state.ui.library.import = Some(path);
                state.ui.library.metadata = false;
                state.host.error = None;
            }
            Ok(None) => {}
            Err(error) => state.host.error = Some(format!("Could not choose firmware: {error}")),
        },
    );
}

fn select(
    state: &State,
    key: &str,
    title: &str,
    options: &[String],
    current: &str,
    layers: &mut Vec<El>,
) -> El {
    let key = key.to_owned();
    let toggle = key.clone();
    let choose = key.clone();
    Select::new(key.clone(), options)
        .label(title)
        .current(Some(current))
        .open(state.ui.library.open_select.as_deref() == Some(&key))
        .on_toggle(move |state: &mut State| {
            state.ui.library.open_select =
                if state.ui.library.open_select.as_deref() == Some(&toggle) {
                    None
                } else {
                    Some(toggle.clone())
                };
        })
        .on_close(|state: &mut State| state.ui.library.open_select = None)
        .on_choose(move |state: &mut State, option: &str, _| {
            match choose.as_str() {
                "vendor" => state.ui.library.choices[0] = option.into(),
                "type" => state.ui.library.choices[1] = option.into(),
                "support" => state.ui.library.choices[2] = option.into(),
                "sort" => state.ui.library.choices[3] = option.into(),
                "import-type" => state.ui.library.machine_type = option.into(),
                "import-parent" => state.ui.library.parent = option.into(),
                _ => {}
            }
            state.ui.library.open_select = None;
        })
        .build(&theme::kit(), layers)
        .h(size("--rb-control-height"))
        .w_full()
}

fn filter_fields(state: &State, narrow: bool, layers: &mut Vec<El>) -> El {
    let library = &state.ui.library;
    let mut vendors = vec![ALL[0].to_owned()];
    if let Some(snapshot) = &state.host.snapshot {
        vendors.extend(
            snapshot
                .cases
                .iter()
                .filter_map(|v| v["library"]["vendor"].as_str())
                .map(str::to_owned)
                .collect::<BTreeSet<_>>(),
        );
    }
    let values = [
        vendors,
        [
            "All types",
            "CNC",
            "3D printer",
            "Robot arm",
            "Instrument",
            "Unknown type",
        ]
        .map(str::to_owned)
        .to_vec(),
        ["Any support", "Available", "No adapter"]
            .map(str::to_owned)
            .to_vec(),
        [
            "Recovery evidence first",
            "Manufacturer A–Z",
            "Model A–Z",
            "Recently added",
            "Emulator available first",
        ]
        .map(str::to_owned)
        .to_vec(),
    ];
    let mut fields = if narrow { col() } else { row() };
    fields = fields.gap(16.0).w_full();
    for (index, (key, title)) in [
        ("vendor", "Manufacturer"),
        ("type", "Machine type"),
        ("support", "System emulator"),
        ("sort", "Sort"),
    ]
    .into_iter()
    .enumerate()
    {
        fields = fields.child(
            col()
                .min_w(0.0)
                .grow(1.0)
                .gap(8.0)
                .child(secondary(title))
                .child(select(
                    state,
                    key,
                    title,
                    &values[index],
                    &library.choices[index],
                    layers,
                )),
        );
    }
    fields
        .padding(16.0)
        .fill(color("--rb-color-surface"))
        .border(1.0, color("--rb-color-divider"))
}

fn file_options(state: &State, item: &Value, group: &[&Value]) -> El {
    let mut files=col().w_full().gap(8.0).child(secondary("Internal components retained for investigation. Selecting a file does not run the controller."));
    for artifact in group {
        let id = value(artifact, "case_id", "").to_owned();
        let info = info(artifact);
        let name = if artifact["original_filename"] == "motiondev.ko" {
            value(&artifact["library"], "model", "")
        } else {
            value(artifact, "original_filename", "")
        }
        .to_owned();
        let component = info.component.to_owned();
        let format = format!("{} · {} bytes", info.format, artifact["firmware_bytes"]);
        files = files.child(
            command(name.clone(), false)
                .label_with(move |_, _| {
                    col()
                        .w_full()
                        .gap(6.0)
                        .child(label(name).font(font("--rb-text-secondary", 500.0)))
                        .child(secondary(component))
                        .child(secondary(format))
                })
                .on_press(move |state: &mut State| {
                    state.select_system(id.clone());
                    state.ui.library.provenance = false;
                    state.ui.library.evidence = false;
                })
                .build(&theme::kit())
                .key(format!("artifact-{}", value(artifact, "case_id", "")))
                .w_full()
                .min_h(96.0)
                .border(
                    if artifact["case_id"] == item["case_id"] {
                        1.0
                    } else {
                        0.0
                    },
                    color("--rb-color-selection-edge"),
                ),
        );
    }
    Collapsible::new(
        format!("{} package files", group.len()),
        state.ui.library.files,
    )
    .on_toggle(|state: &mut State| state.ui.library.files = !state.ui.library.files)
    .child(files)
    .build(&theme::kit())
}

fn property(name: &str, content: impl Into<String>, mono: bool) -> El {
    row()
        .w_full()
        .gap(16.0)
        .child(secondary(name).w(112.0))
        .child(
            secondary(content)
                .font(if mono {
                    font("--rb-text-caption", 400.0).mono()
                } else {
                    font("--rb-text-secondary", 400.0)
                })
                .wrap_anywhere()
                .grow(1.0)
                .min_w(0.0),
        )
}

fn details(state: &State, item: &Value, group: &[&Value]) -> El {
    let snapshot = state.host.snapshot.as_ref().unwrap();
    let info = info(item);
    let meta = &item["library"];
    let record = catalog::recovery(item, &snapshot.recovery);
    let support = catalog::support(item, &snapshot.recovery);
    let haas = support == "Bounded Haas research";
    let supported = support != "Investigation only";
    let mut details = col()
        .w_full()
        .gap(12.0)
        .child(
            secondary(format!(
                "{} · {}",
                info.vendor,
                catalog::machine_type(value(meta, "machine_type", "unknown"))
            ))
            .font(font("--rb-text-caption", 400.0)),
        )
        .child(
            label(
                info.short_name
                    .unwrap_or(info.name)
                    .strip_prefix("Haas ")
                    .unwrap_or(info.short_name.unwrap_or(info.name)),
            )
            .font(font("--rb-text-title", 500.0))
            .key("case-model"),
        )
        .child(secondary(info.version))
        .child(label(
            if haas && item["profile_id"] != "haas-coldfire2-loader" {
                "Independent machine preview; original firmware startup is partial."
            } else {
                info.purpose
            },
        ));
    let mut actions = row().gap(8.0).flex_wrap().py(12.0);
    if supported {
        actions = actions.child(workspace::route_button("Open Lab", "lab", true));
    }
    actions = actions.child(workspace::route_button("Open REA", "rea", !supported));
    if let Some(map) = record
        .and_then(|r| r["map"].as_str())
        .filter(|map| map.starts_with("/process/") && !map.starts_with("//"))
    {
        let path = map.to_owned();
        actions = actions.child(
            command("Recovery map", false)
                .help("Open recovery map inside the app")
                .on_press(move |state: &mut State| workspace::open_path(state, &path))
                .build(&theme::kit())
                .h(44.0),
        );
    }
    details = details.child(actions);
    let fallback = serde_json::json!([
        ["Acquire", "Saved", "info"],
        ["Analyze", "Unrecorded", "secondary"],
        ["Compare", "Unrecorded", "secondary"],
        [
            "Execute",
            if item["emulation"] == "recorded" {
                "Startup saved"
            } else if catalog::has_adapter(item) {
                "Not run"
            } else {
                "No adapter"
            },
            "secondary"
        ],
        ["Hardware", "Unverified", "secondary"]
    ]);
    let milestones = record
        .map(|r| &r["milestones"])
        .filter(|v| v.is_array())
        .unwrap_or(&fallback);
    let mut stages = row().w_full().gap(12.0).flex_wrap();
    for stage in milestones.as_array().unwrap() {
        stages = stages.child(
            col()
                .gap(8.0)
                .child(secondary(stage[0].as_str().unwrap_or("Unknown")))
                .child(badge(
                    stage[1].as_str().unwrap_or("Unrecorded"),
                    stage[2].as_str().unwrap_or("secondary"),
                )),
        );
    }
    let mut evidence = col().w_full().gap(16.0).py(12.0).child(stages);
    if haas {
        evidence=evidence.child(badge("Original firmware is partial","warning")).child(secondary(if item["profile_id"]=="haas-coldfire2-loader"{"A bounded loader helper is verified. Complete loader boot is unavailable."}else{"Original startup stops at an unmodeled device. The independent machine preview does not run OEM motion."}));
    }
    evidence = evidence.child(
        label(
            record
                .map(|r| value(r, "headline", "Retained artifact"))
                .unwrap_or("Retained artifact"),
        )
        .font(font("--rb-text-body", 500.0)),
    );
    if let Some(stages) = record.and_then(|r| r["stages"].as_array()) {
        for stage in stages {
            evidence = evidence.child(property(
                stage[0].as_str().unwrap_or(""),
                stage[1].as_str().unwrap_or(""),
                false,
            ));
        }
    } else {
        evidence = evidence
            .child(property(
                "Acquired",
                "Original imported bytes retained",
                false,
            ))
            .child(property(
                "Analyzed",
                "No analysis result recorded for this artifact",
                false,
            ));
    }
    evidence=evidence.child(secondary(record.map(|r|value(r,"proof","Original bytes retained. Analysis, execution and hardware acceptance require their own evidence.")).unwrap_or("Original bytes retained. Analysis, execution and hardware acceptance require their own evidence.")));
    if let Some(gaps) = record.and_then(|r| r["gaps"].as_str()) {
        evidence = evidence.child(secondary(gaps));
    }
    details = details.child(
        Collapsible::new("Evidence & limits", state.ui.library.evidence)
            .on_toggle(|state: &mut State| state.ui.library.evidence = !state.ui.library.evidence)
            .child(evidence)
            .build(&theme::kit()),
    );
    if group.len() > 1 {
        details = details.child(file_options(state, item, group));
    }
    let execution = if haas {
        if item["profile_id"] == "haas-coldfire2-loader" {
            "Bounded loader helper only"
        } else {
            "Bounded ColdFire startup · video device blocked"
        }
    } else {
        match value(item, "emulation", "") {
            "unsupported" => "No emulator adapter",
            "recorded" => "Startup report saved",
            "not_run" => "Adapter available · not run",
            "failed" => "Last emulation failed",
            _ => "Not established",
        }
    };
    let mut provenance = col().w_full().gap(12.0).py(12.0);
    for (name, content, mono) in [
        ("Component", info.component.to_owned(), false),
        ("File format", info.format.into(), false),
        (
            "CPU",
            value(meta, "architecture", "Not established").into(),
            false,
        ),
        ("Execution", execution.into(), false),
        ("Hardware", "No recorded hardware acceptance".into(), false),
        ("File", value(item, "original_filename", "").into(), false),
        ("Size", format!("{} bytes", item["firmware_bytes"]), false),
        (
            "Image",
            if item["kind"] == "modified" {
                "Modified copy"
            } else {
                "Original import"
            }
            .into(),
            false,
        ),
        ("SHA-256", value(item, "case_id", "").into(), true),
        ("Parent", value(item, "parent_case_id", "").into(), true),
        (
            "Profile",
            value(item, "profile_id", "No exact-hash profile").into(),
            true,
        ),
    ] {
        if !content.is_empty() {
            provenance = provenance.child(property(name, content, mono));
        }
    }
    provenance = provenance.child(secondary(value(
        item,
        "verified_scope",
        "Import and labeling only. No verified analysis or emulator workspace.",
    )));
    if let Some(notes) = meta["notes"].as_str() {
        provenance = provenance.child(secondary(notes));
    }
    details = details.child(
        Collapsible::new("File details & provenance", state.ui.library.provenance)
            .on_toggle(|state: &mut State| {
                state.ui.library.provenance = !state.ui.library.provenance
            })
            .child(provenance)
            .build(&theme::kit()),
    );
    if let Some(source) = meta["source_url"]
        .as_str()
        .filter(|url| url.starts_with("https://") || url.starts_with("http://"))
    {
        let source = source.to_owned();
        details = details.child(
            Button::new("Acquisition source")
                .variant(ButtonVariant::Ghost)
                .help(&source)
                .on_press(move |state: &mut State| workspace::open_url(state, &source))
                .build(&theme::kit())
                .h(40.0)
                .self_start(),
        );
    }
    details
}

fn import_panel(state: &State, narrow: bool, layers: &mut Vec<El>) -> Option<El> {
    let library = &state.ui.library;
    let path = library.import.as_ref()?;
    let mut panel = col()
        .w_full()
        .padding(24.0)
        .gap(16.0)
        .fill(color("--rb-color-surface"))
        .child(label("Artifact file").font(font("--rb-text-secondary", 500.0)))
        .child(secondary(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        ))
        .child(secondary(
            "Up to 64 MiB. Importing does not execute or modify firmware.",
        ));
    let mut fields = col().w_full().gap(16.0);
    for (index, name) in ["Brand", "Model", "Version"].into_iter().enumerate() {
        fields = fields.child(
            col().w_full().gap(8.0).child(secondary(name)).child(
                TextInput::new(&library.labels[index])
                    .label(name)
                    .height(40.0)
                    .on_edit(move |state: &mut State| &mut state.ui.library.labels[index])
                    .build(&theme::kit())
                    .key(format!("import-label-{index}")),
            ),
        );
    }
    fields = fields.child(secondary("Machine type")).child(select(
        state,
        "import-type",
        "Machine type",
        &[
            "Use profile / unknown",
            "CNC",
            "3D printer",
            "Robot arm",
            "Instrument",
        ]
        .map(str::to_owned),
        &library.machine_type,
        layers,
    ));
    let mut parents = vec!["This is an original image".to_owned()];
    if let Some(snapshot) = &state.host.snapshot {
        parents.extend(snapshot.cases.iter().map(|item| {
            format!(
                "{} · {}",
                value(
                    &item["library"],
                    "model",
                    value(item, "original_filename", "")
                ),
                &value(item, "case_id", "")[..12]
            )
        }));
    }
    fields = fields
        .child(secondary("Original for a modified copy"))
        .child(select(
            state,
            "import-parent",
            "Original for a modified copy",
            &parents,
            &library.parent,
            layers,
        ));
    panel = panel.child(
        Collapsible::new("Optional labels", library.metadata)
            .on_toggle(|state: &mut State| state.ui.library.metadata = !state.ui.library.metadata)
            .child(fields)
            .build(&theme::kit()),
    );
    let actions = if narrow { col() } else { row() };
    panel = panel.child(
        actions
            .gap(12.0)
            .child(
                command(
                    if state.host.loading {
                        "Importing…"
                    } else {
                        "Import & inspect"
                    },
                    true,
                )
                .state(if state.host.loading {
                    ControlState::Disabled
                } else {
                    ControlState::Rest
                })
                .on_press(|state: &mut State| {
                    let library = &state.ui.library;
                    let Some(path) = library.import.clone() else {
                        return;
                    };
                    let mut labels = Vec::new();
                    for (index, key) in ["vendor", "model", "version"].into_iter().enumerate() {
                        let text = library.labels[index].text.trim();
                        if !text.is_empty() {
                            labels.push((key.to_owned(), text.to_owned()));
                        }
                    }
                    let machine_type = match library.machine_type.as_str() {
                        "CNC" => "cnc",
                        "3D printer" => "3d-printer",
                        "Robot arm" => "robot-arm",
                        "Instrument" => "instrument",
                        _ => "",
                    };
                    if !machine_type.is_empty() {
                        labels.push(("machine_type".into(), machine_type.into()));
                    }
                    if library.parent != "This is an original image" {
                        let parent = state.host.snapshot.as_ref().and_then(|snapshot| {
                            snapshot.cases.iter().find(|item| {
                                format!(
                                    "{} · {}",
                                    value(
                                        &item["library"],
                                        "model",
                                        value(item, "original_filename", "")
                                    ),
                                    &value(item, "case_id", "")[..12]
                                ) == library.parent
                            })
                        });
                        let Some(parent) = parent else {
                            state.host.error = Some(
                                "Original image is no longer available. Choose it again.".into(),
                            );
                            return;
                        };
                        labels.push(("parent".into(), value(parent, "case_id", "").into()));
                    }
                    state.host.import(path, labels);
                })
                .build(&theme::kit())
                .key("import-submit")
                .h(44.0),
            )
            .child(
                command("Choose another file", false)
                    .on_press_tx(choose_file)
                    .build(&theme::kit())
                    .h(44.0),
            )
            .child(
                Button::new("Cancel")
                    .variant(ButtonVariant::Ghost)
                    .on_press(|state: &mut State| state.ui.library.import = None)
                    .build(&theme::kit())
                    .h(44.0),
            ),
    );
    Some(panel)
}

pub fn view(state: &State, width: f32, layers: &mut Vec<El>) -> El {
    let narrow = width <= 780.0;
    let library = &state.ui.library;
    let kit = theme::kit();
    let mut heading = if narrow {
        col()
    } else {
        row().center_y().justify_between()
    };
    heading = heading
        .w_full()
        .gap(16.0)
        .child(label("Artifact library").font(font("--rb-text-title", 500.0)))
        .child(
            row()
                .gap(8.0)
                .flex_wrap()
                .child(
                    command("Import artifact", false)
                        .icon(icon("upload"))
                        .state(if state.host.loading {
                            ControlState::Disabled
                        } else {
                            ControlState::Rest
                        })
                        .on_press_tx(choose_file)
                        .build(&kit)
                        .key("import-open")
                        .h(40.0),
                )
                .child(
                    Button::new(if state.host.loading {
                        "Refreshing…"
                    } else {
                        "Refresh"
                    })
                    .variant(ButtonVariant::Ghost)
                    .icon(icon("restart"))
                    .state(if state.host.loading {
                        ControlState::Disabled
                    } else {
                        ControlState::Rest
                    })
                    .on_press(|state: &mut State| state.host.refresh())
                    .build(&kit)
                    .key("host-refresh")
                    .h(40.0),
                ),
        );
    let mut root = col()
        .w_full()
        .h_full()
        .min_h(0.0)
        .padding(24.0)
        .gap(16.0)
        .child(heading);
    if let Some(panel) = import_panel(state, narrow, layers) {
        root = root.child(panel);
    }
    root = root.child(
        row()
            .w_full()
            .gap(16.0)
            .items_end()
            .child(
                col()
                    .grow(1.0)
                    .min_w(0.0)
                    .max_w(720.0)
                    .gap(8.0)
                    .child(secondary("Search systems"))
                    .child(
                        TextInput::new(&library.search)
                            .label("Search systems")
                            .placeholder("Manufacturer, model or version")
                            .height(40.0)
                            .on_edit(|state: &mut State| &mut state.ui.library.search)
                            .build(&kit)
                            .key("library-search")
                            .w_full(),
                    ),
            )
            .child(
                command("Filters", false)
                    .on_press(|state: &mut State| {
                        state.ui.library.filters = !state.ui.library.filters
                    })
                    .build(&kit)
                    .key("library-filters")
                    .h(40.0),
            ),
    );
    if library.filters {
        root = root.child(filter_fields(state, narrow, layers));
    }
    let Some(snapshot) = &state.host.snapshot else {
        return root.child(secondary(if state.host.loading {
            "Loading imported firmware…"
        } else {
            "Library unavailable. Check the local server, then Refresh."
        }));
    };
    let groups = groups(&snapshot.cases, &snapshot.recovery, library);
    let brands: Vec<_> =
        groups
            .iter()
            .map(|group| info(group[0]).vendor)
            .fold(Vec::new(), |mut vendors, vendor| {
                if !vendors
                    .iter()
                    .any(|v: &&str| v.eq_ignore_ascii_case(vendor))
                {
                    vendors.push(vendor);
                }
                vendors
            });
    let visible = groups.iter().map(Vec::len).sum::<usize>();
    root = root.child(
        row()
            .w_full()
            .justify_between()
            .center_y()
            .gap(16.0)
            .child(
                secondary(format!(
                    "{} brands · {} systems · {visible} of {} files",
                    brands.len(),
                    groups.len(),
                    snapshot.cases.len()
                ))
                .font(font("--rb-text-caption", 400.0))
                .key("library-status"),
            )
            .child(
                Button::new("Clear filters")
                    .variant(ButtonVariant::Ghost)
                    .state(if library.dirty() {
                        ControlState::Rest
                    } else {
                        ControlState::Disabled
                    })
                    .on_press(|state: &mut State| state.ui.library.clear())
                    .build(&kit)
                    .key("library-clear")
                    .h(32.0),
            ),
    );
    let mut list = col().w_full().min_h(0.0).scroll(ScrollSpec {
        stick_bottom: false,
    });
    for vendor in brands {
        let brand_groups: Vec<_> = groups
            .iter()
            .filter(|group| info(group[0]).vendor.eq_ignore_ascii_case(vendor))
            .collect();
        let count = brand_groups.iter().map(|group| group.len()).sum::<usize>();
        let open = library.expanded.get(vendor).copied().unwrap_or_else(|| {
            !library.search.text.trim().is_empty()
                || brand_groups.iter().any(|group| {
                    group
                        .iter()
                        .any(|item| item["case_id"].as_str() == state.host.selected.as_deref())
                })
        });
        let mut family_body = col().w_full();
        let mut families = Vec::new();
        for group in &brand_groups {
            let family = info(group[0]).family;
            if !families.contains(&family) {
                families.push(family);
            }
        }
        for family in families {
            family_body = family_body.child(secondary(family).px(16.0).py(8.0));
            for group in brand_groups
                .iter()
                .filter(|group| info(group[0]).family == family)
            {
                let item = group[0];
                let info = info(item);
                let active = group
                    .iter()
                    .any(|item| item["case_id"].as_str() == state.host.selected.as_deref());
                let id = value(item, "case_id", "").to_owned();
                let name = info
                    .short_name
                    .unwrap_or(info.name)
                    .strip_prefix("Haas ")
                    .unwrap_or(info.short_name.unwrap_or(info.name))
                    .to_owned();
                let version = info.version.to_owned();
                let purpose = info.purpose.to_owned();
                let files = group.len();
                let progress = group
                    .iter()
                    .map(|item| catalog::progress(item, &snapshot.recovery))
                    .max_by_key(|v| v.0)
                    .unwrap();
                family_body = family_body.child(
                    Button::new(name.clone())
                        .variant(ButtonVariant::Ghost)
                        .label_with(move |_, _| {
                            col()
                                .w_full()
                                .min_w(0.0)
                                .gap(8.0)
                                .child(
                                    row()
                                        .w_full()
                                        .justify_between()
                                        .gap(8.0)
                                        .child(
                                            secondary(version)
                                                .font(font("--rb-text-caption", 400.0)),
                                        )
                                        .child(
                                            secondary(format!(
                                                "{files} {}",
                                                if files == 1 { "file" } else { "files" }
                                            ))
                                            .font(font("--rb-text-caption", 400.0)),
                                        ),
                                )
                                .child(label(name).font(font("--rb-text-body", 500.0)))
                                .child(secondary(purpose))
                                .child(badge(progress.1, progress.2))
                        })
                        .on_press(move |state: &mut State| {
                            if !active {
                                state.select_system(id.clone());
                                state.ui.library.files = false;
                                state.ui.library.evidence = false;
                                state.ui.library.provenance = false;
                            }
                        })
                        .build(&kit)
                        .key(format!("case-{}", value(item, "case_id", "")))
                        .w_full()
                        .min_h(142.0)
                        .radius(0.0)
                        .padding(16.0)
                        .fill(color(if active {
                            "--rb-color-surface"
                        } else {
                            "--rb-color-canvas"
                        }))
                        .border_sides(
                            [0.0, 0.0, 1.0, 3.0],
                            color(if active {
                                "--rb-color-selection-edge"
                            } else {
                                "--rb-color-divider"
                            }),
                        ),
                );
            }
        }
        let vendor = vendor.to_owned();
        let key = vendor.clone();
        list = list.child(
            Collapsible::new(format!("{vendor} · {count} files"), open)
                .on_toggle(move |state: &mut State| {
                    state.ui.library.expanded.insert(vendor.clone(), !open);
                })
                .child(family_body)
                .build(&kit)
                .key(format!("brand-{key}")),
        );
    }
    if groups.is_empty() {
        list = list.child(
            secondary(if snapshot.cases.is_empty() {
                "No artifacts imported. Choose Import artifact to add a file."
            } else {
                "No firmware matches these filters. Clear filters to see all systems."
            })
            .padding(24.0),
        );
    }
    let selected = groups.iter().find_map(|group| {
        group
            .iter()
            .find(|item| item["case_id"].as_str() == state.host.selected.as_deref())
            .map(|item| (*item, group))
    });
    let detail = selected
        .map(|(item, group)| details(state, item, group))
        .unwrap_or_else(|| {
            col().child(secondary(if snapshot.cases.is_empty() {
                "Imported firmware details appear here."
            } else {
                "Select a matching system to inspect its firmware."
            }))
        });
    let mut body = if narrow { col() } else { row() };
    body = body
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .gap(32.0)
        .pt(16.0)
        .border_sides([1.0, 0.0, 0.0, 0.0], color("--rb-color-divider"));
    body = body
        .child(if narrow {
            list.max_h(320.0).min_h(160.0)
        } else {
            list.w(if width < 1100.0 { 260.0 } else { 340.0 }).h_full()
        })
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .min_h(0.0)
                .w_full()
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(detail),
        );
    root = root.child(body);
    if narrow || library.import.is_some() || library.filters {
        root = root.scroll(ScrollSpec {
            stick_bottom: false,
        });
    }
    root
}
