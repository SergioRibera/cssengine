use taffy::style::{Dimension, LengthPercentage, LengthPercentageAuto};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::Auto;

/// A pixel value
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Px(pub f32);

/// A percent value
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Pct(pub f32);
impl<T> From<T> for Pct
where
    T: Into<f32>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl<T> From<T> for Px
where
    T: Into<f32>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

/// CSS length value supporting all modern units.
///
/// Relative units (`Em`, `Rem`, `Vw`, `Vh`, etc.) cannot be resolved to pixels
/// at parse time — the consumer must resolve them with knowledge of the current
/// font-size and viewport dimensions.
///
/// For taffy integration, call `.to_px(font_size, viewport_width, viewport_height)`
/// or fall back to `PxPct::Px(0.0)` for layout purposes.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum PxPct {
    // Absolute
    Px(f32),
    // Relative to parent
    Pct(f32),
    // Font-relative
    Em(f32),
    Rem(f32),
    Ch(f32),
    Ex(f32),
    // Viewport-relative
    Vw(f32),
    Vh(f32),
    Vmin(f32),
    Vmax(f32),
    Svh(f32),
    Dvh(f32),
    // CSS functions
    Calc(Box<CalcExpr>),
    Min(Vec<PxPct>),
    Max(Vec<PxPct>),
    Clamp(Box<PxPct>, Box<PxPct>, Box<PxPct>),
    /// Explicit zero — always 0px regardless of unit
    Zero,
}

impl PxPct {
    /// Best-effort conversion to pixels for layout engines.
    /// Relative units default to 0 unless resolution context is provided.
    pub fn to_px_lossy(&self) -> f32 {
        match self {
            PxPct::Px(v) | PxPct::Em(v) | PxPct::Rem(v) => *v,
            PxPct::Pct(_)
            | PxPct::Ch(_)
            | PxPct::Ex(_)
            | PxPct::Vw(_)
            | PxPct::Vh(_)
            | PxPct::Vmin(_)
            | PxPct::Vmax(_)
            | PxPct::Svh(_)
            | PxPct::Dvh(_) => 0.0,
            PxPct::Zero => 0.0,
            PxPct::Calc(_) | PxPct::Min(_) | PxPct::Max(_) | PxPct::Clamp(_, _, _) => 0.0,
        }
    }
}

/// Simple expression tree for `calc()`, `min()`, `max()`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum CalcExpr {
    Value(Box<PxPct>),
    Add(Box<CalcExpr>, Box<CalcExpr>),
    Sub(Box<CalcExpr>, Box<CalcExpr>),
    Mul(Box<CalcExpr>, f32),
    Div(Box<CalcExpr>, f32),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum PxPctAuto {
    Px(f32),
    Pct(f32),
    // Font-relative
    Em(f32),
    Rem(f32),
    // Viewport-relative
    Vw(f32),
    Vh(f32),
    Vmin(f32),
    Vmax(f32),
    // CSS functions
    Calc(Box<CalcExpr>),
    Min(Vec<PxPct>),
    Max(Vec<PxPct>),
    Auto,
    Zero,
}

impl PxPctAuto {
    pub fn to_px_lossy(&self) -> f32 {
        match self {
            PxPctAuto::Px(v) | PxPctAuto::Em(v) | PxPctAuto::Rem(v) => *v,
            _ => 0.0,
        }
    }
}

impl From<Pct> for PxPct {
    fn from(value: Pct) -> Self {
        PxPct::Pct(value.0)
    }
}

impl From<f32> for PxPct {
    fn from(value: f32) -> Self {
        PxPct::Px(value)
    }
}

impl From<Px> for PxPct {
    fn from(value: Px) -> Self {
        PxPct::Px(value.0)
    }
}

impl From<Pct> for PxPctAuto {
    fn from(value: Pct) -> Self {
        PxPctAuto::Pct(value.0)
    }
}

impl From<Auto> for PxPctAuto {
    fn from(_: Auto) -> Self {
        PxPctAuto::Auto
    }
}

impl From<Px> for PxPctAuto {
    fn from(value: Px) -> Self {
        PxPctAuto::Px(value.0)
    }
}

impl From<PxPct> for PxPctAuto {
    fn from(value: PxPct) -> Self {
        match value {
            PxPct::Pct(pct) => PxPctAuto::Pct(pct),
            PxPct::Px(px) => PxPctAuto::Px(px),
            PxPct::Em(v) => PxPctAuto::Em(v),
            PxPct::Rem(v) => PxPctAuto::Rem(v),
            PxPct::Vw(v) => PxPctAuto::Vw(v),
            PxPct::Vh(v) => PxPctAuto::Vh(v),
            PxPct::Vmin(v) => PxPctAuto::Vmin(v),
            PxPct::Vmax(v) => PxPctAuto::Vmax(v),
            PxPct::Calc(e) => PxPctAuto::Calc(e),
            PxPct::Min(v) => PxPctAuto::Min(v),
            PxPct::Max(v) => PxPctAuto::Max(v),
            PxPct::Zero => PxPctAuto::Zero,
            // Ch/Ex/Svh/Dvh don't have PxPctAuto equivalents → fall back to 0
            PxPct::Ch(_) | PxPct::Ex(_) | PxPct::Svh(_) | PxPct::Dvh(_) | PxPct::Clamp(_, _, _) => PxPctAuto::Zero,
        }
    }
}

// ---------------------------------------------------------------------------
// Taffy conversion — uses lossy px resolution for relative units
// ---------------------------------------------------------------------------

impl From<PxPctAuto> for Dimension {
    fn from(value: PxPctAuto) -> Self {
        match value {
            PxPctAuto::Px(v) => Dimension::length(v),
            PxPctAuto::Pct(v) => Dimension::percent(v / 100.0),
            PxPctAuto::Auto => Dimension::auto(),
            PxPctAuto::Zero => Dimension::length(0.0),
            // Relative units: store as zero; consumer must resolve before layout
            other => Dimension::length(other.to_px_lossy()),
        }
    }
}

impl From<PxPct> for LengthPercentage {
    fn from(value: PxPct) -> Self {
        match value {
            PxPct::Px(v) => LengthPercentage::length(v),
            PxPct::Pct(v) => LengthPercentage::percent(v / 100.0),
            PxPct::Zero => LengthPercentage::length(0.0),
            other => LengthPercentage::length(other.to_px_lossy()),
        }
    }
}

impl From<PxPctAuto> for LengthPercentageAuto {
    fn from(value: PxPctAuto) -> Self {
        match value {
            PxPctAuto::Px(v) => LengthPercentageAuto::length(v),
            PxPctAuto::Pct(v) => LengthPercentageAuto::percent(v / 100.0),
            PxPctAuto::Auto => LengthPercentageAuto::auto(),
            PxPctAuto::Zero => LengthPercentageAuto::length(0.0),
            other => LengthPercentageAuto::length(other.to_px_lossy()),
        }
    }
}
