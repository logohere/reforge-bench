use std::sync::mpsc::{self, Receiver};

use rune::{Cx, El, Image, ImageCache, col, el, row};
use rune_kit::{Button, ButtonVariant, Collapsible};
use serde_json::Value;

use super::{State, catalog, host, label, local_http, secondary, theme, workspace};
use theme::{color, font};

type Reply = Result<(Value, Vec<u8>), String>;

pub struct Controller {
    build: &'static str,
    phase: &'static str,
    target: Option<String>,
    data: Option<Value>,
    image: Option<Image>,
    receiver: Option<Receiver<Reply>>,
    image_ready: bool,
    error: Option<String>,
    details: bool,
    started: Option<std::time::Instant>,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            build: "original",
            phase: "final",
            target: None,
            data: None,
            image: None,
            receiver: None,
            image_ready: false,
            error: None,
            details: false,
            started: None,
        }
    }
}

impl Controller {
    pub fn loading(&self) -> bool {
        self.receiver.is_some() || self.image.is_some() && !self.image_ready
    }
}

fn validate(data: &Value, build: &str, phase: &str) -> Result<String, String> {
    if data["ok"] != true
        || data["source"] != "retained_oem_qemu_rtl"
        || data["firmware_sha256"] != catalog::DDCS_SHA
        || data["build"] != build
        || !data["record_count"].as_u64().is_some_and(|n| n <= 65536)
        || ![
            "executable_sha256",
            "program_sha256",
            "setting_sha256",
            "trace_sha256",
        ]
        .iter()
        .all(|key| data[key].as_str().is_some_and(catalog::valid_id))
        || build == "original" && data["executable_sha256"] != catalog::DDCS_SHA
    {
        return Err("Saved display does not identify the selected DDCS build".into());
    }
    let screen = data["screens"]
        .as_array()
        .and_then(|screens| screens.iter().find(|screen| screen["phase"] == phase))
        .ok_or("Selected saved display is unavailable")?;
    if screen["width"] != 1024
        || screen["height"] != 600
        || !screen["png_sha256"].as_str().is_some_and(catalog::valid_id)
        || !screen["framebuffer_sha256"]
            .as_str()
            .is_some_and(catalog::valid_id)
        || !screen["captured_at"]
            .as_str()
            .is_some_and(|date| chrono::DateTime::parse_from_rfc3339(date).is_ok())
        || !screen["after_sequence"]
            .as_u64()
            .zip(data["record_count"].as_u64())
            .is_some_and(|(a, b)| a <= b)
    {
        return Err("Saved display has invalid image or record identity".into());
    }
    Ok(screen["png_sha256"].as_str().unwrap().into())
}

pub fn preload(state: &mut State, force: bool) {
    if state.host.selected.as_deref() != Some(catalog::DDCS_SHA) {
        state.ui.controller = Controller::default();
        return;
    }
    let controller = &mut state.ui.controller;
    if !force && controller.target.as_deref() == Some(catalog::DDCS_SHA) {
        return;
    }
    controller.target = state.host.selected.clone();
    controller.error = None;
    controller.started = Some(std::time::Instant::now());
    let build = controller.build;
    let phase = controller.phase;
    let port = state.host.port;
    let expected = state
        .host
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.health.clone());
    let wake = state.host.waker.clone();
    let (sender, receiver) = mpsc::channel();
    controller.receiver = Some(receiver);
    std::thread::spawn(move || {
        let result = (|| {
            let before = host::read(port, "/api/desktop/health")?;
            if expected.as_ref() != Some(&before) {
                return Err("Backend changed. Refresh the connection first".into());
            }
            let data: Value = serde_json::from_slice(&local_http::transfer(
                port,
                &format!("/api/oem/retained?build={build}"),
                None,
                1_048_576,
                5,
            )?)
            .map_err(|_| "Invalid saved controller response")?;
            let hash = validate(&data, build, phase)?;
            let bytes = local_http::transfer(
                port,
                &format!("/api/oem/retained/frame?build={build}&phase={phase}&sha256={hash}"),
                None,
                2 * 1_048_576,
                5,
            )?;
            if crate::sha256(&bytes) != hash {
                return Err("Saved display bytes do not match their receipt".into());
            }
            if before != host::read(port, "/api/desktop/health")? {
                return Err("Backend changed while loading the display. Refresh and retry".into());
            }
            Ok((data, bytes))
        })();
        let _ = sender.send(result);
        if let Some(wake) = wake {
            wake();
        }
    });
}

pub fn poll(state: &mut State) -> bool {
    let controller = &mut state.ui.controller;
    let reply = controller
        .receiver
        .as_ref()
        .and_then(|receiver| receiver.try_recv().ok());
    let mut changed = reply.is_some();
    if let Some(reply) = reply {
        controller.receiver = None;
        if state.host.selected.as_deref() != controller.target.as_deref() {
            return true;
        }
        match reply {
            Ok((data, bytes)) => {
                controller.data = Some(data);
                let wake = state
                    .host
                    .waker
                    .clone()
                    .unwrap_or_else(|| std::sync::Arc::new(|| {}));
                controller.image = Some(ImageCache::new(wake).load(bytes));
                controller.image_ready = false;
                controller.error = None;
            }
            Err(error) => {
                controller.error = Some(error);
                controller.started = None;
            }
        }
    }
    if let Some(image) = &controller.image
        && image.is_ready()
        && !controller.image_ready
    {
        controller.image_ready = true;
        if image.size() != Some([1024, 600]) {
            controller.error =
                Some("Saved controller image is damaged or has the wrong dimensions".into());
        } else if let Some(started) = controller.started.take() {
            super::backend_session::record_metric(
                &serde_json::json!({"event":"performance",
                "name":"native-retained-display-ready","ms":started.elapsed().as_secs_f64()*1000.0})
                .to_string(),
            );
        }
        changed = true;
    }
    changed
}

