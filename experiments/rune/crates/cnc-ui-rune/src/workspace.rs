#[cfg(not(target_os = "windows"))]
use std::process::Command;

use rune::{App, Cx, El, Icon, Image, ImageCache, MenuItem, ScrollSpec, TextEdit, col, el, row};
use rune_kit::{Button, ButtonVariant, Collapsible, Dialog, TextInput};
use serde_json::Value;

use super::{State, catalog, host::Page, inspector, label, library, secondary, theme, timeline};
use theme::{color, font, size};

macro_rules! carbon {
    ($name:ident, $file:literal) => {
        static $name: Icon = Icon {
            name: $file,
            view: 32.0,
            body: include_str!(concat!("../../../../../web/ui/icons/", $file, ".svg")),
        };
    };
}
carbon!(CATALOG, "catalog");
carbon!(TOOLS, "tools");
carbon!(FLOW, "flow");
carbon!(TIME, "time");
carbon!(HELP, "help");
carbon!(OPEN, "side-panel--open");
carbon!(CLOSE, "side-panel--close");
carbon!(DOWN, "chevron--down");
carbon!(UPLOAD, "upload");
carbon!(RESTART, "restart");

pub fn icon(name: &str) -> &'static Icon {
    match name {
        "catalog" => &CATALOG,
        "tools" => &TOOLS,
        "flow" => &FLOW,
        "time" => &TIME,
        "help" => &HELP,
        "open" => &OPEN,
        "close" => &CLOSE,
        "down" => &DOWN,
        "upload" => &UPLOAD,
        "restart" => &RESTART,
        _ => unreachable!("declared Carbon icon"),
    }
}

#[derive(Default)]
pub struct Workspace {
    pub embedded: super::embedded::Workspace,
    pub controller: super::controller::Controller,
    pub hardware: super::hardware::Hardware,
    pub library: library::Library,
    pub inspector: inspector::Inspector,
    pub graph: super::graph::Map,
    pub timeline: timeline::Timeline,
    pub collapsed: bool,
    pub navigation_open: bool,
    pub chooser_expanded: std::collections::HashSet<String>,
    pub chooser: bool,
    pub chooser_search: TextEdit,
    pub notice: Option<String>,
    pub diagnostics: bool,
    pub brand: Option<Image>,
}

impl Workspace {
    pub fn assets(&mut self, wake: rune::Waker) {
        let cache = ImageCache::new(wake);
        self.brand = Some(
            cache.load(include_bytes!("../../../../../web/assets/brand/rea-app.png").as_slice()),
        );
    }
}

pub fn selected(state: &State) -> Option<&Value> {
    state
        .host
        .snapshot
        .as_ref()?
        .cases
        .iter()
        .find(|item| item["case_id"].as_str() == state.host.selected.as_deref())
}

pub fn badge(value: impl Into<String>, tone: &str) -> El {
    let (ink, bg) = match tone {
        "info" => ("--rb-color-info", "--rb-color-info-bg"),
        "success" | "pass" => ("--rb-color-success", "--rb-color-success-bg"),
        "warning" | "warn" => ("--rb-color-warning", "--rb-color-warning-bg"),
        "danger" => ("--rb-color-danger", "--rb-color-danger-bg"),
        _ => ("--rb-color-secondary", "--rb-color-raised"),
    };
    secondary(value)
        .font(font("--rb-text-caption", 400.0))
        .color(color(ink))
        .px(8.0)
        .py(4.0)
        .radius(size("--rb-radius-control"))
        .fill(color(bg))
        .self_start()
}

pub fn command(title: impl Into<String>, primary: bool) -> Button {
    Button::new(title).variant(if primary {
        ButtonVariant::Contrast
    } else {
        ButtonVariant::Outline
    })
}

pub fn open_url(state: &mut State, url: &str) {
    if !["https://", "http://"].iter().any(|prefix| {
        url.strip_prefix(prefix)
            .is_some_and(|rest| !rest.is_empty() && !rest.starts_with('/'))
    }) || url.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        state.ui.notice = Some("Only HTTP(S) source references can open externally.".into());
        return;
    }

    match launch_url(url) {
        Ok(()) => state.ui.notice = Some("Opened the source reference.".into()),
        Err(error) => {
            state.ui.notice = Some(format!("Could not open the source reference: {error}"))
        }
    }
}

