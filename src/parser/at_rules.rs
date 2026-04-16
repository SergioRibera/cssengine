/// Parsed representation of CSS at-rules.
///
/// At-rules are parsed and stored as structured data.
/// The caller is responsible for evaluating conditions like media queries.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{declaration::Declaration, PxPct, Selector, TextStyle, Weight};
use super::values::parse_px_pct;

// ---------------------------------------------------------------------------
// MediaQuery
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum ColorScheme {
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum MediaFeature {
    MaxWidth(PxPct),
    MinWidth(PxPct),
    MaxHeight(PxPct),
    MinHeight(PxPct),
    Width(PxPct),
    Height(PxPct),
    Orientation(Orientation),
    PrefersColorScheme(ColorScheme),
    PrefersReducedMotion,
    HoverCapable,
    /// Unknown or unsupported feature — stored raw for future use
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum MediaQuery {
    All,
    Screen,
    Print,
    Feature(MediaFeature),
    Not(Box<MediaQuery>),
    And(Box<MediaQuery>, Box<MediaQuery>),
    Or(Box<MediaQuery>, Box<MediaQuery>),
}

impl MediaQuery {
    /// Evaluate this query against a `MediaContext`.
    pub fn matches(&self, ctx: &MediaContext) -> bool {
        match self {
            MediaQuery::All => true,
            MediaQuery::Screen => ctx.is_screen,
            MediaQuery::Print => !ctx.is_screen,
            MediaQuery::Feature(f) => f.matches(ctx),
            MediaQuery::Not(q) => !q.matches(ctx),
            MediaQuery::And(a, b) => a.matches(ctx) && b.matches(ctx),
            MediaQuery::Or(a, b) => a.matches(ctx) || b.matches(ctx),
        }
    }
}

impl MediaFeature {
    pub fn matches(&self, ctx: &MediaContext) -> bool {
        match self {
            MediaFeature::MaxWidth(v) => ctx.width <= v.to_px_lossy(),
            MediaFeature::MinWidth(v) => ctx.width >= v.to_px_lossy(),
            MediaFeature::MaxHeight(v) => ctx.height <= v.to_px_lossy(),
            MediaFeature::MinHeight(v) => ctx.height >= v.to_px_lossy(),
            MediaFeature::Width(v) => (ctx.width - v.to_px_lossy()).abs() < 0.5,
            MediaFeature::Height(v) => (ctx.height - v.to_px_lossy()).abs() < 0.5,
            MediaFeature::Orientation(o) => match o {
                Orientation::Portrait => ctx.height >= ctx.width,
                Orientation::Landscape => ctx.width > ctx.height,
            },
            MediaFeature::PrefersColorScheme(s) => match s {
                ColorScheme::Dark => ctx.prefers_dark,
                ColorScheme::Light => !ctx.prefers_dark,
            },
            MediaFeature::PrefersReducedMotion => ctx.prefers_reduced_motion,
            MediaFeature::HoverCapable => ctx.hover_capable,
            MediaFeature::Unknown(_) => false,
        }
    }
}

/// Runtime context for evaluating media queries.
#[derive(Debug, Clone)]
pub struct MediaContext {
    pub width: f32,
    pub height: f32,
    pub is_screen: bool,
    pub prefers_dark: bool,
    pub prefers_reduced_motion: bool,
    pub hover_capable: bool,
}

impl Default for MediaContext {
    fn default() -> Self {
        Self {
            width: 1920.0,
            height: 1080.0,
            is_screen: true,
            prefers_dark: false,
            prefers_reduced_motion: false,
            hover_capable: true,
        }
    }
}

// ---------------------------------------------------------------------------
// MediaRule
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MediaRule {
    pub query: MediaQuery,
    /// (selector, source_index, declarations) — same format as StyleSheet::rules
    pub rules: Vec<(Selector, usize, Vec<Declaration>)>,
}

// ---------------------------------------------------------------------------
// Keyframes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct KeyframeStop {
    /// Position from 0.0 (from/0%) to 1.0 (to/100%)
    pub at: f32,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Keyframes {
    pub name: String,
    pub stops: Vec<KeyframeStop>,
}

// ---------------------------------------------------------------------------
// FontFace
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum FontSource {
    Url(String),
    Local(String),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct FontFace {
    pub family: String,
    pub src: Vec<FontSource>,
    pub weight: Option<Weight>,
    pub style: Option<TextStyle>,
}

// ---------------------------------------------------------------------------
// ImportRule
// ---------------------------------------------------------------------------

/// A parsed `@import` rule.
///
/// ```css
/// @import "base.css";
/// @import url("mobile.css") (max-width: 768px);
/// @import "print.css" print;
/// ```
///
/// The library does not fetch URLs — use `StyleSheet::resolve_imports` to
/// supply content via a loader callback.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ImportRule {
    /// The URL or path exactly as written in the stylesheet.
    pub url: String,
    /// Optional media condition. When `None` the import is unconditional.
    pub media: Option<MediaQuery>,
}

/// Parse an `@import` prelude (the text between `@import` and `;`).
///
/// Returns `None` if the prelude is not a valid `@import` declaration.
pub fn parse_import_rule(prelude: &str) -> Option<ImportRule> {
    let (url, rest) = extract_import_url(prelude.trim())?;
    let media = if rest.trim().is_empty() {
        None
    } else {
        Some(parse_media_query(rest.trim()))
    };
    Some(ImportRule { url, media })
}

/// Extract the URL string and the remaining text (media condition) from an
/// `@import` prelude.
fn extract_import_url(s: &str) -> Option<(String, &str)> {
    if let Some(after) = s.strip_prefix("url(") {
        // url( "..." ) or url( '...' ) or url( bare )
        let close = after.find(')')?;
        let url = after[..close]
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_owned();
        let rest = after[close + 1..].trim_start();
        Some((url, rest))
    } else if let Some(inner) = s.strip_prefix('"') {
        let end = inner.find('"')?;
        let url = inner[..end].to_owned();
        let rest = inner[end + 1..].trim_start();
        Some((url, rest))
    } else if let Some(inner) = s.strip_prefix('\'') {
        let end = inner.find('\'')?;
        let url = inner[..end].to_owned();
        let rest = inner[end + 1..].trim_start();
        Some((url, rest))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// MediaQuery parser
// ---------------------------------------------------------------------------

/// Parse an `@media` prelude string into a `MediaQuery`.
pub fn parse_media_query(prelude: &str) -> MediaQuery {
    let s = prelude.trim();
    if s.is_empty() || s == "all" {
        return MediaQuery::All;
    }
    if s == "screen" { return MediaQuery::Screen; }
    if s == "print" { return MediaQuery::Print; }

    // Handle `not ...`
    if let Some(rest) = s.strip_prefix("not ") {
        return MediaQuery::Not(Box::new(parse_media_query(rest)));
    }

    // Handle comma-separated OR: `screen, print`
    if s.contains(',') {
        let mut parts = s.splitn(2, ',');
        let lhs = parse_media_query(parts.next().unwrap_or("all"));
        let rhs = parse_media_query(parts.next().unwrap_or("all"));
        return MediaQuery::Or(Box::new(lhs), Box::new(rhs));
    }

    // Handle `and`: `screen and (max-width: 600px)`
    if let Some(idx) = s.find(" and ") {
        let lhs = parse_media_query(&s[..idx]);
        let rhs = parse_media_query(&s[idx + 5..]);
        return MediaQuery::And(Box::new(lhs), Box::new(rhs));
    }

    // Handle `(feature: value)` or `(feature)`
    if let Some(inner) = s.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
        return MediaQuery::Feature(parse_media_feature(inner));
    }

    // Fallback: treat as unknown feature
    MediaQuery::Feature(MediaFeature::Unknown(s.to_owned()))
}

fn parse_media_feature(s: &str) -> MediaFeature {
    let s = s.trim();
    if let Some(idx) = s.find(':') {
        let name = s[..idx].trim();
        let value = s[idx + 1..].trim();
        match name {
            "max-width" => parse_px_pct(value).map(MediaFeature::MaxWidth).unwrap_or(MediaFeature::Unknown(s.to_owned())),
            "min-width" => parse_px_pct(value).map(MediaFeature::MinWidth).unwrap_or(MediaFeature::Unknown(s.to_owned())),
            "max-height" => parse_px_pct(value).map(MediaFeature::MaxHeight).unwrap_or(MediaFeature::Unknown(s.to_owned())),
            "min-height" => parse_px_pct(value).map(MediaFeature::MinHeight).unwrap_or(MediaFeature::Unknown(s.to_owned())),
            "width" => parse_px_pct(value).map(MediaFeature::Width).unwrap_or(MediaFeature::Unknown(s.to_owned())),
            "height" => parse_px_pct(value).map(MediaFeature::Height).unwrap_or(MediaFeature::Unknown(s.to_owned())),
            "orientation" => match value {
                "portrait" => MediaFeature::Orientation(Orientation::Portrait),
                "landscape" => MediaFeature::Orientation(Orientation::Landscape),
                _ => MediaFeature::Unknown(s.to_owned()),
            },
            "prefers-color-scheme" => match value {
                "dark" => MediaFeature::PrefersColorScheme(ColorScheme::Dark),
                "light" => MediaFeature::PrefersColorScheme(ColorScheme::Light),
                _ => MediaFeature::Unknown(s.to_owned()),
            },
            "prefers-reduced-motion" => MediaFeature::PrefersReducedMotion,
            _ => MediaFeature::Unknown(s.to_owned()),
        }
    } else {
        // Boolean feature without value
        match s {
            "hover" => MediaFeature::HoverCapable,
            "prefers-reduced-motion" => MediaFeature::PrefersReducedMotion,
            _ => MediaFeature::Unknown(s.to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// @font-face parser
// ---------------------------------------------------------------------------

/// Parse `@font-face` block declarations into a `FontFace` struct.
pub fn parse_font_face(declarations: &[(String, String)]) -> Option<FontFace> {
    use crate::parser::values::{parse_font_style, parse_font_weight};

    let mut family = None;
    let mut src = Vec::new();
    let mut weight = None;
    let mut style = None;

    for (prop, value) in declarations {
        match prop.as_str() {
            "font-family" => {
                family = Some(value.trim_matches('"').trim_matches('\'').to_owned());
            }
            "src" => {
                for part in value.split(',') {
                    let part = part.trim();
                    if let Some(url) = part.strip_prefix("url(").and_then(|s| s.split(')').next()) {
                        let url = url.trim().trim_matches('"').trim_matches('\'');
                        src.push(FontSource::Url(url.to_owned()));
                    } else if let Some(local) = part.strip_prefix("local(").and_then(|s| s.split(')').next()) {
                        let local = local.trim().trim_matches('"').trim_matches('\'');
                        src.push(FontSource::Local(local.to_owned()));
                    }
                }
            }
            "font-weight" => {
                weight = parse_font_weight(value);
            }
            "font-style" => {
                style = parse_font_style(value);
            }
            _ => {}
        }
    }

    Some(FontFace {
        family: family?,
        src,
        weight,
        style,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_query_max_width() {
        let q = parse_media_query("(max-width: 600px)");
        let ctx_small = MediaContext { width: 400.0, ..Default::default() };
        let ctx_large = MediaContext { width: 800.0, ..Default::default() };
        assert!(q.matches(&ctx_small), "400px should match max-width: 600px");
        assert!(!q.matches(&ctx_large), "800px should NOT match max-width: 600px");
    }

    #[test]
    fn media_query_and() {
        let q = parse_media_query("screen and (max-width: 768px)");
        let ctx = MediaContext { width: 600.0, is_screen: true, ..Default::default() };
        assert!(q.matches(&ctx));
        let print_ctx = MediaContext { width: 600.0, is_screen: false, ..Default::default() };
        assert!(!q.matches(&print_ctx));
    }

    #[test]
    fn media_query_not() {
        let q = parse_media_query("not (max-width: 600px)");
        let ctx_large = MediaContext { width: 800.0, ..Default::default() };
        assert!(q.matches(&ctx_large));
    }

    #[test]
    fn media_query_prefers_dark() {
        let q = parse_media_query("(prefers-color-scheme: dark)");
        let dark_ctx = MediaContext { prefers_dark: true, ..Default::default() };
        let light_ctx = MediaContext { prefers_dark: false, ..Default::default() };
        assert!(q.matches(&dark_ctx));
        assert!(!q.matches(&light_ctx));
    }
}
