use csscolorparser::Color as CssColor;
use cssengine::{BorderDef, BoxShadow, PxPct, PxPctAuto, TextAlign as CssTextAlign, TextStyle, Weight};
use freya_core::prelude::{
    Border, Color, CornerRadius, FontSlant, FontWeight, Shadow, ShadowPosition, TextAlign,
};
use torin::gaps::Gaps;

// ---------------------------------------------------------------------------
// Color
// ---------------------------------------------------------------------------

/// Convert a `csscolorparser::Color` (f64 channels 0.0..=1.0) to freya's
/// packed-ARGB `Color`.
pub fn css_color(c: &CssColor) -> Color {
    Color::from_argb(
        (c.a * 255.0) as u8,
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
    )
}

// ---------------------------------------------------------------------------
// Lengths
// ---------------------------------------------------------------------------

/// Resolve a `PxPct` to pixels (percentage / relative units fall back to 0).
pub fn to_px(v: &PxPct) -> f32 {
    v.to_px_lossy()
}

/// Resolve a `PxPctAuto` to pixels (`Auto` → 0).
pub fn to_px_auto(v: &PxPctAuto) -> f32 {
    v.to_px_lossy()
}

/// Convert `PxPct` to the string form that Freya's layout engine accepts:
/// `"n"` for absolute pixels, `"n%"` for percentages.
pub fn pxpct_to_size(v: &PxPct) -> String {
    match v {
        PxPct::Px(n) => format!("{n}"),
        PxPct::Pct(n) => format!("{n}%"),
        other => format!("{}", other.to_px_lossy()),
    }
}

/// Convert `PxPctAuto` to the string form that Freya's layout engine accepts.
pub fn pxpctauto_to_size(v: &PxPctAuto) -> String {
    match v {
        PxPctAuto::Px(n) => format!("{n}"),
        PxPctAuto::Pct(n) => format!("{n}%"),
        PxPctAuto::Auto => "auto".to_string(),
        PxPctAuto::Zero => "0".to_string(),
        other => format!("{}", other.to_px_lossy()),
    }
}

// ---------------------------------------------------------------------------
// Gaps
// ---------------------------------------------------------------------------

/// Build a uniform `Gaps` value from a single px length.
pub fn uniform_gaps(px: f32) -> Gaps {
    Gaps::new(px, px, px, px)
}

// ---------------------------------------------------------------------------
// CornerRadius
// ---------------------------------------------------------------------------

pub fn border_radius(v: &PxPct) -> CornerRadius {
    CornerRadius::from(to_px(v))
}

// ---------------------------------------------------------------------------
// Shadow
// ---------------------------------------------------------------------------

pub fn box_shadow(s: &BoxShadow) -> Shadow {
    Shadow::new()
        .x(to_px(&s.h_offset))
        .y(to_px(&s.v_offset))
        .blur(to_px(&s.blur_radius))
        .spread(to_px(&s.spread))
        .color(css_color(&s.color))
        .position(ShadowPosition::Normal)
}

// ---------------------------------------------------------------------------
// Border
// ---------------------------------------------------------------------------

pub fn border_def(b: &BorderDef) -> Border {
    let width = b.width.as_ref().map(to_px).unwrap_or(1.0);
    let color = b.color.as_ref().map(css_color).unwrap_or_default();
    Border::new().fill(color).width(width)
}

// ---------------------------------------------------------------------------
// Font weight & style
// ---------------------------------------------------------------------------

/// Map a cssengine font weight to freya's `FontWeight`.
/// Both types are newtype wrappers around `u16`.
pub fn font_weight(w: &Weight) -> FontWeight {
    FontWeight(w.0)
}

/// Map a cssengine font style to freya's `FontSlant`.
pub fn font_slant(s: &TextStyle) -> FontSlant {
    match s {
        TextStyle::Normal => FontSlant::Normal,
        TextStyle::Italic => FontSlant::Italic,
        TextStyle::Oblique => FontSlant::Oblique,
    }
}

// ---------------------------------------------------------------------------
// Text alignment
// ---------------------------------------------------------------------------

pub fn text_align(ta: &CssTextAlign) -> TextAlign {
    match ta {
        CssTextAlign::Left => TextAlign::Left,
        CssTextAlign::Right => TextAlign::Right,
        CssTextAlign::Center => TextAlign::Center,
        CssTextAlign::Justify => TextAlign::Justify,
        CssTextAlign::Start => TextAlign::Start,
        CssTextAlign::End => TextAlign::End,
    }
}