#[cfg(target_os = "windows")]
fn launch_url(url: &str) -> Result<(), String> {
    use windows_sys::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL};
    let url: Vec<u16> = url.encode_utf16().chain(Some(0)).collect();
    // SAFETY: the URL is a validated, NUL-terminated HTTP(S) string; no shell command is built.
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            std::ptr::null(),
            url.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    if result as isize > 32 {
        Ok(())
    } else {
        Err(format!(
            "Windows could not open the URL (code {})",
            result as isize
        ))
    }
}

#[cfg(not(target_os = "windows"))]
fn launch_url(url: &str) -> Result<(), String> {
    Command::new(if cfg!(target_os = "macos") {
        "/usr/bin/open"
    } else {
        "xdg-open"
    })
    .arg(url)
    .spawn()
    .map(|_| ())
    .map_err(|error| error.to_string())
}

pub fn open_path(state: &mut State, path: &str) {
    super::embedded::open(state, path);
}

pub fn menus(mut app: App<State>) -> App<State> {
    let items: [MenuItem<State>; 12] = [
        MenuItem::new("Import artifact…", library::choose_file)
            .id("file-open-firmware")
            .menu("File")
            .key_equivalent("mod+o".parse().unwrap()),
        MenuItem::new("Artifact library", |state: &mut State, _| {
            state.page = Page::Library
        })
        .id("file-library")
        .menu("File"),
        MenuItem::new("Choose system…", |state: &mut State, _| {
            state.ui.chooser = true
        })
        .id("file-choose-system")
        .menu("File"),
        MenuItem::new("Bench home", |state: &mut State, _| state.page = Page::Home)
            .id("view-home")
            .menu("View"),
        MenuItem::new("REA Tool", |state, _| open_existing(state, "rea"))
            .id("view-rea")
            .menu("View"),
        MenuItem::new("Machine lab", |state, _| open_existing(state, "lab"))
            .id("view-lab")
            .menu("View"),
        MenuItem::new("Hardware", |state, _| open_existing(state, "hardware"))
            .id("view-hardware")
            .menu("View"),
        MenuItem::new("Timeline", |state, _| open_existing(state, "timeline"))
            .id("view-timeline")
            .menu("View"),
        MenuItem::new("Capture controls", |state, _| {
            open_existing(state, "capture-controls")
        })
        .id("tools-capture")
        .menu("Tools"),
        MenuItem::new("Diagnostics", |state: &mut State, _| {
            state.ui.diagnostics = true;
            state.ui.navigation_open = false;
        })
        .id("tools-diagnostics")
        .menu("Tools"),
        MenuItem::new("Setup guides", |state, _| open_existing(state, "guides"))
            .id("help-setup")
            .menu("Help"),
        MenuItem::new("Operator guide", |state, _| open_existing(state, "help"))
            .id("help-operator")
            .menu("Help"),
    ];
    for item in items {
        app = app.menu_item(item);
    }
    app
}

pub fn open_existing(state: &mut State, destination: &str) {
    if destination == "hardware" {
        open_path(state, &full_hardware_path(state));
        return;
    }
    if destination == "capture-controls" {
        super::hardware::open(state, false);
        return;
    }
    if destination == "timeline" {
        timeline::open(state);
        return;
    }
    if destination == "rea" {
        state.page = Page::Rea;
        state.ui.navigation_open = false;
        inspector::refresh(state);
        return;
    }
    let path = if matches!(destination, "guides" | "help") {
        Some(format!("/{destination}/"))
    } else {
        state.host.snapshot.as_ref().and_then(|snapshot| {
            selected(state).and_then(|item| catalog::route(item, &snapshot.recovery, destination))
        })
    };
    let Some(path) = path else {
        state.ui.notice = Some("Select a system with a valid firmware identity first.".into());
        return;
    };
    let path = if destination == "lab" {
        format!("{path}#experiment")
    } else {
        path
    };
    open_path(state, &path);
}

fn full_hardware_path(state: &State) -> String {
    let path = state
        .host
        .snapshot
        .as_ref()
        .and_then(|snapshot| {
            selected(state).and_then(|item| catalog::route(item, &snapshot.recovery, "hardware"))
        })
        .unwrap_or_else(|| "/hardware/".into());
    format!("{path}#bench")
}

pub fn route_button(title: &'static str, destination: &'static str, primary: bool) -> El {
    command(title, primary)
        .help(match destination {
            "rea" => "Open native target-byte inspection",
            "timeline" => "Investigation timeline",
            "capture-controls" => "Native acquisition and saved captures",
            _ => "Open inside the app",
        })
        .on_press(move |state: &mut State| open_existing(state, destination))
        .build(&theme::kit())
        .h(size("--rb-control-primary"))
}