pub fn view(state: &State, cx: &Cx<State>, width: f32) -> El {
    let controller = &state.ui.controller;
    if state.host.selected.as_deref() != Some(catalog::DDCS_SHA) {
        return col()
            .w_full()
            .gap(16.0)
            .child(label(
                "This selected system has no qualified native controller display.",
            ))
            .child(workspace::route_button("Open existing Lab", "lab", false));
    }
    let mut toolbar = row().w_full().flex_wrap().gap(8.0);
    for (title, build) in [
        ("Original", "original"),
        ("24-entry hybrid", "reconstructed"),
    ] {
        toolbar = toolbar.child(
            Button::new(title)
                .variant(ButtonVariant::Neutral)
                .on_press(move |state: &mut State| {
                    state.ui.controller.build = build;
                    state.ui.controller.image = None;
                    state.ui.controller.data = None;
                    preload(state, true);
                })
                .build(&theme::kit())
                .key(format!("controller-build-{build}"))
                .fill(color(if controller.build == build {
                    "--rb-color-selected"
                } else {
                    "--rb-color-raised"
                }))
                .h(40.0),
        );
    }
    for (title, phase) in [("Startup", "startup"), ("Final", "final")] {
        toolbar = toolbar.child(
            Button::new(title)
                .variant(ButtonVariant::Neutral)
                .on_press(move |state: &mut State| {
                    state.ui.controller.phase = phase;
                    state.ui.controller.image = None;
                    state.ui.controller.data = None;
                    preload(state, true);
                })
                .build(&theme::kit())
                .key(format!("controller-phase-{phase}"))
                .fill(color(if controller.phase == phase {
                    "--rb-color-selected"
                } else {
                    "--rb-color-raised"
                }))
                .h(40.0),
        );
    }
    let mut content = col()
        .w_full()
        .gap(16.0)
        .child(workspace::badge("Saved OEM display · not live", "info").key("controller-origin"))
        .child(toolbar.child(workspace::route_button(
            "Open full controller workflow",
            "lab",
            false,
        )));
    if let Some(error) = &controller.error {
        content = content
            .child(
                label(error)
                    .key("controller-error")
                    .color(color("--rb-color-danger")),
            )
            .child(
                workspace::command("Retry display", true)
                    .disabled(controller.receiver.is_some())
                    .on_press(|state: &mut State| preload(state, true))
                    .build(&theme::kit())
                    .key("controller-retry")
                    .h(44.0),
            );
    }
    let viewport_width = cx
        .layout_size("controller-viewport")
        .map_or(width, |size| size[0]);
    let window_height = cx
        .layout_size("workspace-root")
        .map_or(800.0, |size| size[1]);
    let image_width = viewport_width.min((window_height - 360.0).max(180.0) * 1024.0 / 600.0);
    let image = if let Some(image) = &controller.image
        && image.size() == Some([1024, 600])
    {
        el().pixels(image.pixels())
            .key("controller-frame")
            .role("img")
            .label("Hash-verified saved DDCS firmware framebuffer; not a live controller")
            .w(image_width)
            .h(image_width * 600.0 / 1024.0)
    } else {
        col().w_full().h(240.0).center().child(secondary(
            if controller.receiver.is_some()
                || controller.image.is_some() && !controller.image_ready
            {
                "Loading the saved controller display…"
            } else {
                "No verified saved display loaded"
            },
        ))
    };
    content = content.child(
        col()
            .key("controller-viewport")
            .w_full()
            .items_center()
            .fill(color("--rb-color-field"))
            .child(image),
    );
    if let Some(data) = &controller.data {
        let screen = data["screens"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["phase"] == controller.phase)
            .unwrap();
        content=content.child(secondary(format!("{} · {} of {} acknowledged contour records",screen["captured_at"].as_str().unwrap(),screen["after_sequence"],data["record_count"])))
            .child(secondary("This retained contour run is separate from the selected Rust gear. No job starts when this display loads."))
            .child(Collapsible::new("Display provenance",controller.details)
                .on_toggle(|state: &mut State| state.ui.controller.details = !state.ui.controller.details)
                .child(secondary(serde_json::to_string_pretty(data).unwrap()).font(font("--rb-text-secondary",400.0).mono()).wrap_anywhere())
                .build(&theme::kit()));
    }
    if controller.phase == "startup" {
        content = content.child(secondary("Boot capture: the retained frame is blank before the OEM interface draws. Final shows the completed run's display."));
    }
    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn saved_controller_requires_exact_target_build_frame_and_record_identity() {
        let mut value = json!({"ok":true,"source":"retained_oem_qemu_rtl","build":"original","firmware_sha256":catalog::DDCS_SHA,
            "executable_sha256":catalog::DDCS_SHA,"program_sha256":"a".repeat(64),"setting_sha256":"b".repeat(64),"trace_sha256":"c".repeat(64),
            "record_count":6608,"screens":[{"phase":"startup","width":1024,"height":600,"png_sha256":"d".repeat(64),"framebuffer_sha256":"e".repeat(64),"captured_at":"2026-11-01T00:00:00Z","after_sequence":0}]});
        assert_eq!(
            validate(&value, "original", "startup").unwrap(),
            "d".repeat(64)
        );
        assert!(validate(&value, "reconstructed", "startup").is_err());
        assert!(validate(&value, "original", "final").is_err());
        value["screens"][0]["after_sequence"] = json!(6609);
        assert!(validate(&value, "original", "startup").is_err());
        value["screens"][0]["after_sequence"] = json!(0);
        value["firmware_sha256"] = json!("f".repeat(64));
        assert!(validate(&value, "original", "startup").is_err());
    }
}
