use cssengine::Declaration;
use dioxus::prelude::try_consume_context;
use freya_core::prelude::{
    ContainerExt, ContainerSizeExt, EffectExt, Image, Label, Paragraph, Rect, StyleExt, Svg,
    TextStyleExt,
};
use torin::gaps::Gaps;

use crate::context::CssTheme;
use crate::convert;

// ---------------------------------------------------------------------------
// Public trait
// ---------------------------------------------------------------------------

/// Adds `.class()` to Freya element builders.
///
/// The class name is matched against the nearest [`CssTheme`] context.  A bare
/// name like `"card"` maps to the `.card` CSS selector; names that already
/// start with `.`, `#`, or `:` are used verbatim.  Multiple space-separated
/// class names are supported: `element.class("card hero")`.
///
/// # Availability
///
/// Implemented for [`Rect`], [`Label`], [`Paragraph`], [`Svg`], and [`Image`].
///
/// Pseudo-class styles (`:hover`, `:focus`, …) are collected at parse time
/// but not yet wired to Dioxus event handlers — base styles only.
pub trait CssClassExt: Sized {
    fn class(self, names: &str) -> Self;
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn to_selector(name: &str) -> String {
    if name.starts_with(['.', '#', ':']) {
        name.to_string()
    } else {
        format!(".{name}")
    }
}

/// Return the base (non-pseudo-class) declarations for a selector.
///
/// Reads via `Signal::read()` to subscribe the calling component to future
/// stylesheet changes, then locks the inner `Mutex` to call `get_styles`
/// (which requires `&mut StyleSheet` for the optional tailwind cache).
fn base_decls(theme: &CssTheme, selector: &str) -> Vec<Declaration> {
    let arc = theme.0.read();
    let mut ss = arc.lock().expect("cssengine stylesheet lock poisoned");
    ss.get_styles(selector)
        .into_iter()
        .filter(|(pseudo, _)| pseudo.is_none())
        .flat_map(|(_, decls)| decls)
        .collect()
}

// ---------------------------------------------------------------------------
// apply_full: Rect — background, border, shadow, layout, text, effects
// ---------------------------------------------------------------------------

fn apply_full<T>(mut element: T, decls: &[Declaration]) -> T
where
    T: StyleExt + TextStyleExt + ContainerExt + ContainerSizeExt + EffectExt,
{
    let mut padding = Gaps::new(0.0, 0.0, 0.0, 0.0);
    let mut has_padding = false;
    let mut margin = Gaps::new(0.0, 0.0, 0.0, 0.0);
    let mut has_margin = false;

    for decl in decls {
        element = match decl {
            // Background
            Declaration::BackgroundColor(c) => element.background(convert::css_color(c)),

            // Text / font
            Declaration::Color(c) => element.color(convert::css_color(c)),
            Declaration::FontSize(px) => element.font_size(px.0),
            Declaration::FontWeight(w) => element.font_weight(convert::font_weight(w)),
            Declaration::FontFamily(f) => element.font_family(f.clone()),
            Declaration::FontStyle(s) => element.font_slant(convert::font_slant(s)),
            Declaration::TextAlign(ta) => element.text_align(convert::text_align(ta)),

            // Dimensions
            Declaration::Width(v) => element.width(convert::pxpctauto_to_size(v)),
            Declaration::Height(v) => element.height(convert::pxpctauto_to_size(v)),
            Declaration::MinWidth(v) => element.min_width(convert::pxpctauto_to_size(v)),
            Declaration::MinHeight(v) => element.min_height(convert::pxpctauto_to_size(v)),
            Declaration::MaxWidth(v) => element.max_width(convert::pxpctauto_to_size(v)),
            Declaration::MaxHeight(v) => element.max_height(convert::pxpctauto_to_size(v)),

            // Padding (PxPct) — accumulate for a single Gaps call
            Declaration::Padding(v) => {
                let px = convert::to_px(v);
                padding = Gaps::new(px, px, px, px);
                has_padding = true;
                element
            }
            Declaration::PaddingTop(v) => {
                padding.top = convert::to_px(v);
                has_padding = true;
                element
            }
            Declaration::PaddingRight(v) => {
                padding.right = convert::to_px(v);
                has_padding = true;
                element
            }
            Declaration::PaddingBottom(v) => {
                padding.bottom = convert::to_px(v);
                has_padding = true;
                element
            }
            Declaration::PaddingLeft(v) => {
                padding.left = convert::to_px(v);
                has_padding = true;
                element
            }

            // Margin (PxPctAuto) — accumulate for a single Gaps call
            Declaration::Margin(v) => {
                let px = convert::to_px_auto(v);
                margin = Gaps::new(px, px, px, px);
                has_margin = true;
                element
            }
            Declaration::MarginTop(v) => {
                margin.top = convert::to_px_auto(v);
                has_margin = true;
                element
            }
            Declaration::MarginRight(v) => {
                margin.right = convert::to_px_auto(v);
                has_margin = true;
                element
            }
            Declaration::MarginBottom(v) => {
                margin.bottom = convert::to_px_auto(v);
                has_margin = true;
                element
            }
            Declaration::MarginLeft(v) => {
                margin.left = convert::to_px_auto(v);
                has_margin = true;
                element
            }

            // Visual
            Declaration::BorderRadius(v) => element.corner_radius(convert::border_radius(v)),
            Declaration::BoxShadow(s) => element.shadow(convert::box_shadow(s)),
            Declaration::Border(b) => element.border(Some(convert::border_def(b))),
            Declaration::Opacity(o) => element.opacity(*o),

            _ => element,
        };
    }

    if has_padding {
        element = element.padding(padding);
    }
    if has_margin {
        element = element.margin(margin);
    }

    element
}

// ---------------------------------------------------------------------------
// apply_text: Label / Paragraph — text + layout
// ---------------------------------------------------------------------------

fn apply_text<T>(mut element: T, decls: &[Declaration]) -> T
where
    T: TextStyleExt + ContainerExt + ContainerSizeExt,
{
    let mut padding = Gaps::new(0.0, 0.0, 0.0, 0.0);
    let mut has_padding = false;
    let mut margin = Gaps::new(0.0, 0.0, 0.0, 0.0);
    let mut has_margin = false;

    for decl in decls {
        element = match decl {
            Declaration::Color(c) => element.color(convert::css_color(c)),
            Declaration::FontSize(px) => element.font_size(px.0),
            Declaration::FontWeight(w) => element.font_weight(convert::font_weight(w)),
            Declaration::FontFamily(f) => element.font_family(f.clone()),
            Declaration::FontStyle(s) => element.font_slant(convert::font_slant(s)),
            Declaration::TextAlign(ta) => element.text_align(convert::text_align(ta)),

            Declaration::Width(v) => element.width(convert::pxpctauto_to_size(v)),
            Declaration::Height(v) => element.height(convert::pxpctauto_to_size(v)),
            Declaration::MinWidth(v) => element.min_width(convert::pxpctauto_to_size(v)),
            Declaration::MinHeight(v) => element.min_height(convert::pxpctauto_to_size(v)),
            Declaration::MaxWidth(v) => element.max_width(convert::pxpctauto_to_size(v)),
            Declaration::MaxHeight(v) => element.max_height(convert::pxpctauto_to_size(v)),

            Declaration::Padding(v) => {
                let px = convert::to_px(v);
                padding = Gaps::new(px, px, px, px);
                has_padding = true;
                element
            }
            Declaration::PaddingTop(v) => {
                padding.top = convert::to_px(v);
                has_padding = true;
                element
            }
            Declaration::PaddingRight(v) => {
                padding.right = convert::to_px(v);
                has_padding = true;
                element
            }
            Declaration::PaddingBottom(v) => {
                padding.bottom = convert::to_px(v);
                has_padding = true;
                element
            }
            Declaration::PaddingLeft(v) => {
                padding.left = convert::to_px(v);
                has_padding = true;
                element
            }

            Declaration::Margin(v) => {
                let px = convert::to_px_auto(v);
                margin = Gaps::new(px, px, px, px);
                has_margin = true;
                element
            }
            Declaration::MarginTop(v) => {
                margin.top = convert::to_px_auto(v);
                has_margin = true;
                element
            }
            Declaration::MarginRight(v) => {
                margin.right = convert::to_px_auto(v);
                has_margin = true;
                element
            }
            Declaration::MarginBottom(v) => {
                margin.bottom = convert::to_px_auto(v);
                has_margin = true;
                element
            }
            Declaration::MarginLeft(v) => {
                margin.left = convert::to_px_auto(v);
                has_margin = true;
                element
            }

            _ => element,
        };
    }

    if has_padding {
        element = element.padding(padding);
    }
    if has_margin {
        element = element.margin(margin);
    }

    element
}

// ---------------------------------------------------------------------------
// apply_layout: Svg — size + padding only
// ---------------------------------------------------------------------------

fn apply_layout<T>(mut element: T, decls: &[Declaration]) -> T
where
    T: ContainerExt + ContainerSizeExt,
{
    let mut padding = Gaps::new(0.0, 0.0, 0.0, 0.0);
    let mut has_padding = false;

    for decl in decls {
        element = match decl {
            Declaration::Width(v) => element.width(convert::pxpctauto_to_size(v)),
            Declaration::Height(v) => element.height(convert::pxpctauto_to_size(v)),
            Declaration::MinWidth(v) => element.min_width(convert::pxpctauto_to_size(v)),
            Declaration::MinHeight(v) => element.min_height(convert::pxpctauto_to_size(v)),
            Declaration::MaxWidth(v) => element.max_width(convert::pxpctauto_to_size(v)),
            Declaration::MaxHeight(v) => element.max_height(convert::pxpctauto_to_size(v)),

            Declaration::Padding(v) => {
                let px = convert::to_px(v);
                padding = Gaps::new(px, px, px, px);
                has_padding = true;
                element
            }
            _ => element,
        };
    }

    if has_padding {
        element = element.padding(padding);
    }

    element
}

// ---------------------------------------------------------------------------
// apply_effect: Image — size + visual effects
// ---------------------------------------------------------------------------

fn apply_effect<T>(mut element: T, decls: &[Declaration]) -> T
where
    T: EffectExt + ContainerExt + ContainerSizeExt,
{
    let mut padding = Gaps::new(0.0, 0.0, 0.0, 0.0);
    let mut has_padding = false;

    for decl in decls {
        element = match decl {
            Declaration::Width(v) => element.width(convert::pxpctauto_to_size(v)),
            Declaration::Height(v) => element.height(convert::pxpctauto_to_size(v)),
            Declaration::MinWidth(v) => element.min_width(convert::pxpctauto_to_size(v)),
            Declaration::MinHeight(v) => element.min_height(convert::pxpctauto_to_size(v)),
            Declaration::MaxWidth(v) => element.max_width(convert::pxpctauto_to_size(v)),
            Declaration::MaxHeight(v) => element.max_height(convert::pxpctauto_to_size(v)),
            Declaration::Opacity(o) => element.opacity(*o),

            Declaration::Padding(v) => {
                let px = convert::to_px(v);
                padding = Gaps::new(px, px, px, px);
                has_padding = true;
                element
            }
            _ => element,
        };
    }

    if has_padding {
        element = element.padding(padding);
    }

    element
}

// ---------------------------------------------------------------------------
// CssClassExt implementations
// ---------------------------------------------------------------------------

impl CssClassExt for Rect {
    fn class(self, names: &str) -> Self {
        let Some(theme) = try_consume_context::<CssTheme>() else {
            return self;
        };
        names.split_whitespace().fold(self, |el, name| {
            let decls = base_decls(&theme, &to_selector(name));
            apply_full(el, &decls)
        })
    }
}

impl CssClassExt for Label {
    fn class(self, names: &str) -> Self {
        let Some(theme) = try_consume_context::<CssTheme>() else {
            return self;
        };
        names.split_whitespace().fold(self, |el, name| {
            let decls = base_decls(&theme, &to_selector(name));
            apply_text(el, &decls)
        })
    }
}

impl CssClassExt for Paragraph {
    fn class(self, names: &str) -> Self {
        let Some(theme) = try_consume_context::<CssTheme>() else {
            return self;
        };
        names.split_whitespace().fold(self, |el, name| {
            let decls = base_decls(&theme, &to_selector(name));
            apply_text(el, &decls)
        })
    }
}

impl CssClassExt for Svg {
    fn class(self, names: &str) -> Self {
        let Some(theme) = try_consume_context::<CssTheme>() else {
            return self;
        };
        names.split_whitespace().fold(self, |el, name| {
            let decls = base_decls(&theme, &to_selector(name));
            apply_layout(el, &decls)
        })
    }
}

impl CssClassExt for Image {
    fn class(self, names: &str) -> Self {
        let Some(theme) = try_consume_context::<CssTheme>() else {
            return self;
        };
        names.split_whitespace().fold(self, |el, name| {
            let decls = base_decls(&theme, &to_selector(name));
            apply_effect(el, &decls)
        })
    }
}