fn route_link(title: &'static str, destination: &'static str) -> El {
    Button::new(title)
        .variant(ButtonVariant::Ghost)
        .label_with(move |_, _| secondary(title).color(color("--rb-color-info")))
        .help(if destination == "timeline" {
            "Investigation timeline"
        } else {
            "Open inside the app"
        })
        .on_press(move |state: &mut State| open_existing(state, destination))
        .build(&theme::kit())
        .h(size("--rb-control-height"))
}

fn header(state: &State, width: f32) -> El {
    let mobile = width <= 1000.0;
    let compact = width <= 1180.0;
    let kit = theme::kit();
    let brand = state
        .ui
        .brand
        .as_ref()
        .map(|image| el().size(32.0, 32.0).pixels(image.pixels()))
        .unwrap_or_else(|| el().size(32.0, 32.0));
    let show_name = !state.ui.collapsed && !mobile;
    let home = Button::new("Reforge Bench")
        .variant(ButtonVariant::Ghost)
        .label_with(move |_, ink| {
            let mut contents = row().gap(8.0).center_y().child(brand);
            if show_name {
                contents = contents.child(
                    label("Reforge Bench")
                        .color(ink)
                        .font(font("--rb-text-body", 500.0)),
                );
            }
            contents
        })
        .on_press(|state: &mut State| {
            state.page = Page::Home;
            state.ui.navigation_open = false;
        })
        .build(&kit)
        .key("nav-home")
        .h(44.0);
    let menu = Button::new("Open workspace navigation")
        .variant(ButtonVariant::Ghost)
        .label_with(|_, ink| rune::icon(icon("open"), 20.0).color(ink))
        .on_press(|state: &mut State| state.ui.navigation_open = !state.ui.navigation_open)
        .build(&kit)
        .key("navigation-menu")
        .size(40.0, 40.0);
    let mut header = row()
        .w_full()
        .min_h(size("--rb-shell-height"))
        .px(8.0)
        .center_y()
        .gap(16.0)
        .fill(color("--rb-color-surface"))
        .border_sides([0.0, 0.0, 1.0, 0.0], color("--rb-color-divider"))
        .children(if mobile { vec![menu, home] } else { vec![home] });
    if let (Some(item), Some(snapshot)) = (selected(state), &state.host.snapshot) {
        let info = catalog::info(item);
        let name = info.short_name.unwrap_or(info.name).to_owned();
        let purpose = info.purpose.to_owned();
        header = header.child(
            Button::new(format!("Choose system: {name}"))
                .variant(ButtonVariant::Ghost)
                .label_with(move |_, _| {
                    row()
                        .gap(12.0)
                        .center_y()
                        .child(
                            col()
                                .gap(2.0)
                                .child(label(name).font(font("--rb-text-secondary", 500.0)))
                                .child(secondary(purpose).font(font("--rb-text-caption", 400.0))),
                        )
                        .child(rune::icon(icon("down"), 16.0).color(color("--rb-color-secondary")))
                })
                .on_press(|state: &mut State| state.ui.chooser = true)
                .build(&kit)
                .key("system-chooser")
                .h(44.0)
                .min_w(0.0),
        );
        if !compact && state.page != Page::Workspace {
            header = header.child(secondary(info.version)).child(badge(
                catalog::support(item, &snapshot.recovery),
                if catalog::support(item, &snapshot.recovery) == "Investigation only" {
                    "secondary"
                } else {
                    "info"
                },
            ));
        }
    }
    if !compact {
        header = header.child(el().grow(1.0)).child(
            Button::new("Import artifact")
                .variant(ButtonVariant::Ghost)
                .on_press_tx(library::choose_file)
                .build(&kit)
                .key("shell-import")
                .h(40.0),
        );
    }
    if state.page == Page::Workspace && !mobile {
        header = header.child(
            command("Reload workspace", false)
                .help("Reload this view; saved drafts and retained evidence stay available")
                .on_press(super::embedded::reload)
                .build(&kit)
                .key("workspace-reload")
                .h(40.0),
        );
    }
    if state.page == Page::Workspace && state.ui.embedded.stop_available {
        header = header.child(
            Button::new("E-stop simulation")
                .variant(ButtonVariant::Danger)
                .help("Stop this simulation, not a physical machine safety circuit")
                .disabled(!state.ui.embedded.stop_available)
                .on_press(super::embedded::stop)
                .build(&kit)
                .key("workspace-simulation-stop")
                .h(44.0),
        );
    }
    header
}

