use std::sync::mpsc::{self, Receiver};

use rune::{Cx, El, TextEdit, col, el, row};
use rune_kit::{Button, ButtonVariant, Collapsible, TextInput};
use serde_json::Value;

use super::{
    State, catalog,
    graph_model::{self, Graph},
    host, label, secondary, theme,
};
use catalog::value;
use theme::{color, font};

type Reply = (String, bool, Result<Graph, String>);

#[derive(Default)]
pub struct Map {
    data: Option<Graph>,
    case: Option<String>,
    recovery: bool,
    receiver: Option<Receiver<Reply>>,
    error: Option<String>,
    selected: String,
    search: TextEdit,
    page: usize,
    mode: usize,
    all: bool,
    provenance: bool,
    audit: bool,
}

impl Map {
    pub fn loading(&self) -> bool {
        self.receiver.is_some()
    }

    pub fn poll(&mut self, selected: Option<&str>) -> bool {
        let Some((case, recovery, result)) = self
            .receiver
            .as_ref()
            .and_then(|receiver| receiver.try_recv().ok())
        else {
            return false;
        };
        self.receiver = None;
        if selected != Some(&case)
            || self.case.as_deref() != Some(&case)
            || self.recovery != recovery
        {
            return true;
        }
        match result {
            Ok(data) => {
                if data.node(&self.selected).is_none() {
                    self.selected = data.initial.clone();
                }
                self.data = Some(data);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
        true
    }

    fn choose(&mut self, id: String) {
        self.selected = id;
        self.page = 0;
        self.search = TextEdit::default();
        self.all = false;
        self.provenance = false;
    }
}

pub fn refresh(state: &mut State) {
    let Some(case) = state.host.selected.clone() else {
        return;
    };
    let recovery = state.ui.inspector.step == 7;
    let map = &mut state.ui.graph;
    if map.case.as_ref() != Some(&case) || map.recovery != recovery {
        *map = Map {
            case: Some(case.clone()),
            recovery,
            ..Default::default()
        };
    }
    if map.receiver.is_some() {
        return;
    }
    let port = state.host.port;
    let expected = state
        .host
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.health.clone());
    let wake = state.host.waker.clone();
    let (sender, receiver) = mpsc::channel();
    map.receiver = Some(receiver);
    map.error = None;
    std::thread::spawn(move || {
        let result = (|| {
            let health = host::read(port, "/api/desktop/health")?;
            if expected.as_ref() != Some(&health) || health["ok"] != true {
                return Err(
                    "Host identity changed. Refresh the library before reading the map.".into(),
                );
            }
            if case != catalog::DDCS_SHA {
                return Graph::parse(super::provider::map(&case)?, &case);
            }
            let data = host::read(
                port,
                if recovery {
                    "/process/recovery-map.json"
                } else {
                    "/process/firmware-map.json"
                },
            )?;
            let data = if recovery {
                graph_model::recovery(data, &case)?
            } else if !graph_model::has_functions(&data, &case) {
                match host::read(port, &format!("/api/library/history?case_id={case}")) {
                    Ok(history) => graph_model::resolve(data, history, &case)?,
                    Err(_) if data["targets"][&case].is_object() => data,
                    Err(error) => return Err(error),
                }
            } else {
                data
            };
            let graph = Graph::parse(data, &case)?;
            if health != host::read(port, "/api/desktop/health")? {
                return Err("Host build changed during map refresh. Refresh again.".into());
            }
            Ok(graph)
        })();
        let _ = sender.send((case, recovery, result));
        if let Some(wake) = wake {
            wake();
        }
    });
}

fn command(title: impl Into<String>, key: &str, action: impl Fn(&mut State) + 'static) -> El {
    Button::new(title)
        .variant(ButtonVariant::Ghost)
        .on_press(action)
        .build(&theme::kit())
        .key(key)
        .min_h(40.0)
}

fn card(node: &Value, selected: bool, key: String) -> El {
    let id = value(node, "id", "").to_owned();
    let title = value(node, "displayLabel", value(node, "label", ""))
        .trim_start_matches("<EXTERNAL>::")
        .to_owned();
    let address = value(node, "address", "").to_owned();
    Button::new(value(node, "label", ""))
        .variant(ButtonVariant::Ghost)
        .label_with(move |_, _| {
            let mut content = col().w_full().min_w(0.0).gap(4.0).child(
                label(&title)
                    .font(font("--rb-text-body", 500.0))
                    .wrap_anywhere(),
            );
            if !address.is_empty() {
                content = content
                    .child(secondary(&address).font(font("--rb-text-secondary", 400.0).mono()));
            }
            content
        })
        .on_press_tx(move |state: &mut State, tx: &mut rune::Tx| {
            state.ui.graph.choose(id.clone());
            tx.focus("graph-node-1-0", true);
        })
        .build(&theme::kit())
        .key(key)
        .checked(selected)
        .w_full()
        .min_h(64.0)
        .padding(12.0)
        .fill(color(if selected {
            "--rb-color-selected"
        } else {
            "--rb-color-surface"
        }))
        .border(
            1.0,
            color(if selected {
                "--rb-color-selection-edge"
            } else {
                "--rb-color-divider"
            }),
        )
        .radius(4.0)
}

fn diagram(
    map: &Map,
    data: &Graph,
    cx: &Cx<State>,
    width: f32,
    incoming: &[String],
    outgoing: &[String],
    page: usize,
) -> El {
    let narrow = width < 600.0;
    let mut body = if narrow { col() } else { row() };
    body = body
        .key("graph-diagram")
        .w_full()
        .min_h(360.0)
        .gap(if narrow { 24.0 } else { 32.0 })
        .py(16.0);
    let page_nodes = |ids: &[String]| -> Vec<String> {
        let page = page.min(ids.len().saturating_sub(1) / 4);
        ids.iter().skip(page * 4).take(4).cloned().collect()
    };
    let groups = [
        page_nodes(incoming),
        vec![map.selected.clone()],
        page_nodes(outgoing),
    ];
    let origin = cx.layout_with("graph-diagram", |rect| rect);
    let focus = cx.layout_with("graph-node-1-0", |rect| rect);
    let target_rects: Vec<_> = [(0, &groups[0]), (2, &groups[2])]
        .into_iter()
        .map(|(side, nodes)| {
            (
                side,
                nodes
                    .iter()
                    .enumerate()
                    .filter_map(|(index, _)| {
                        cx.layout_with(&format!("graph-node-{side}-{index}"), |rect| rect)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    if !narrow
        && let Some(origin) = origin
        && let Some(focus) = focus
    {
        for (side, rects) in target_rects {
            let x = if side == 0 {
                focus[0]
            } else {
                focus[0] + focus[2]
            } - origin[0];
            let y = focus[1] + focus[3] / 2.0 - origin[1];
            let targets: Vec<_> = rects
                .into_iter()
                .map(|rect| {
                    [
                        if side == 0 {
                            rect[0] + rect[2]
                        } else {
                            rect[0]
                        } - origin[0],
                        rect[1] + rect[3] / 2.0 - origin[1],
                    ]
                })
                .collect();
            if let Some(first) = targets.first() {
                let trunk = (x + first[0]) / 2.0;
                let top = targets.iter().fold(y, |a, b| a.min(b[1]));
                let bottom = targets.iter().fold(y, |a, b| a.max(b[1]));
                body = body.child(
                    el().key(format!("graph-edge-{side}-trunk"))
                        .absolute(trunk, top)
                        .size(1.5, (bottom - top).max(1.5))
                        .fill(color("--rb-color-control-edge")),
                );
                for [to_x, to_y] in std::iter::once([x, y]).chain(targets) {
                    body = body.child(
                        el().absolute(trunk.min(to_x), to_y)
                            .size((trunk - to_x).abs(), 1.5)
                            .fill(color("--rb-color-control-edge")),
                    );
                    if side == 2 || to_x == x {
                        body = body.child(secondary("›").absolute(to_x - 7.0, to_y - 10.0).w(12.0));
                    }
                }
            }
        }
    }
    for (side, (title, ids)) in [
        (format!("Incoming ({})", incoming.len()), &groups[0]),
        ("Selected node".into(), &groups[1]),
        (format!("Outgoing ({})", outgoing.len()), &groups[2]),
    ]
    .into_iter()
    .enumerate()
    {
        let mut nodes = col().w_full().gap(12.0).grow(1.0).justify_center();
        if ids.is_empty() {
            nodes = nodes.child(secondary("No links"));
        }
        for (index, id) in ids.iter().enumerate() {
            if let Some(node) = data.node(id) {
                nodes = nodes.child(card(
                    node,
                    id == &map.selected,
                    format!("graph-node-{side}-{index}"),
                ));
            }
        }
        body = body.child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .w(if narrow { width } else { (width - 64.0) / 3.0 })
                .gap(16.0)
                .child(secondary(title))
                .child(nodes),
        );
    }
    body
}

fn detail(map: &Map, data: &Graph) -> El {
    let Some(node) = data.node(&map.selected) else {
        return col();
    };
    let mut body = col()
        .min_w(0.0)
        .gap(16.0)
        .child(secondary(value(node, "kind", "")))
        .child(
            label(value(node, "label", ""))
                .key("graph-node-title")
                .font(font("--rb-text-body", 500.0))
                .wrap_anywhere(),
        );
    for (title, key) in [
        ("Address", "address"),
        ("End", "end"),
        ("Permissions", "permissions"),
        ("Status", "status"),
        ("Compared cases", "cases"),
        ("Signature", "signature"),
    ] {
        let field = &node[key];
        if !field.is_null() && field != "" && field != "not established" {
            body = body.child(
                col().gap(4.0).child(secondary(title)).child(
                    label(
                        field
                            .as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| field.to_string()),
                    )
                    .wrap_anywhere(),
                ),
            );
        }
    }
    if node["source"] == "not established" {
        body = body.child(secondary("Source: not established"));
    }
    for text in [
        value(node, "observation", ""),
        value(&data.metadata, "scope", ""),
    ] {
        if !text.is_empty() {
            body = body.child(secondary(text));
        }
    }
    if data.metadata["scope"].as_str().is_none() {
        let mut verbs = std::collections::BTreeSet::new();
        for edge in &data.edges {
            if (edge["source"] == map.selected || edge["target"] == map.selected)
                && (map.mode == 0 || (edge["verb"] == "calls") == (map.mode == 1))
            {
                verbs.insert(value(edge, "verb", "").replace('_', " "));
            }
        }
        body = body.child(secondary(verbs.into_iter().collect::<Vec<_>>().join(" · ")));
    }
    let mut provenance = col().w_full().gap(8.0);
    for (title, field) in [
        ("Source", &node["source"]),
        ("SHA-256", &node["sha256"]),
        ("Graph SHA-256", &data.metadata["graph_sha256"]),
    ] {
        if let Some(field) = field.as_str() {
            provenance = provenance
                .child(secondary(title))
                .child(label(field).wrap_anywhere().selectable(true));
        }
    }
    for (key, value) in data.metadata["provenance"]
        .as_object()
        .into_iter()
        .flatten()
    {
        provenance = provenance.child(secondary(key)).child(
            label(
                value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string()),
            )
            .wrap_anywhere()
            .selectable(true),
        );
    }
    body.child(
        Collapsible::new("Source & identity", map.provenance)
            .on_toggle(|state: &mut State| state.ui.graph.provenance = !state.ui.graph.provenance)
            .child(provenance)
            .build(&theme::kit())
            .key("graph-provenance"),
    )
}

pub fn view(state: &State, cx: &Cx<State>, width: f32) -> El {
    let map = &state.ui.graph;
    let mut body = col().w_full().gap(16.0).instant_changes();
    if let Some(error) = &map.error {
        body = body.child(
            label(format!(
                "{error}{}",
                if map.data.is_some() {
                    " Showing retained map; refresh failed."
                } else {
                    ""
                }
            ))
            .key("graph-error")
            .color(color("--rb-color-danger")),
        );
    }
    let Some(data) = map
        .data
        .as_ref()
        .filter(|_| map.case.as_deref() == state.host.selected.as_deref())
    else {
        return body.child(
            label(if map.receiver.is_some() {
                "Loading firmware map…"
            } else {
                "Firmware map unavailable. Retry to read the selected host."
            })
            .key("graph-empty"),
        );
    };
    body = body.child(
        secondary(format!(
            "{} nodes · {} relationships{}",
            data.nodes.len(),
            data.edges.len(),
            data.metadata["summary"]
                .as_str()
                .map_or(String::new(), |summary| format!(" · {summary}"))
        ))
        .key("graph-status"),
    );
    if data.audit["ok"] == false {
        let findings = data.audit["findings"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let mut content = col().w_full().gap(12.0);
        for finding in &findings {
            content = content
                .child(secondary(value(finding, "node", "")))
                .child(label(value(finding, "message", "Evidence finding")).wrap_anywhere());
        }
        body = body.child(
            Collapsible::new(
                format!("{} evidence audit findings", findings.len()),
                map.audit,
            )
            .on_toggle(|state: &mut State| state.ui.graph.audit = !state.ui.graph.audit)
            .child(content)
            .build(&theme::kit())
            .key("graph-audit"),
        );
    }
    let mut search = row().w_full().gap(12.0).flex_wrap().child(
        TextInput::new(&map.search)
            .label("Search evidence map")
            .placeholder("Find a component, function, address or type")
            .height(40.0)
            .on_edit(|state: &mut State| &mut state.ui.graph.search)
            .on_change(|state: &mut State, _| {
                state.ui.graph.page = 0;
                state.ui.graph.all = false;
            })
            .build(&theme::kit())
            .key("graph-search")
            .grow(1.0)
            .min_w(180.0),
    );
    if !map.recovery {
        for (index, title) in ["All links", "Calls", "Evidence"].into_iter().enumerate() {
            search = search.child(
                command(title, &format!("graph-mode-{index}"), move |state| {
                    state.ui.graph.mode = index;
                    state.ui.graph.page = 0;
                })
                .checked(map.mode == index)
                .fill(color(if map.mode == index {
                    "--rb-color-selected"
                } else {
                    "--rb-color-surface"
                })),
            );
        }
    }
    body = body.child(search);
    let mut shortcuts = row().w_full().gap(12.0).flex_wrap();
    if map.recovery {
        for (index, (title, query)) in [
            ("Geometry", None),
            ("All functions", Some("")),
            ("Compared", Some("bounded comparison")),
            ("Static only", Some("static observation only")),
        ]
        .into_iter()
        .enumerate()
        {
            shortcuts = shortcuts.child(command(
                title,
                &format!("graph-shortcut-{index}"),
                move |state| {
                    if let Some(query) = query {
                        state.ui.graph.search = TextEdit::new(query);
                        state.ui.graph.all = true;
                        state.ui.graph.page = 0;
                    } else {
                        state.ui.graph.choose("recovery:0x7eda4".into());
                    }
                },
            ));
        }
    } else {
        for (index, (title, id)) in data.landmarks.iter().enumerate() {
            let id = id.clone();
            shortcuts = shortcuts.child(command(
                title,
                &format!("graph-shortcut-{index}"),
                move |state| state.ui.graph.choose(id.clone()),
            ));
        }
    }
    body = body.child(shortcuts);
    let searching = !map.search.text.trim().is_empty() || map.all;
    let matches = data.search(&map.search.text, map.all);
    let incoming = data.linked(&map.selected, true, map.mode);
    let outgoing = data.linked(&map.selected, false, map.mode);
    let count = if searching {
        matches.len()
    } else {
        incoming.len().max(outgoing.len())
    };
    let pages = count.div_ceil(4).max(1);
    let page = map.page.min(pages - 1);
    let mut paging = row().w_full().gap(12.0).flex_wrap().center_y().child(
        secondary(if searching {
            format!("{} matches · Page {} of {pages}", matches.len(), page + 1)
        } else {
            format!(
                "{} incoming · {} outgoing · Page {} of {pages}",
                incoming.len(),
                outgoing.len(),
                page + 1
            )
        })
        .key("graph-page"),
    );
    if pages > 1 {
        paging = paging
            .child(
                command(
                    if searching {
                        "Previous results"
                    } else {
                        "Previous links"
                    },
                    "graph-previous",
                    |state| state.ui.graph.page = state.ui.graph.page.saturating_sub(1),
                )
                .disabled(page == 0),
            )
            .child(
                command(
                    if searching {
                        "Next results"
                    } else {
                        "Next links"
                    },
                    "graph-next",
                    move |state| state.ui.graph.page = (page + 1).min(pages - 1),
                )
                .disabled(page + 1 == pages),
            );
    }
    body = body.child(paging);
    let separate = width >= 900.0;
    let canvas_width = if separate { width - 344.0 } else { width };
    let canvas = if searching {
        let mut results = col().w_full().min_h(360.0).gap(12.0).py(16.0);
        for (index, node) in matches.iter().skip(page * 4).take(4).enumerate() {
            results = results.child(card(
                &data.nodes[*node],
                false,
                format!("graph-result-{index}"),
            ));
        }
        if matches.is_empty() {
            results = results.child(label("No matching nodes.")).child(command(
                "Clear search",
                "graph-clear",
                |state| {
                    state.ui.graph.search = TextEdit::default();
                    state.ui.graph.all = false;
                    state.ui.graph.page = 0;
                },
            ));
        }
        results
    } else {
        diagram(map, data, cx, canvas_width, &incoming, &outgoing, page)
    };
    let mut layout = if separate { row() } else { col() };
    layout = layout.w_full().gap(24.0).child(if separate {
        canvas.w(canvas_width)
    } else {
        canvas
    });
    layout = layout.child(if separate {
        detail(map, data).w(320.0)
    } else {
        detail(map, data).w_full()
    });
    body.child(layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};
    use serde_json::json;

    #[test]
    fn evidence_details_fit_the_real_content_width() {
        let item = super::super::provider::example();
        let id = item["case_id"].as_str().unwrap();
        for width in [1440.0, 1000.0, 560.0] {
            let mut state = State::idle();
            state.page = host::Page::Rea;
            state.host.selected = Some(id.into());
            state.host.snapshot = Some(
                host::Snapshot::parse(
                    json!({"ok":true,"version":"test","frontend":"test"}),
                    json!({"ok":true,"cases":[item.clone()]}),
                    json!({"profiles":{}}),
                )
                .unwrap(),
            );
            state.ui.inspector.step = 6;
            let data = Graph::parse(
                super::super::dotdog::parse(
                    include_bytes!("../../../../../web/hardware/workbench-map.json"),
                    id,
                )
                .unwrap(),
                id,
            )
            .unwrap();
            state.ui.graph = Map {
                selected: data.initial.clone(),
                data: Some(data),
                case: Some(id.into()),
                ..Default::default()
            };
            let mut runtime = Runtime::new(
                state,
                super::super::view,
                (width, 800.0),
                Env {
                    system_dark: true,
                    scale: 1.0,
                },
            );
            for _ in 0..3 {
                runtime.frame(0.0);
            }
            let elements = runtime.elements();
            let detail = elements
                .iter()
                .find(|element| element.key.as_deref() == Some("graph-node-title"))
                .unwrap();
            assert!(
                detail.rect[0] >= 0.0 && detail.rect[0] + detail.rect[2] <= width,
                "Detail clipped at {width}: {:?}",
                detail.rect
            );
        }
    }
}
