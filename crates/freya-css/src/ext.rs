use cssengine::Declaration;
use freya_core::prelude::{
    ContainerExt, ContainerSizeExt, EffectExt, Label, Paragraph, Rect, StyleExt, Svg, TextStyleExt,
};
use torin::gaps::Gaps;

use crate::convert;
use crate::theme::base_decls;

// ---------------------------------------------------------------------------
// Public trait
// ---------------------------------------------------------------------------

/// Adds `.class()` to Freya element builders.
///
/// A bare name like `"card"` maps to the `.card` CSS selector; names already
/// starting with `.`, `#`, or `:` are used verbatim.  Multiple space-separated
/// class names are supported: `element.class("card hero")`.
///
/// Implemented for [`Rect`], [`Label`], [`Paragraph`], and [`Svg`].
pub trait CssClassExt: Sized {
    fn class(self, names: &str) -> Self;
}

// ---------------------------------------------------------------------------
// Selector helper
// ---------------------------------------------------------------------------

fn to_selector(name: &str) -> String {
    if name.starts_with(['.', '#', ':']) {
        name.to_string()
    } else {
        format!(".{name}")
    }
}

// ---------------------------------------------------------------------------
// Gaps accumulator — deferred Gaps construction from individual sides
// ---------------------------------------------------------------------------

struct GapsAccum {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
    dirty: bool,
}

impl GapsAccum {
    fn new() -> Self {
        Self { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0, dirty: false }
    }

    fn set_all(&mut self, v: f32) {
        self.top = v;
        self.right = v;
        self.bottom = v;
        self.left = v;
        self.dirty = true;
    }

    fn into_gaps(self) -> Gaps {
        Gaps::new(self.top, self.right, self.bottom, self.left)
    }
}

// ---------------------------------------------------------------------------
// apply_full: StyleExt + TextStyleExt + ContainerExt + ContainerSizeExt + EffectExt
// ---------------------------------------------------------------------------

pub(crate) fn apply_full<E>(mut el: E, decls: &[Declaration]) -> E
where
    E: StyleExt + TextStyleExt + ContainerExt + ContainerSizeExt + EffectExt,
{
    let mut pad = GapsAccum::new();
    let mut mar = GapsAccum::new();

    for decl in decls {
        el = match decl {
            Declaration::BackgroundColor(c) => el.background(convert::css_color(c)),

            Declaration::Color(c) => el.color(convert::css_color(c)),
            Declaration::FontSize(px) => el.font_size(px.0),
            Declaration::FontWeight(w) => el.font_weight(convert::font_weight(w)),
            Declaration::FontFamily(f) => el.font_family(f.clone()),
            Declaration::FontStyle(s) => el.font_slant(convert::font_slant(s)),
            Declaration::TextAlign(ta) => el.text_align(convert::text_align(ta)),

            Declaration::Width(v) => el.width(convert::pxpctauto_to_size(v)),
            Declaration::Height(v) => el.height(convert::pxpctauto_to_size(v)),
            Declaration::MinWidth(v) => el.min_width(convert::pxpctauto_to_size(v)),
            Declaration::MinHeight(v) => el.min_height(convert::pxpctauto_to_size(v)),
            Declaration::MaxWidth(v) => el.max_width(convert::pxpctauto_to_size(v)),
            Declaration::MaxHeight(v) => el.max_height(convert::pxpctauto_to_size(v)),

            Declaration::Padding(v) => { pad.set_all(convert::to_px(v)); el }
            Declaration::PaddingTop(v) => { pad.top = convert::to_px(v); pad.dirty = true; el }
            Declaration::PaddingRight(v) => { pad.right = convert::to_px(v); pad.dirty = true; el }
            Declaration::PaddingBottom(v) => { pad.bottom = convert::to_px(v); pad.dirty = true; el }
            Declaration::PaddingLeft(v) => { pad.left = convert::to_px(v); pad.dirty = true; el }

            Declaration::Margin(v) => { mar.set_all(convert::to_px_auto(v)); el }
            Declaration::MarginTop(v) => { mar.top = convert::to_px_auto(v); mar.dirty = true; el }
            Declaration::MarginRight(v) => { mar.right = convert::to_px_auto(v); mar.dirty = true; el }
            Declaration::MarginBottom(v) => { mar.bottom = convert::to_px_auto(v); mar.dirty = true; el }
            Declaration::MarginLeft(v) => { mar.left = convert::to_px_auto(v); mar.dirty = true; el }

            Declaration::BorderRadius(v) => el.corner_radius(convert::border_radius(v)),
            Declaration::BoxShadow(s) => el.shadow(convert::box_shadow(s)),
            Declaration::Border(b) => el.border(Some(convert::border_def(b))),
            Declaration::Opacity(o) => el.opacity(*o),

            _ => el,
        };
    }

    if pad.dirty { el = el.padding(pad.into_gaps()); }
    if mar.dirty { el = el.margin(mar.into_gaps()); }
    el
}