fn sidebar(state: &State, mobile: bool) -> El {
    let collapsed = state.ui.collapsed && !mobile;
    let kit = theme::kit();
    let mut heading = row().center_y().w_full().px(8.0).h(44.0);
    if !collapsed {
        heading = heading.child(secondary("Workspace")).child(el().grow(1.0));
    }
    heading = heading.child(
        Button::new(if collapsed {
            "Expand navigation"
        } else {
            "Collapse navigation"
        })
        .variant(ButtonVariant::Ghost)
        .label_with(move |_, ink| {
            rune::icon(icon(if collapsed { "open" } else { "close" }), 16.0).color(ink)
        })
        .on_press(move |state: &mut State| {
            if mobile {
                state.ui.navigation_open = false;
            } else {
                state.ui.collapsed = !state.ui.collapsed;
                state.persist_preferences();
            }
        })
        .build(&kit)
        .key("navigation-toggle")
        .size(40.0, 40.0),
    );
    let mut nav = col()
        .key("workspace-navigation")
        .w(if collapsed {
            size("--rb-nav-rail-width")
        } else {
            size("--rb-nav-width")
        })
        .h_full()
        .pt(size("--rb-space-6"))
        .pb(size("--rb-space-5"))
        .fill(color("--rb-color-surface"))
        .border_sides([0.0, 1.0, 0.0, 0.0], color("--rb-color-divider"))
        .child(heading);
    for (title, destination, symbol) in [
        ("Library", "library", "catalog"),
        ("Lab", "lab", "tools"),
        ("REA Tool", "rea", "flow"),
        ("Timeline", "timeline", "time"),
        ("Hardware", "hardware", "tools"),
        ("Capture controls", "capture-controls", "tools"),
    ] {
        let active = (destination == "library" && state.page == Page::Library)
            || (destination == "rea" && state.page == Page::Rea)
            || (destination == "timeline" && state.page == Page::Timeline)
            || (destination == "capture-controls" && state.page == Page::Hardware)
            || (state.page == Page::Workspace
                && state.ui.embedded.destination() == Some(destination));
        let button = Button::new(title)
            .variant(ButtonVariant::Ghost)
            .help(
                if matches!(
                    destination,
                    "library" | "rea" | "timeline" | "capture-controls"
                ) {
                    title
                } else {
                    "Open inside the app"
                },
            )
            .label_with(move |_, ink| {
                let mut contents = row()
                    .gap(12.0)
                    .center_y()
                    .w_full()
                    .child(rune::icon(icon(symbol), 16.0).color(ink));
                if !collapsed {
                    contents =
                        contents.child(label(title).font(font("--rb-text-secondary", 400.0)));
                }
                contents
            })
            .on_press(move |state: &mut State| {
                if destination == "library" {
                    state.page = Page::Library;
                    state.ui.navigation_open = false;
                } else {
                    open_existing(state, destination);
                }
            })
            .build(&kit)
            .key(format!("nav-{destination}"))
            .w_full()
            .h(48.0)
            .radius(0.0);
        nav = nav.child(if active {
            button
                .fill(color("--rb-color-selected"))
                .border_sides([0.0, 0.0, 0.0, 3.0], color("--rb-color-selection-edge"))
        } else {
            button
        });
    }
    nav = nav.child(el().grow(1.0));
    for (title, destination) in [
        ("Setup guides", "guides"),
        ("Help", "help"),
        ("Diagnostics", "diagnostics"),
    ] {
        nav = nav.child(
            Button::new(title)
                .variant(ButtonVariant::Ghost)
                .help(if destination == "diagnostics" {
                    "Local error details and report"
                } else {
                    "Open inside the app"
                })
                .label_with(move |_, ink| {
                    let mut body = row()
                        .gap(12.0)
                        .center_y()
                        .w_full()
                        .child(rune::icon(icon("help"), 16.0).color(ink));
                    if !collapsed {
                        body = body.child(secondary(title));
                    }
                    body
                })
                .on_press(move |state: &mut State| {
                    if destination == "diagnostics" {
                        state.ui.diagnostics = true;
                        state.ui.navigation_open = false;
                    } else {
                        open_existing(state, destination);
                    }
                })
                .build(&kit)
                .key(format!("nav-{destination}"))
                .w_full()
                .h(40.0),
        );
    }
    if !collapsed {
        let version = format!(
            "v{} · build {}",
            env!("CARGO_PKG_VERSION"),
            super::backend_session::build_identity()["build_number"]
        );
        nav = nav.child(
            secondary(version)
                .key("native-version")
                .font(font("--rb-text-caption", 400.0))
                .px(16.0)
                .py(8.0),
        );
    }
    if !collapsed && let Some(snapshot) = &state.host.snapshot {
        nav = nav
            .child(
                secondary(format!(
                    "Host {}",
                    catalog::value(&snapshot.health, "version", "Unknown")
                ))
                .font(font("--rb-text-caption", 400.0))
                .px(16.0)
                .py(8.0),
            )
            .child(
                secondary(format!(
                    "Frontend {}",
                    catalog::value(&snapshot.health, "frontend", "Unknown")
                ))
                .font(font("--rb-text-caption", 400.0))
                .px(16.0)
                .pb(12.0)
                .wrap_anywhere(),
            );
    }
    nav
}

