use rune::{App, El, col, el};

use super::{State, host::Page, secondary, workspace};

#[derive(Default)]
pub struct Workspace {
    path: String,
    revision: u64,
    reload: u64,
    error: Option<String>,
    pub stop_available: bool,
    stop: u64,
    pending_case: Option<String>,
}

impl Workspace {
    pub fn destination(&self) -> Option<&str> {
        self.path
            .split(['?', '#'])
            .next()?
            .trim_matches('/')
            .split('/')
            .next()
    }
}

fn valid_path(path: &str) -> bool {
    path.starts_with('/')
        && !path.starts_with("//")
        && !path.contains(['\\', '\r', '\n'])
        && path.len() <= 4096
}

pub fn open(state: &mut State, path: &str) {
    if !valid_path(path) {
        state.ui.notice = Some("Invalid local workspace route.".into());
        return;
    }
    if state.ui.embedded.path != path {
        state.ui.embedded.stop_available = false;
        state.ui.embedded.pending_case = state.host.selected.clone();
    }
    state.ui.embedded.path = path.into();
    state.ui.embedded.revision += 1;
    state.ui.embedded.error = None;
    state.ui.notice = None;
    state.ui.navigation_open = false;
    state.page = Page::Workspace;
}

pub fn reload(state: &mut State) {
    state.ui.embedded.reload += 1;
    state.ui.embedded.error = None;
}

pub fn stop(state: &mut State) {
    if state.ui.embedded.stop_available {
        state.ui.embedded.stop += 1;
        state.ui.embedded.stop_available = false;
    }
}

pub fn view(state: &State) -> El {
    if let Some(error) = &state.ui.embedded.error {
        return col()
            .w_full()
            .grow(1.0)
            .padding(24.0)
            .gap(16.0)
            .child(secondary(error).key("embedded-error"))
            .child(
                workspace::command("Retry workspace", true)
                    .on_press(|state: &mut State| {
                        let path = state.ui.embedded.path.clone();
                        open(state, &path);
                        reload(state);
                    })
                    .build(&super::theme::kit())
                    .h(44.0),
            );
    }
    el().key("embedded-workspace")
        .label("In-app compatibility workspace")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
}

#[cfg(target_os = "windows")]
const ORIGIN: &str = "https://reforge.localhost/";
#[cfg(not(target_os = "windows"))]
const ORIGIN: &str = "reforge://localhost/";