// ---------------------------------------------------------------------------
// apply_text: TextStyleExt + ContainerExt + ContainerSizeExt
// ---------------------------------------------------------------------------

pub(crate) fn apply_text<E>(mut el: E, decls: &[Declaration]) -> E
where
    E: TextStyleExt + ContainerExt + ContainerSizeExt,
{
    let mut pad = GapsAccum::new();
    let mut mar = GapsAccum::new();

    for decl in decls {
        el = match decl {
            Declaration::Color(c) => el.color(convert::css_color(c)),
            Declaration::FontSize(px) => el.font_size(px.0),
            Declaration::FontWeight(w) => el.font_weight(convert::font_weight(w)),
            Declaration::FontFamily(f) => el.font_family(f.clone()),
            Declaration::FontStyle(s) => el.font_slant(convert::font_slant(s)),
            Declaration::TextAlign(ta) => el.text_align(convert::text_align(ta)),

            Declaration::Width(v) => el.width(convert::pxpctauto_to_size(v)),
            Declaration::Height(v) => el.height(convert::pxpctauto_to_size(v)),
            Declaration::MinWidth(v) => el.min_width(convert::pxpctauto_to_size(v)),
            Declaration::MinHeight(v) => el.min_height(convert::pxpctauto_to_size(v)),
            Declaration::MaxWidth(v) => el.max_width(convert::pxpctauto_to_size(v)),
            Declaration::MaxHeight(v) => el.max_height(convert::pxpctauto_to_size(v)),

            Declaration::Padding(v) => { pad.set_all(convert::to_px(v)); el }
            Declaration::PaddingTop(v) => { pad.top = convert::to_px(v); pad.dirty = true; el }
            Declaration::PaddingRight(v) => { pad.right = convert::to_px(v); pad.dirty = true; el }
            Declaration::PaddingBottom(v) => { pad.bottom = convert::to_px(v); pad.dirty = true; el }
            Declaration::PaddingLeft(v) => { pad.left = convert::to_px(v); pad.dirty = true; el }

            Declaration::Margin(v) => { mar.set_all(convert::to_px_auto(v)); el }
            Declaration::MarginTop(v) => { mar.top = convert::to_px_auto(v); mar.dirty = true; el }
            Declaration::MarginRight(v) => { mar.right = convert::to_px_auto(v); mar.dirty = true; el }
            Declaration::MarginBottom(v) => { mar.bottom = convert::to_px_auto(v); mar.dirty = true; el }
            Declaration::MarginLeft(v) => { mar.left = convert::to_px_auto(v); mar.dirty = true; el }

            _ => el,
        };
    }

    if pad.dirty { el = el.padding(pad.into_gaps()); }
    if mar.dirty { el = el.margin(mar.into_gaps()); }
    el
}

// ---------------------------------------------------------------------------
// apply_layout: ContainerExt + ContainerSizeExt (Svg)
// ---------------------------------------------------------------------------

pub(crate) fn apply_layout<E>(mut el: E, decls: &[Declaration]) -> E
where
    E: ContainerExt + ContainerSizeExt,
{
    let mut pad = GapsAccum::new();

    for decl in decls {
        el = match decl {
            Declaration::Width(v) => el.width(convert::pxpctauto_to_size(v)),
            Declaration::Height(v) => el.height(convert::pxpctauto_to_size(v)),
            Declaration::MinWidth(v) => el.min_width(convert::pxpctauto_to_size(v)),
            Declaration::MinHeight(v) => el.min_height(convert::pxpctauto_to_size(v)),
            Declaration::MaxWidth(v) => el.max_width(convert::pxpctauto_to_size(v)),
            Declaration::MaxHeight(v) => el.max_height(convert::pxpctauto_to_size(v)),
            Declaration::Padding(v) => { pad.set_all(convert::to_px(v)); el }
            _ => el,
        };
    }

    if pad.dirty { el = el.padding(pad.into_gaps()); }
    el
}

// ---------------------------------------------------------------------------
// Macro: reduce CssClassExt boilerplate
// ---------------------------------------------------------------------------

macro_rules! impl_css_class_ext {
    ($type:ty, $apply:ident) => {
        impl CssClassExt for $type {
            fn class(self, names: &str) -> Self {
                names.split_whitespace().fold(self, |el, name| {
                    let decls = base_decls(&to_selector(name));
                    $apply(el, &decls)
                })
            }
        }
    };
}

impl_css_class_ext!(Rect, apply_full);
impl_css_class_ext!(Label, apply_text);
impl_css_class_ext!(Paragraph, apply_text);
impl_css_class_ext!(Svg, apply_layout);