fn home(state: &State, narrow: bool) -> El {
    let Some(item) = selected(state) else {
        return col()
            .padding(24.0)
            .gap(16.0)
            .child(label("Bench home").font(font("--rb-text-title", 500.0)))
            .child(secondary(if state.host.loading {
                "Loading imported firmware…"
            } else {
                "Choose a system from the firmware library."
            }))
            .child(
                command("Artifact library", true)
                    .on_press(|state: &mut State| state.page = Page::Library)
                    .build(&theme::kit())
                    .h(44.0),
            );
    };
    let info = catalog::info(item);
    let snapshot = state.host.snapshot.as_ref().unwrap();
    let support = catalog::support(item, &snapshot.recovery);
    let mut identity = if narrow {
        col()
    } else {
        row().center_y().justify_between()
    };
    identity = identity
        .w_full()
        .padding(24.0)
        .gap(16.0)
        .fill(color("--rb-color-surface"))
        .border_sides([0.0, 0.0, 0.0, 2.0], color("--rb-color-info"))
        .child(
            col()
                .min_w(0.0)
                .grow(1.0)
                .gap(4.0)
                .child(secondary("Selected system"))
                .child(
                    label(
                        info.short_name
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("{} {}", info.vendor, info.name)),
                    )
                    .font(font("--rb-text-title", 500.0))
                    .key("home-system"),
                )
                .child(secondary(format!("{} · {}", info.version, info.purpose))),
        )
        .child(badge(
            support,
            if support == "Investigation only" {
                "secondary"
            } else {
                "info"
            },
        ));
    let mut tasks = if narrow { col() } else { row() };
    tasks = tasks.w_full().gap(16.0);
    let lab = "Replay the pulse fixture and inspect its simulated signals.";
    for (title, description, destination, symbol, primary) in [
        (
            "Open REA",
            "Inspect the selected artifact and its retained evidence.",
            "rea",
            "flow",
            true,
        ),
        ("Open Lab", lab, "lab", "tools", false),
        (
            "Getting started",
            "Configure instruments, connect the bench and check the first signals.",
            "procedure",
            "catalog",
            false,
        ),
    ] {
        let task = col()
            .min_w(0.0)
            .gap(12.0)
            .padding(24.0)
            .fill(color("--rb-color-surface"))
            .radius(size("--rb-radius-panel"))
            .child(rune::icon(icon(symbol), 24.0).color(color("--rb-color-secondary")))
            .child(label(title).font(font("--rb-text-body", 500.0)))
            .child(secondary(description).grow(1.0))
            .child(route_button(title, destination, primary).self_start());
        tasks = tasks.child(if narrow {
            task.w_full()
        } else {
            task.grow(1.0).w_frac(1.0 / 3.0)
        });
    }
    let links = row()
        .flex_wrap()
        .gap(24.0)
        .child(
            Button::new("Artifact library")
                .variant(ButtonVariant::Ghost)
                .label_with(|_, _| secondary("Artifact library").color(color("--rb-color-info")))
                .on_press(|state: &mut State| state.page = Page::Library)
                .build(&theme::kit())
                .h(40.0),
        )
        .child(route_link("Investigation timeline", "timeline"))
        .child(route_link("Operator guide", "help"));
    col()
        .w_full()
        .h_full()
        .min_h(0.0)
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .child(
            col()
                .w_full()
                .max_w(1152.0)
                .self_center()
                .px(size("--rb-gutter"))
                .py(if narrow {
                    size("--rb-space-6")
                } else {
                    size("--rb-space-8")
                })
                .gap(24.0)
                .child(secondary("Reforge Bench"))
                .child(label("Bench home").font(font("--rb-text-title", 500.0)))
                .child(identity)
                .child(tasks)
                .child(links),
        )
}

