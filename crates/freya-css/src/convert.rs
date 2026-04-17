use cssengine::{
    BorderDef, BoxShadow, Color as CssColor, PxPct, PxPctAuto, TextAlign as CssTextAlign,
    TextStyle, Weight,
};
use freya_core::prelude::{
    Border, Color, CornerRadius, FontSlant, FontWeight, Shadow, ShadowPosition, TextAlign,
};
use torin::{gaps::Gaps, size::Size};

// ---------------------------------------------------------------------------
// Color
// ---------------------------------------------------------------------------

pub fn css_color(c: &CssColor) -> Color {
    Color::from_argb(
        (c.a * 255.0) as u8,
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
    )
}

// ---------------------------------------------------------------------------
// Lengths — raw px extraction
// ---------------------------------------------------------------------------

pub fn to_px(v: &PxPct) -> f32 {
    v.to_px_lossy()
}

pub fn to_px_auto(v: &PxPctAuto) -> f32 {
    v.to_px_lossy()
}

// ---------------------------------------------------------------------------
// Lengths — torin Size
// ---------------------------------------------------------------------------

pub fn pxpct_to_size(v: &PxPct) -> Size {
    match v {
        PxPct::Px(n) => Size::px(*n),
        PxPct::Pct(n) => Size::percent(*n),
        other => Size::px(other.to_px_lossy()),
    }
}

pub fn pxpctauto_to_size(v: &PxPctAuto) -> Size {
    match v {
        PxPctAuto::Px(n) => Size::px(*n),
        PxPctAuto::Pct(n) => Size::percent(*n),
        PxPctAuto::Auto => Size::auto(),
        PxPctAuto::Zero => Size::px(0.0),
        other => Size::px(other.to_px_lossy()),
    }
}

// ---------------------------------------------------------------------------
// Gaps
// ---------------------------------------------------------------------------

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

pub fn font_weight(w: &Weight) -> FontWeight {
    FontWeight(w.0 as i32)
}

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