fn local_url(url: &str) -> bool {
    url.starts_with(ORIGIN) || url.starts_with(&format!("blob:{ORIGIN}"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn attach(app: App<State>, _port: u16) -> App<State> {
    app
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn attach(app: App<State>, port: u16) -> App<State> {
    #[cfg(target_os = "macos")]
    use objc2::{MainThreadMarker, MainThreadOnly, rc::Retained};
    #[cfg(target_os = "macos")]
    use objc2_app_kit::NSView;
    use rune::{AppHandle, Compositor, compose::Frame};
    use std::{cell::RefCell, rc::Rc};
    #[cfg(target_os = "macos")]
    use wry::WebViewExtMacOS;
    use wry::{
        NewWindowResponse, WebView, WebViewBuilder,
        dpi::{LogicalPosition, LogicalSize},
    };
    #[cfg(target_os = "windows")]
    use wry::{WebContext, WebViewBuilderExtWindows};

    struct Child {
        view: WebView,
        #[cfg(target_os = "macos")]
        container: Retained<NSView>,
        #[cfg(target_os = "macos")]
        parent: Retained<NSView>,
        revision: u64,
        reload: u64,
        visible: bool,
        bounds: [f32; 4],
        loaded: bool,
        stop: u64,
    }
    struct Host {
        child: Rc<RefCell<Option<Child>>>,
        #[cfg(all(feature = "agent-debug", target_os = "macos"))]
        inspector: Option<super::agent_debug::Inspector>,
    }
    impl Compositor<State> for Host {
        fn render(&mut self, frame: &mut Frame<'_, State>) {
            frame.draw_ui();
            let mut child = self.child.borrow_mut();
            #[cfg(all(feature = "agent-debug", target_os = "macos"))]
            if let Some(inspector) = self.inspector.as_mut() {
                inspector.poll(
                    frame,
                    child
                        .as_ref()
                        .filter(|child| child.loaded)
                        .map(|child| &child.view),
                );
            }
            let Some(child) = child.as_mut() else { return };
            let state = &frame.rt.state;
            let active = state.page == Page::Workspace
                && !state.ui.chooser
                && !state.ui.diagnostics
                && !state.ui.navigation_open
                && state.ui.embedded.error.is_none();
            let bounds = if active {
                frame
                    .rt
                    .elements()
                    .into_iter()
                    .find(|item| item.key.as_deref() == Some("embedded-workspace"))
                    .map(|item| item.visible)
            } else {
                None
            };
            let result = (|| -> Result<(), wry::Error> {
                if let Some(bounds) = bounds {
                    if child.stop != state.ui.embedded.stop {
                        child.view.evaluate_script(
                            "document.getElementById('simulation-estop')?.click()",
                        )?;
                        child.stop = state.ui.embedded.stop;
                    }
                    if child.bounds != bounds {
                        #[cfg(target_os = "macos")]
                        {
                            let mut rect = child.container.frame();
                            rect.origin.x = bounds[0] as f64;
                            rect.origin.y = if child.parent.isFlipped() {
                                bounds[1] as f64
                            } else {
                                child.parent.bounds().size.height
                                    - bounds[1] as f64
                                    - bounds[3] as f64
                            };
                            rect.size.width = bounds[2] as f64;
                            rect.size.height = bounds[3] as f64;
                            child.container.setFrame(rect);
                            #[cfg(any(debug_assertions, feature = "agent-debug"))]
                            let inspector_open = child.view.is_devtools_open();
                            #[cfg(not(any(debug_assertions, feature = "agent-debug")))]
                            let inspector_open = false;
                            if !inspector_open {
                                child.view.set_bounds(wry::Rect {
                                    position: LogicalPosition::new(0.0, 0.0).into(),
                                    size: LogicalSize::new(bounds[2] as f64, bounds[3] as f64)
                                        .into(),
                                })?;
                            }
                        }
                        #[cfg(target_os = "windows")]
                        child.view.set_bounds(wry::Rect {
                            position: LogicalPosition::new(bounds[0] as f64, bounds[1] as f64)
                                .into(),
                            size: LogicalSize::new(bounds[2] as f64, bounds[3] as f64).into(),
                        })?;
                        child.bounds = bounds;
                    }
                    if child.revision != state.ui.embedded.revision
                        || child.reload != state.ui.embedded.reload
                    {
                        let path = &state.ui.embedded.path;
                        if child.loaded && child.reload == state.ui.embedded.reload {
                            child.view.evaluate_script(&format!(
                                "window.__reaPendingWorkspace={0};if(document.body?.dataset.router==='ready'){{window.dispatchEvent(new CustomEvent('rea-workspace-open',{{detail:{0}}}))}}else{{location.assign({0})}}",
                                serde_json::to_string(path).expect("route JSON")))?;
                        } else {
                            child
                                .view
                                .load_url(&format!("{}{path}", ORIGIN.trim_end_matches('/')))?;
                            child.loaded = true;
                        }
                        child.revision = state.ui.embedded.revision;
                        child.reload = state.ui.embedded.reload;
                    }
                }
                let visible = bounds.is_some();
                if visible != child.visible {
                    child.view.evaluate_script(&format!(
                        "window.dispatchEvent(new CustomEvent('rea-workspace-visible',{{detail:{visible}}}))"))?;
                    #[cfg(target_os = "macos")]
                    child.container.setHidden(!visible);
                    child.view.set_visible(visible)?;
                    if visible {
                        child.view.focus()?;
                    } else {
                        child.view.focus_parent()?;
                    }
                    child.visible = visible;
                }
                Ok(())
            })();
            if let Err(error) = result {
                frame.rt.update(|state, _| {
                    state.ui.embedded.error = Some(format!("In-app workspace failed: {error}"))
                });
            }
        }
        fn draws_own(&self) -> bool {
            false
        }
    }

    fn create(
        window: &rune::WindowHandle,
        handle: AppHandle<State>,
        _port: u16,
        slot: &Rc<RefCell<Option<Child>>>,
    ) -> Result<(), String> {
        let failed = handle.clone();
        let inspector_enabled = super::backend_session::build_identity()["agent_debug"] == true
            || cfg!(debug_assertions) && std::env::var("REFORGE_AGENT_DEBUG").as_deref() == Ok("1");
        #[cfg(target_os = "windows")]
        let mut context = {
            let data = super::backend_session::project_root()
                .map_err(|error| error.to_string())?
                .join("webview");
            super::backend_session::private_directory(&data).map_err(|error| error.to_string())?;
            WebContext::new(Some(data))
        };
        let view = window
            .with_raw_handle(|raw| {
                #[cfg(target_os = "macos")]
                let builder = WebViewBuilder::new();
                #[cfg(target_os = "windows")]
                let builder =
                    WebViewBuilder::new_with_web_context(&mut context).with_https_scheme(true);
                builder
                    .with_custom_protocol("reforge".into(), |_id, request| {
                        let path = request
                            .uri()
                            .path_and_query()
                            .map(|value| value.as_str())
                            .unwrap_or("/");
                        let body = if request.body().is_empty() {
                            None
                        } else {
                            serde_json::from_slice(request.body()).ok()
                        };
                        let result = if request.body().len() > 1_048_576 {
                            Err("Request exceeds 1 MiB".into())
                        } else if !matches!(
                            *request.method(),
                            wry::http::Method::GET | wry::http::Method::POST
                        ) {
                            Err("Unsupported request method".into())
                        } else if request.method() != wry::http::Method::GET && body.is_none() {
                            Err("Invalid request body".into())
                        } else {
                            super::provider::request(path, body.as_ref())
                        };
                        let (status, bytes) = match result {
                            Ok(bytes) => (200, bytes),
                            Err(error) => (
                                404,
                                serde_json::json!({"ok":false,"error":error})
                                    .to_string()
                                    .into_bytes(),
                            ),
                        };
                        let mime = match path
                            .split('?')
                            .next()
                            .unwrap_or(path)
                            .rsplit('.')
                            .next()
                            .unwrap_or("")
                        {
                            "js" => "text/javascript",
                            "css" => "text/css",
                            "json" => "application/json",
                            "svg" => "image/svg+xml",
                            "png" => "image/png",
                            "jpg" | "jpeg" => "image/jpeg",
                            "webp" => "image/webp",
                            "wasm" => "application/wasm",
                            "glb" => "model/gltf-binary",
                            _ => {
                                if path.contains("/api/") {
                                    "application/json"
                                } else {
                                    "text/html"
                                }
                            }
                        };
                        wry::http::Response::builder()
                            .status(status)
                            .header("Content-Type", mime)
                            .header("X-Content-Type-Options", "nosniff")
                            .body(bytes.into())
                            .unwrap()
                    })
                    .with_visible(false)
                    .with_devtools(inspector_enabled)
                    .with_background_color((17, 25, 28, 255))
                    .with_initialization_script(
                        "Object.defineProperty(window,'__REA_RUNE_COMPAT__',{value:true});",
                    )
                    .with_initialization_script(include_str!(
                        "../../../../../desktop/client-diagnostics.js"
                    ))
                    .with_initialization_script(if inspector_enabled {
                        include_str!("../../../../../desktop/agent-debug.js")
                    } else {
                        ""
                    })
                    .with_ipc_handler(move |request| {
                        if request.body().len() > 16384 {
                            return;
                        }
                        let Ok(value) = serde_json::from_str::<serde_json::Value>(request.body())
                        else {
                            return;
                        };
                        let uri = request.uri().to_string();
                        handle.update(move |state, _| {
                            if !uri.starts_with(ORIGIN) {
                                return;
                            }
                            if value["kind"] == "client-error" {
                                super::diagnostics::client(&value.to_string());
                                return;
                            }
                            if value["kind"] != "workspace-state" {
                                return;
                            }
                            if state.page == Page::Workspace {
                                if state
                                    .ui
                                    .embedded
                                    .pending_case
                                    .as_deref()
                                    .is_some_and(|id| value["case"].as_str() != Some(id))
                                {
                                    return;
                                }
                                state.ui.embedded.pending_case = None;
                                state.ui.embedded.stop_available = value["stop_available"] == true;
                                if let Some(path) =
                                    value["path"].as_str().filter(|path| valid_path(path))
                                {
                                    state.ui.embedded.path = path.into();
                                }
                                if let Some(id) = value["case"].as_str()
                                    && state.host.selected.as_deref() != Some(id)
                                    && state.host.snapshot.as_ref().is_some_and(|snapshot| {
                                        snapshot.cases.iter().any(|item| item["case_id"] == id)
                                    })
                                {
                                    state.select_system(id.into());
                                }
                            }
                        });
                    })
                    .with_navigation_handler(move |url| {
                        if url == "about:blank" {
                            return true;
                        }
                        // Only the packaged local host owns application navigation.
                        let allowed = local_url(&url);
                        if !allowed {
                            failed.update(move |state, _| {
                                state.ui.notice =
                                    Some(format!("Reference navigation blocked: {url}"))
                            });
                        }
                        allowed
                    })
                    .with_new_window_req_handler(|_, _| NewWindowResponse::Deny)
                    .build_as_child(&raw)
            })?
            .map_err(|error| error.to_string())?;
        #[cfg(target_os = "macos")]
        let (container, parent) = {
            let mtm = MainThreadMarker::new().ok_or("Workspace must attach on the main thread")?;
            let webview = view.webview();
            let parent =
                unsafe { webview.superview() }.ok_or("Workspace parent view is unavailable")?;
            let container = NSView::initWithFrame(NSView::alloc(mtm), parent.bounds());
            parent.addSubview(&container);
            webview.removeFromSuperview();
            container.addSubview(&webview);
            container.setHidden(true);
            (container, parent)
        };
        *slot.borrow_mut() = Some(Child {
            view,
            #[cfg(target_os = "macos")]
            container,
            #[cfg(target_os = "macos")]
            parent,
            revision: 0,
            reload: 0,
            visible: false,
            bounds: [0.0; 4],
            loaded: false,
            stop: 0,
        });
        Ok(())
    }

    let slot = Rc::new(RefCell::new(None));
    let handle = app.handle();
    #[cfg(all(feature = "agent-debug", target_os = "macos"))]
    let inspector = match super::agent_debug::Inspector::start(handle.clone()) {
        Ok(inspector) => inspector,
        Err(error) => {
            handle.update(move |state, _| {
                state.ui.notice = Some(format!("Agent inspector unavailable: {error}"))
            });
            None
        }
    };
    let host = Host {
        child: slot.clone(),
        #[cfg(all(feature = "agent-debug", target_os = "macos"))]
        inspector,
    };
    app.compositor(host).on_window(move |window| {
        if let Err(error) = create(window, handle.clone(), port, &slot) {
            handle.update(move |state, _| {
                state.ui.embedded.error = Some(format!("Cannot create in-app workspace: {error}"))
            });
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_navigation_stays_in_app_and_rejects_foreign_routes() {
        let mut state = State::idle();
        open(&mut state, "/lab/?system=haas-lathe");
        assert!(state.page == Page::Workspace);
        assert_eq!(state.ui.embedded.destination(), Some("lab"));
        assert_eq!(state.ui.embedded.revision, 1);
        for bad in [
            "https://example.com",
            "//example.com",
            "/\\example.com",
            "/lab/\n",
        ] {
            open(&mut state, bad);
            assert_eq!(state.ui.embedded.path, "/lab/?system=haas-lathe");
            assert_eq!(state.ui.embedded.revision, 1);
        }
    }

    #[test]
    fn embedded_origin_excludes_foreign_hosts_and_schemes() {
        assert!(local_url(&format!("{ORIGIN}hardware/")));
        assert!(local_url(&format!("blob:{ORIGIN}local-download")));
        for url in [
            "https://reforge.localhost.evil.invalid/hardware/",
            "https://reforge.localhost@evil.invalid/hardware/",
            "http://reforge.localhost/hardware/",
            "reforge://localhost.evil.invalid/hardware/",
            "file:///hardware/",
            "javascript:alert(1)",
        ] {
            assert!(!local_url(url));
        }
    }
}