fn chooser_option(state: &State, group: &[&Value]) -> El {
    let item = group
        .iter()
        .find(|item| item["case_id"].as_str() == state.host.selected.as_deref())
        .copied()
        .unwrap_or(group[0]);
    let info = catalog::info(item);
    let id = catalog::value(item, "case_id", "").to_owned();
    let active = state.host.selected.as_deref() == Some(&id);
    let name = format!("{} · {}", info.vendor, info.name);
    let version = info.version.to_owned();
    let purpose = info.purpose.to_owned();
    let support = catalog::support(item, &state.host.snapshot.as_ref().unwrap().recovery);
    command(name.clone(), false)
        .label_with(move |_, _| {
            col()
                .w_full()
                .gap(4.0)
                .child(label(name))
                .child(secondary(purpose))
                .child(
                    row()
                        .gap(12.0)
                        .flex_wrap()
                        .child(secondary(version))
                        .child(badge(
                            if support == "Research emulation" {
                                "Emulation available"
                            } else {
                                support
                            },
                            if support == "Investigation only" {
                                "secondary"
                            } else {
                                "info"
                            },
                        )),
                )
        })
        .on_press(move |state: &mut State| {
            state.select_system(id.clone());
            state.ui.chooser = false;
            state.ui.chooser_search = TextEdit::default();
            state.ui.library.clear();
        })
        .build(&theme::kit())
        .key(format!("choose-{}", catalog::value(item, "case_id", "")))
        .w_full()
        .min_h(96.0)
        .fill(color(if active {
            "--rb-color-selected"
        } else {
            "--rb-color-field"
        }))
}

fn chooser(state: &State, layers: &mut Vec<El>) {
    if !state.ui.chooser {
        return;
    }
    let mut list = col()
        .w_full()
        .gap(8.0)
        .padding(16.0)
        .min_h(0.0)
        .max_h(420.0)
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .child(
            TextInput::new(&state.ui.chooser_search)
                .label("Search systems")
                .placeholder("Controller or manufacturer")
                .height(40.0)
                .on_edit(|state: &mut State| &mut state.ui.chooser_search)
                .build(&theme::kit())
                .key("chooser-search"),
        );
    let mut count = 0;
    if let Some(snapshot) = &state.host.snapshot {
        let query = state.ui.chooser_search.text.trim().to_lowercase();
        let mut groups: Vec<Vec<&Value>> = Vec::new();
        for item in &snapshot.cases {
            let group = catalog::info(item).group;
            if let Some(existing) = groups
                .iter_mut()
                .find(|items| catalog::info(items[0]).group == group)
            {
                existing.push(item);
            } else {
                groups.push(vec![item]);
            }
        }
        groups.sort_by(|a, b| {
            let rank = |items: &Vec<&Value>| {
                items
                    .iter()
                    .map(|item| catalog::progress(item, &snapshot.recovery).0)
                    .max()
                    .unwrap_or(0)
            };
            rank(b)
                .cmp(&rank(a))
                .then_with(|| {
                    catalog::info(a[0])
                        .vendor
                        .to_lowercase()
                        .cmp(&catalog::info(b[0]).vendor.to_lowercase())
                })
                .then_with(|| {
                    catalog::info(a[0])
                        .name
                        .to_lowercase()
                        .cmp(&catalog::info(b[0]).name.to_lowercase())
                })
        });
        groups.retain(|group| {
            let item = group
                .iter()
                .find(|item| item["case_id"].as_str() == state.host.selected.as_deref())
                .copied()
                .unwrap_or(group[0]);
            let info = catalog::info(item);
            format!(
                "{} {} {} {}",
                info.vendor, info.name, info.purpose, info.version
            )
            .to_lowercase()
            .contains(&query)
        });
        let mut brands = Vec::new();
        for group in &groups {
            let vendor = catalog::info(group[0]).vendor;
            if !brands.contains(&vendor) {
                brands.push(vendor);
            }
        }
        for vendor in brands {
            list = list.child(
                label(vendor)
                    .font(font("--rb-text-secondary", 500.0))
                    .pt(12.0),
            );
            let mut families = Vec::new();
            for group in groups
                .iter()
                .filter(|group| catalog::info(group[0]).vendor.eq_ignore_ascii_case(vendor))
            {
                let family = catalog::info(group[0]).family;
                if !families.contains(&family) {
                    families.push(family);
                }
            }
            for family in families {
                list = list.child(secondary(family));
                let matching: Vec<_> = groups
                    .iter()
                    .filter(|group| {
                        catalog::info(group[0]).vendor.eq_ignore_ascii_case(vendor)
                            && catalog::info(group[0]).family == family
                    })
                    .collect();
                let mut inactive = Vec::new();
                for group in matching {
                    if group
                        .iter()
                        .any(|item| item["case_id"].as_str() == state.host.selected.as_deref())
                    {
                        list = list.child(chooser_option(state, group));
                    } else {
                        inactive.push(group);
                    }
                    count += 1;
                }
                if inactive.len() > 1 {
                    let key = format!("{vendor}/{family}");
                    let open = !query.is_empty() || state.ui.chooser_expanded.contains(&key);
                    let mut others = col().w_full().gap(8.0);
                    for group in &inactive {
                        others = others.child(chooser_option(state, group));
                    }
                    list = list.child(
                        Collapsible::new(
                            format!("Other {family} systems · {}", inactive.len()),
                            open,
                        )
                        .on_toggle(move |state: &mut State| {
                            if !state.ui.chooser_expanded.remove(&key) {
                                state.ui.chooser_expanded.insert(key.clone());
                            }
                        })
                        .child(others)
                        .build(&theme::kit()),
                    );
                } else if let Some(group) = inactive.first() {
                    list = list.child(chooser_option(state, group));
                }
            }
        }
    }
    if count == 0 {
        list = list.child(secondary(
            "No matching systems. Change or clear the search.",
        ));
    }
    Dialog::new("system-picker", "Choose a system")
        .open(true)
        .width(680.0)
        .fit()
        .on_close(|state: &mut State| state.ui.chooser = false)
        .child(list)
        .build(&theme::kit(), layers);
}

