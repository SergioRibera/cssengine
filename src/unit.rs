mod color;
mod cursor;
mod px;
mod text;

pub(crate) use color::NAMED_COLORS;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "tailwind_colors")]
pub(crate) use color::{TAILWIND_COLORS, TAILWIND_NAME_COLORS};

pub use csscolorparser::Color;
pub use cursor::*;
pub use px::{CalcExpr, Pct, Px, PxPct, PxPctAuto};
pub use text::{Style as TextStyle, TextOverflow, Weight};

/// Used for automatically computed values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Auto;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct BorderDef {
    pub width: Option<PxPct>,
    pub color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct BoxShadow {
    pub blur_radius: PxPct,
    pub color: Color,
    pub spread: PxPct,
    pub h_offset: PxPct,
    pub v_offset: PxPct,
}

impl Default for BoxShadow {
    fn default() -> Self {
        Self {
            blur_radius: PxPct::Px(0.),
            color: Color::from_rgba8(0, 0, 0, 255),
            spread: PxPct::Px(0.),
            h_offset: PxPct::Px(0.),
            v_offset: PxPct::Px(0.),
        }
    }
}

// ---------------------------------------------------------------------------
// New CSS3 unit types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Visibility {
    Visible,
    Hidden,
    Collapse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum PointerEvents {
    None,
    Auto,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum OverflowAxis {
    Visible,
    Hidden,
    Scroll,
    Auto,
    Clip,
}

/// Both axes for the `overflow` shorthand
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct OverflowAxes {
    pub x: OverflowAxis,
    pub y: OverflowAxis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum TextDecorationLine {
    None,
    Underline,
    Overline,
    LineThrough,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TextDecoration {
    pub line: TextDecorationLine,
    pub color: Option<Color>,
}

impl Default for TextDecoration {
    fn default() -> Self {
        Self { line: TextDecorationLine::None, color: None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum TextTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum WhiteSpace {
    Normal,
    Nowrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum WordBreak {
    Normal,
    BreakAll,
    KeepAll,
    BreakWord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum BoxSizing {
    ContentBox,
    BorderBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum ObjectFit {
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown,
}

// ---------------------------------------------------------------------------
// Transform
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum TransformFunction {
    Translate(PxPct, PxPct),
    TranslateX(PxPct),
    TranslateY(PxPct),
    TranslateZ(PxPct),
    Scale(f32, f32),
    ScaleX(f32),
    ScaleY(f32),
    Rotate(f32),   // degrees
    RotateX(f32),
    RotateY(f32),
    RotateZ(f32),
    SkewX(f32),
    SkewY(f32),
    Matrix(f32, f32, f32, f32, f32, f32),
    Perspective(Px),
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Transform(pub Vec<TransformFunction>);

// ---------------------------------------------------------------------------
// Filter
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum FilterFunction {
    Blur(Px),
    Brightness(f32),
    Contrast(f32),
    Grayscale(f32),
    HueRotate(f32), // degrees
    Invert(f32),
    Opacity(f32),
    Saturate(f32),
    Sepia(f32),
    DropShadow(BoxShadow),
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Filter(pub Vec<FilterFunction>);

// ---------------------------------------------------------------------------
// Background
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum BackgroundImage {
    None,
    Url(String),
    LinearGradient(LinearGradient),
}

impl Default for BackgroundImage {
    fn default() -> Self { Self::None }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct LinearGradient {
    pub angle: f32, // degrees
    pub stops: Vec<(Color, Option<PxPct>)>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum BackgroundSize {
    Cover,
    Contain,
    Auto,
    Length(PxPct, PxPct),
}

impl Default for BackgroundSize {
    fn default() -> Self { Self::Auto }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum BackgroundRepeat {
    Repeat,
    RepeatX,
    RepeatY,
    NoRepeat,
    Space,
    Round,
}

impl Default for BackgroundRepeat {
    fn default() -> Self { Self::Repeat }
}

// ---------------------------------------------------------------------------
// Grid types (always available — CSS grid parsing doesn't require taffy/grid feature)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum TrackSize {
    Px(f32),
    Pct(f32),
    Fr(f32),
    Auto,
    MinMax(Box<TrackSize>, Box<TrackSize>),
    MinContent,
    MaxContent,
    FitContent(PxPct),
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TrackList(pub Vec<TrackSize>);

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum GridPlacement {
    Auto,
    Line(i32),
    Span(u32),
    LineSpan(i32, u32),
}
