use rune::{Color, TextStyle};
use rune_kit::Kit;

const TOKENS: &str = include_str!("../../../../../web/ui/tokens.css");

fn token(name: &str) -> &'static str {
    TOKENS
        .split_once(&format!("{name}:"))
        .and_then(|(_, value)| value.split_once(';'))
        .map(|(value, _)| value.trim())
        .unwrap_or_else(|| panic!("Missing shared design token: {name}"))
}

pub fn color(name: &str) -> Color {
    rune::color::hex(token(name)).expect("shared REA color token must be valid")
}

pub fn size(name: &str) -> f32 {
    token(name)
        .strip_suffix("px")
        .and_then(|value| value.parse().ok())
        .expect("shared REA size token must be valid")
}

pub fn font(name: &str, weight: f32) -> TextStyle {
    TextStyle::new(size(name), weight)
}

pub fn code_spans(source: &str, language: Option<&str>) -> Vec<rune::Span> {
    use rune_kit::syntax::{self, Syntax};
    syntax::SyntaxTheme::from_kit(&kit())
        .with(Syntax::Keyword, color("--rb-code-keyword"))
        .with(Syntax::Type, color("--rb-code-type"))
        .with(Syntax::Number, color("--rb-code-number"))
        .with(Syntax::String, color("--rb-code-string"))
        .spans(&syntax::highlight(language, source), 0)
}

pub fn kit() -> Kit {
    let mut kit = Kit::neutral(true);
    kit.c.page = color("--rb-color-canvas");
    kit.c.bg_base = color("--rb-color-surface");
    kit.c.bg_deep = color("--rb-color-field");
    kit.c.layer_01 = color("--rb-color-surface");
    kit.c.layer_02 = color("--rb-color-raised");
    kit.c.layer_03 = color("--rb-color-raised");
    kit.c.surface_base = color("--rb-color-field");
    kit.c.surface_raised = color("--rb-color-raised");
    kit.c.surface_float = color("--rb-color-surface");
    kit.c.border_base = color("--rb-color-border");
    kit.c.border_muted = color("--rb-color-divider");
    kit.c.border_strong = color("--rb-color-control-edge");
    kit.c.border_weak = color("--rb-color-border");
    kit.c.border_active = color("--rb-color-selection-edge");
    kit.c.border_hover = color("--rb-color-control-edge");
    kit.c.text_weak = color("--rb-color-secondary");
    kit.c.text_strong = color("--rb-color-text");
    kit.c.icon_base = color("--rb-color-text");
    kit.c.icon_muted = color("--rb-color-secondary");
    kit.c.icon_weaker = color("--rb-color-secondary");
    kit.c.scrim = color("--rb-color-scrim");
    kit.elevation.button_neutral.clear();
    kit.elevation.button_contrast.clear();
    kit.c.bg_button_neutral = color("--rb-color-raised");
    kit.c.text_base = color("--rb-color-text");
    kit.c.text_muted = color("--rb-color-secondary");
    kit.c.bg_contrast = color("--rb-color-action");
    kit.c.text_contrast = color("--rb-color-on-action");
    kit.c.border_focus = color("--rb-color-focus");
    kit.c.hover = color("--rb-color-hover");
    kit.c.pressed = color("--rb-color-selected");
    kit.ty.ui = font("--rb-text-body", 500.0);
    kit.ty.ui_medium = font("--rb-text-secondary", 500.0);
    kit.radius.md = size("--rb-radius-control");
    kit
}