fn diagnostics(state: &State, layers: &mut Vec<El>) {
    if !state.ui.diagnostics {
        return;
    }
    let report = super::diagnostics::report(state.host.snapshot.as_ref().map(|s| &s.health));
    let copied = report.clone();
    let content = col().w_full().padding(16.0).gap(16.0)
        .child(secondary("Local error details. Copy and save export a summary without messages or paths. Nothing is uploaded."))
        .child(row().w_full().flex_wrap().gap(8.0)
            .child(command("Copy summary", false).on_press_tx(move |_: &mut State, tx| tx.copy(copied.clone()))
                .build(&theme::kit()).key("diagnostics-copy").h(44.0))
            .child(command("Save summary", true).on_press_tx(move |state: &mut State, tx| {
                let report = super::diagnostics::report(state.host.snapshot.as_ref().map(|s| &s.health));
                tx.save_file(rune::FileDialogOptions {
                    title: "Save diagnostic summary".into(), file_name: Some("reforge-diagnostics.json".into()), ..Default::default()
                }, move |state: &mut State, result, _| match result {
                    Ok(Some(path)) => {
                        state.ui.notice = Some(match std::fs::write(path, &report) {
                            Ok(()) => "Diagnostics saved".into(),
                            Err(error) => format!("Could not save diagnostics: {error}"),
                        });
                    }
                    Ok(None) => {},
                    Err(error) => state.ui.notice = Some(error.to_string()),
                });
            }).build(&theme::kit()).key("diagnostics-save").h(44.0)))
        .child(col().w_full().h(320.0).scroll(ScrollSpec { stick_bottom: false })
            .child(secondary(super::diagnostics::local_report()).font(font("--rb-text-secondary", 400.0).mono()).wrap_anywhere()));
    Dialog::new("diagnostics-dialog", "Diagnostics")
        .open(true)
        .width(720.0)
        .fit()
        .on_close(|state: &mut State| state.ui.diagnostics = false)
        .child(content)
        .build(&theme::kit(), layers);
}

