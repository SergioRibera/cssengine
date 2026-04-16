#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub trait Easing {
    fn eval(&self, time: f64) -> f64;
    fn velocity(&self, time: f64) -> Option<f64> {
        let _ = time;
        None
    }
    fn finished(&self, time: f64) -> bool {
        !(0.0..1.0).contains(&time)
    }
}

// ---------------------------------------------------------------------------
// StepPosition
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum StepPosition {
    Start,
    #[default]
    End,
    Both,
    None,
}

// ---------------------------------------------------------------------------
// EasingFunction
// ---------------------------------------------------------------------------

/// CSS easing function — corresponds to the `transition-timing-function` property.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum EasingFunction {
    #[default]
    Linear,
    /// `cubic-bezier(0.25, 0.1, 0.25, 1.0)`
    Ease,
    /// `cubic-bezier(0.42, 0, 1.0, 1.0)`
    EaseIn,
    /// `cubic-bezier(0, 0, 0.58, 1.0)`
    EaseOut,
    /// `cubic-bezier(0.42, 0, 0.58, 1.0)`
    EaseInOut,
    CubicBezier(f64, f64, f64, f64),
    Steps(u32, StepPosition),
}

impl Easing for EasingFunction {
    fn eval(&self, t: f64) -> f64 {
        match self {
            Self::Linear => t,
            Self::Ease => eval_cubic(t, 0.25, 0.1, 0.25, 1.0),
            Self::EaseIn => eval_cubic(t, 0.42, 0.0, 1.0, 1.0),
            Self::EaseOut => eval_cubic(t, 0.0, 0.0, 0.58, 1.0),
            Self::EaseInOut => eval_cubic(t, 0.42, 0.0, 0.58, 1.0),
            Self::CubicBezier(x1, y1, x2, y2) => eval_cubic(t, *x1, *y1, *x2, *y2),
            Self::Steps(n, pos) => eval_steps(t, *n, *pos),
        }
    }
}

// Cubic-bezier implementation: solve Bx(t)=x via Newton's method, then return By(t).
fn bezier_component(t: f64, p1: f64, p2: f64) -> f64 {
    3.0 * p1 * t * (1.0 - t).powi(2)
        + 3.0 * p2 * t.powi(2) * (1.0 - t)
        + t.powi(3)
}

fn bezier_slope(t: f64, p1: f64, p2: f64) -> f64 {
    3.0 * p1 * (1.0 - 4.0 * t + 3.0 * t.powi(2))
        + 3.0 * p2 * (2.0 * t - 3.0 * t.powi(2))
        + 3.0 * t.powi(2)
}

fn eval_cubic(x: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let mut t = x;
    for _ in 0..8 {
        let slope = bezier_slope(t, x1, x2);
        if slope.abs() < 1e-12 {
            break;
        }
        t -= (bezier_component(t, x1, x2) - x) / slope;
        t = t.clamp(0.0, 1.0);
    }
    bezier_component(t, y1, y2)
}

fn eval_steps(x: f64, n: u32, pos: StepPosition) -> f64 {
    let n = n as f64;
    (match pos {
        StepPosition::Start => (x * n).ceil() / n,
        StepPosition::End => (x * n).floor() / n,
        StepPosition::Both => ((x * n).floor() + 0.5) / n,
        StepPosition::None => {
            if x <= 0.0 {
                0.0
            } else if x >= 1.0 {
                1.0
            } else {
                (x * n).round() / n
            }
        }
    })
    .clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Backward-compatible Linear struct
// ---------------------------------------------------------------------------

/// Legacy linear easing struct — prefer `EasingFunction::Linear` for new code.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Linear;

impl Easing for Linear {
    fn eval(&self, time: f64) -> f64 {
        time
    }
}

// ---------------------------------------------------------------------------
// Transition
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Transition {
    /// Duration in milliseconds.
    pub duration: u64,
    /// Delay before the transition starts, in milliseconds.
    pub delay: u64,
    pub easing: EasingFunction,
}

impl Transition {
    pub fn new(duration: u64) -> Self {
        Self { duration, delay: 0, easing: EasingFunction::Linear }
    }

    pub fn with_easing(duration: u64, easing: EasingFunction) -> Self {
        Self { duration, delay: 0, easing }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_easing() {
        assert!((EasingFunction::Linear.eval(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn cubic_bezier_bounds() {
        let ease = EasingFunction::Ease;
        assert!((ease.eval(0.0)).abs() < 1e-6);
        assert!((ease.eval(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn steps_end() {
        let s = EasingFunction::Steps(4, StepPosition::End);
        assert!((s.eval(0.0)).abs() < 1e-6);
        assert!((s.eval(0.3) - 0.25).abs() < 1e-6);
        assert!((s.eval(0.51) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn transition_new() {
        let t = Transition::new(300);
        assert_eq!(t.duration, 300);
        assert_eq!(t.delay, 0);
        assert_eq!(t.easing, EasingFunction::Linear);
    }
}
