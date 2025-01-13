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

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum PxPct {
    Px(f32),
    Pct(f32),
}

impl From<Pct> for PxPct {
    fn from(value: Pct) -> Self {
        PxPct::Pct(value.0)
    }
}

impl<T> From<T> for PxPct
where
    T: Into<Px>,
{
    fn from(value: T) -> Self {
        PxPct::Px(value.into().0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum PxPctAuto {
    Px(f32),
    Pct(f32),
    Auto,
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

impl<T> From<T> for PxPctAuto
where
    T: Into<Px>,
{
    fn from(value: T) -> Self {
        PxPctAuto::Px(value.into().0)
    }
}

impl From<PxPct> for PxPctAuto {
    fn from(value: PxPct) -> Self {
        match value {
            PxPct::Pct(pct) => PxPctAuto::Pct(pct),
            PxPct::Px(px) => PxPctAuto::Px(px),
        }
    }
}

impl From<PxPctAuto> for Dimension {
    fn from(value: PxPctAuto) -> Self {
        match value {
            PxPctAuto::Px(v) => Dimension::Length(v as f32),
            PxPctAuto::Pct(v) => Dimension::Percent(v as f32 / 100.0),
            PxPctAuto::Auto => Dimension::Auto,
        }
    }
}

impl From<PxPct> for LengthPercentage {
    fn from(value: PxPct) -> Self {
        match value {
            PxPct::Px(v) => LengthPercentage::Length(v as f32),
            PxPct::Pct(v) => LengthPercentage::Percent(v as f32 / 100.0),
        }
    }
}

impl From<PxPctAuto> for LengthPercentageAuto {
    fn from(value: PxPctAuto) -> Self {
        match value {
            PxPctAuto::Px(v) => LengthPercentageAuto::Length(v as f32),
            PxPctAuto::Pct(v) => LengthPercentageAuto::Percent(v as f32 / 100.0),
            PxPctAuto::Auto => LengthPercentageAuto::Auto,
        }
    }
}