pub fn view(state: &State, cx: &mut Cx<State>) -> El {
    cx.foreground(color("--rb-color-text"));
    cx.focus_ring(2.0, 2.0, color("--rb-color-focus"));
    let width = cx
        .layout_size("workspace-root")
        .map_or(1440.0, |size| size[0]);
    let mobile = width <= 1000.0;
    let narrow = width <= 760.0;
    let mut layers = Vec::new();
    let body = match state.page {
        Page::Workspace => super::embedded::view(state),
        Page::Hardware => super::hardware::view(
            state,
            cx,
            width
                - if mobile {
                    0.0
                } else if state.ui.collapsed {
                    size("--rb-nav-rail-width")
                } else {
                    size("--rb-nav-width")
                }
                - 48.0,
        ),
        Page::Library => library::view(state, width, &mut layers),
        Page::Rea => inspector::view(state, cx, width),
        Page::Timeline => timeline::view(
            state,
            cx,
            width
                - if mobile {
                    0.0
                } else if state.ui.collapsed {
                    size("--rb-nav-rail-width")
                } else {
                    size("--rb-nav-width")
                }
                - 48.0,
        ),
        _ => home(state, narrow),
    };
    let mut content = col().grow(1.0).min_w(0.0).h_full();
    if let Some(error) = &state.host.error {
        content = content.child(
            col()
                .w_full()
                .gap(8.0)
                .padding(16.0)
                .child(
                    label(error)
                        .key("host-error")
                        .color(color("--rb-color-danger")),
                )
                .children(if state.host.snapshot.is_some() {
                    vec![secondary("Showing the last library snapshot.")]
                } else {
                    vec![]
                })
                .child(
                    row()
                        .w_full()
                        .flex_wrap()
                        .gap(8.0)
                        .child(
                            command("Retry connection", true)
                                .disabled(state.host.loading)
                                .on_press(|state: &mut State| state.host.refresh())
                                .build(&theme::kit())
                                .key("host-retry")
                                .h(44.0),
                        )
                        .child(
                            command("Diagnostics", false)
                                .on_press(|state: &mut State| state.ui.diagnostics = true)
                                .build(&theme::kit())
                                .key("host-diagnostics")
                                .h(44.0),
                        ),
                ),
        );
    }
    if let Some(notice) = &state.ui.notice {
        content = content.child(secondary(notice).key("workspace-notice").padding(12.0));
    }
    content = content.child(body);
    if mobile && state.ui.navigation_open {
        let height = cx
            .layout_size("workspace-root")
            .map_or(800.0, |size| size[1]);
        layers.push(
            el().key("navigation-drawer")
                .role("dialog")
                .label("Workspace navigation")
                .absolute(0.0, size("--rb-shell-height"))
                .h(height - size("--rb-shell-height"))
                .w(size("--rb-nav-width"))
                .modal(true, |state: &mut State| state.ui.navigation_open = false)
                .modal_click_outside(true)
                .child(sidebar(state, true)),
        );
    }
    chooser(state, &mut layers);
    diagnostics(state, &mut layers);
    col()
        .key("workspace-root")
        .w_full()
        .h_full()
        .fill(color("--rb-color-canvas"))
        .child(header(state, width))
        .child(
            row()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .children(if mobile {
                    vec![]
                } else {
                    vec![sidebar(state, false)]
                })
                .child(content),
        )
        .children(layers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};
    use serde_json::json;

    #[test]
    fn navigation_and_context_fit_desktop_and_narrow_windows() {
        for width in [1440.0, 1000.0, 560.0] {
            let mut state = State::idle();
            state.page = Page::Library;
            state.host.snapshot = Some(
                super::super::host::Snapshot::parse(
                    json!({"ok":true,"version":"test","frontend":"test"}),
                    json!({"ok":true,"cases":[super::super::provider::example()]}),
                    json!({"profiles":{}}),
                )
                .unwrap(),
            );
            state.host.selected = state.host.snapshot.as_ref().unwrap().cases[0]["case_id"]
                .as_str()
                .map(str::to_owned);
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
            let elements = rt.elements();
            assert_eq!(
                elements
                    .iter()
                    .filter(|e| e.key.as_deref() == Some("system-chooser"))
                    .count(),
                1
            );
            let context = elements
                .iter()
                .find(|e| e.key.as_deref() == Some("system-chooser"))
                .unwrap();
            assert!(context.rect[0] >= 0.0 && context.rect[0] + context.rect[2] <= width);
            let selected = rt.state.host.selected.clone();
            open_existing(&mut rt.state, "hardware");
            assert_eq!(rt.state.ui.embedded.destination(), Some("hardware"));
            assert_eq!(rt.state.host.selected, selected);
            assert!(rt.state.page == Page::Workspace);
            open_existing(&mut rt.state, "capture-controls");
            assert!(rt.state.page == Page::Hardware);
        }
    }
}
