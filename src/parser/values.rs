use csscolorparser::Color;
use smallvec::SmallVec;
use taffy::{AlignContent, AlignItems, Display, FlexDirection, FlexWrap, JustifyContent, Position};

#[cfg(feature = "tailwind_colors")]
use crate::TAILWIND_COLORS;
use crate::{
    BackgroundImage, BackgroundRepeat, BackgroundSize, BorderDef, BoxShadow, BoxSizing,
    CalcExpr, CursorIcon, EasingFunction, Filter, FilterFunction, GridPlacement, LinearGradient,
    ObjectFit, OverflowAxes, OverflowAxis, Pct, PointerEvents, Px, PxPct, PxPctAuto, StepPosition,
    TextAlign, TextDecoration, TextDecorationLine, TextOverflow, TextStyle, TextTransform,
    TrackList, TrackSize, Transform, TransformFunction, Transition, Visibility, Weight, WhiteSpace,
    WordBreak, NAMED_COLORS,
};

pub const fn parse_display(s: &str) -> Option<Display> {
    match s.as_bytes() {
        b"block" => Some(Display::Block),
        b"flex" => Some(Display::Flex),
        b"grid" => Some(Display::Grid),
        b"none" => Some(Display::None),
        _ => None,
    }
}

pub const fn parse_justify_content(s: &str) -> Option<JustifyContent> {
    match s.as_bytes() {
        b"start" => Some(JustifyContent::Start),
        b"end" => Some(JustifyContent::End),
        b"flex-start" => Some(JustifyContent::FlexStart),
        b"flex-end" => Some(JustifyContent::FlexEnd),
        b"center" => Some(JustifyContent::Center),
        b"stretch" => Some(JustifyContent::Stretch),
        b"space-between" => Some(JustifyContent::SpaceBetween),
        b"space-evenly" => Some(JustifyContent::SpaceEvenly),
        b"space-around" => Some(JustifyContent::SpaceAround),
        _ => None,
    }
}

pub const fn parse_align_items(s: &str) -> Option<AlignItems> {
    match s.as_bytes() {
        b"center" => Some(AlignItems::Center),
        b"start" => Some(AlignItems::Start),
        b"end" => Some(AlignItems::End),
        b"flex-start" => Some(AlignItems::FlexStart),
        b"flex-end" => Some(AlignItems::FlexEnd),
        b"baseline" => Some(AlignItems::Baseline),
        b"stretch" => Some(AlignItems::Stretch),
        _ => None,
    }
}

pub const fn parse_align_content(s: &str) -> Option<AlignContent> {
    match s.as_bytes() {
        b"center" => Some(AlignContent::Center),
        b"start" => Some(AlignContent::Start),
        b"end" => Some(AlignContent::End),
        b"flex-start" => Some(AlignContent::FlexStart),
        b"flex-end" => Some(AlignContent::FlexEnd),
        b"stretch" => Some(AlignContent::Stretch),
        b"space-between" => Some(AlignContent::SpaceBetween),
        b"space-evenly" => Some(AlignContent::SpaceEvenly),
        b"space-around" => Some(AlignContent::SpaceAround),
        _ => None,
    }
}

pub const fn parse_position(s: &str) -> Option<Position> {
    match s.as_bytes() {
        b"absolute" => Some(Position::Absolute),
        b"relative" => Some(Position::Relative),
        _ => None,
    }
}

pub const fn parse_flex_direction(s: &str) -> Option<FlexDirection> {
    match s.as_bytes() {
        b"row" => Some(FlexDirection::Row),
        b"column" => Some(FlexDirection::Column),
        b"row-reverse" => Some(FlexDirection::RowReverse),
        b"column-reverse" => Some(FlexDirection::ColumnReverse),
        _ => None,
    }
}

pub const fn parse_flex_wrap(s: &str) -> Option<FlexWrap> {
    match s.as_bytes() {
        b"wrap" => Some(FlexWrap::Wrap),
        b"no-wrap" => Some(FlexWrap::NoWrap),
        b"wrap-reverse" => Some(FlexWrap::WrapReverse),
        _ => None,
    }
}

pub fn parse_f32(s: &str) -> Option<f32> {
    s.parse::<f32>().ok()
}

pub fn parse_color(s: &str) -> Option<Color> {
    csscolorparser::parse(s)
        .ok()
        .or_else(|| NAMED_COLORS.get(&s).cloned())
        .or_else(|| {
            #[cfg(feature = "tailwind_colors")]
            return TAILWIND_COLORS.get(&s).cloned();
            #[cfg(not(feature = "tailwind_colors"))]
            None
        })
}

pub fn parse_px(s: &str) -> Option<Px> {
    let pixels = s.strip_suffix("px")?;
    match pixels.trim_end().parse::<f32>() {
        Ok(value) => value.try_into().ok(),
        Err(_) => None,
    }
}

pub fn parse_pct(s: &str) -> Option<Pct> {
    let percents = s.strip_suffix('%')?;
    match percents.trim_end().parse::<f32>() {
        Ok(value) => value.try_into().ok(),
        Err(_) => None,
    }
}

/// Parse any CSS length/percentage value including modern units.
/// Note: `rem` must be checked before `em` to avoid false matches.
pub fn parse_px_pct(s: &str) -> Option<PxPct> {
    let s = s.trim();
    if s == "0" {
        return Some(PxPct::Zero);
    }
    if let Some(v) = s.strip_suffix("px") {
        return v.trim_end().parse::<f32>().ok().map(PxPct::Px);
    }
    if let Some(v) = s.strip_suffix('%') {
        return v.trim_end().parse::<f32>().ok().map(PxPct::Pct);
    }
    // rem before em (avoid matching trailing "em" in "rem")
    if let Some(v) = s.strip_suffix("rem") {
        return v.trim_end().parse::<f32>().ok().map(PxPct::Rem);
    }
    if let Some(v) = s.strip_suffix("em") {
        return v.trim_end().parse::<f32>().ok().map(PxPct::Em);
    }
    if let Some(v) = s.strip_suffix("ch") {
        return v.trim_end().parse::<f32>().ok().map(PxPct::Ch);
    }
    if let Some(v) = s.strip_suffix("ex") {
        return v.trim_end().parse::<f32>().ok().map(PxPct::Ex);
    }
    // viewport: vmin/vmax before vw/vh
    if let Some(v) = s.strip_suffix("vmin") {
        return v.trim_end().parse::<f32>().ok().map(PxPct::Vmin);
    }
    if let Some(v) = s.strip_suffix("vmax") {
        return v.trim_end().parse::<f32>().ok().map(PxPct::Vmax);
    }
    if let Some(v) = s.strip_suffix("svh") {
        return v.trim_end().parse::<f32>().ok().map(PxPct::Svh);
    }
    if let Some(v) = s.strip_suffix("dvh") {
        return v.trim_end().parse::<f32>().ok().map(PxPct::Dvh);
    }
    if let Some(v) = s.strip_suffix("vw") {
        return v.trim_end().parse::<f32>().ok().map(PxPct::Vw);
    }
    if let Some(v) = s.strip_suffix("vh") {
        return v.trim_end().parse::<f32>().ok().map(PxPct::Vh);
    }
    if s.starts_with("calc(") {
        return parse_calc(s).map(|e| PxPct::Calc(Box::new(e)));
    }
    if s.starts_with("min(") || s.starts_with("max(") {
        return parse_min_max_fn(s);
    }
    if s.starts_with("clamp(") {
        return parse_clamp_fn(s);
    }
    None
}

/// Parse `calc(expr)` — simple recursive descent supporting +, -, *, /
pub fn parse_calc(s: &str) -> Option<CalcExpr> {
    let inner = s.strip_prefix("calc(")?.strip_suffix(')')?;
    parse_calc_expr(inner.trim())
}

fn parse_calc_expr(s: &str) -> Option<CalcExpr> {
    // Try to split on + or - (lowest precedence, left-to-right)
    // We scan from right to avoid splitting inside nested parens
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    for i in (0..bytes.len()).rev() {
        match bytes[i] {
            b')' => depth += 1,
            b'(' => depth -= 1,
            b'+' if depth == 0 && i > 0 => {
                let lhs = parse_calc_expr(s[..i].trim())?;
                let rhs = parse_calc_expr(s[i + 1..].trim())?;
                return Some(CalcExpr::Add(Box::new(lhs), Box::new(rhs)));
            }
            b'-' if depth == 0 && i > 0 && bytes[i - 1] == b' ' => {
                let lhs = parse_calc_expr(s[..i].trim())?;
                let rhs = parse_calc_expr(s[i + 1..].trim())?;
                return Some(CalcExpr::Sub(Box::new(lhs), Box::new(rhs)));
            }
            _ => {}
        }
    }
    // Try to split on * or /
    for i in (0..bytes.len()).rev() {
        match bytes[i] {
            b')' => depth += 1,
            b'(' => depth -= 1,
            b'*' if depth == 0 => {
                let lhs = parse_calc_expr(s[..i].trim())?;
                let rhs: f32 = s[i + 1..].trim().parse().ok()?;
                return Some(CalcExpr::Mul(Box::new(lhs), rhs));
            }
            b'/' if depth == 0 => {
                let lhs = parse_calc_expr(s[..i].trim())?;
                let rhs: f32 = s[i + 1..].trim().parse().ok()?;
                return Some(CalcExpr::Div(Box::new(lhs), rhs));
            }
            _ => {}
        }
    }
    // Base case: a length value
    parse_px_pct(s).map(|v| CalcExpr::Value(Box::new(v)))
}

fn parse_min_max_fn(s: &str) -> Option<PxPct> {
    // Unused parameter but kept for clarity — we detect min vs max by prefix
    let (prefix, is_min) = if s.starts_with("min(") {
        ("min(", true)
    } else {
        ("max(", false)
    };
    let inner = s.strip_prefix(prefix)?.strip_suffix(')')?;
    let args: Option<Vec<PxPct>> = inner.split(',').map(|a| parse_px_pct(a.trim())).collect();
    let args = args?;
    if is_min {
        Some(PxPct::Min(args))
    } else {
        Some(PxPct::Max(args))
    }
}

fn parse_clamp_fn(s: &str) -> Option<PxPct> {
    let inner = s.strip_prefix("clamp(")?.strip_suffix(')')?;
    let mut parts = inner.splitn(3, ',');
    let min = parse_px_pct(parts.next()?.trim())?;
    let val = parse_px_pct(parts.next()?.trim())?;
    let max = parse_px_pct(parts.next()?.trim())?;
    Some(PxPct::Clamp(Box::new(min), Box::new(val), Box::new(max)))
}

pub fn parse_pxpctauto(s: &str) -> Option<PxPctAuto> {
    if s == "auto" {
        return Some(PxPctAuto::Auto);
    }
    parse_px_pct(s).map(PxPctAuto::from)
}

pub fn parse_i32(s: &str) -> Option<i32> {
    s.parse::<i32>().ok()
}

pub const fn parse_cursor_style(s: &str) -> Option<CursorIcon> {
    match s.as_bytes() {
        b"default" => Some(CursorIcon::Default),
        b"pointer" => Some(CursorIcon::PointingHand),
        b"text" => Some(CursorIcon::Text),
        b"col-resize" => Some(CursorIcon::ResizeColumn),
        b"row-resize" => Some(CursorIcon::ResizeRow),
        b"w-resize" => Some(CursorIcon::ResizeWest),
        b"e-resize" => Some(CursorIcon::ResizeEast),
        b"s-resize" => Some(CursorIcon::ResizeSouth),
        b"n-resize" => Some(CursorIcon::ResizeNorth),
        b"nw-resize" => Some(CursorIcon::ResizeNorthWest),
        b"ne-resize" => Some(CursorIcon::ResizeNorthEast),
        b"sw-resize" => Some(CursorIcon::ResizeSouthWest),
        b"se-resize" => Some(CursorIcon::ResizeSouthEast),
        b"nesw-resize" => Some(CursorIcon::ResizeNeSw),
        b"nwse-resize" => Some(CursorIcon::ResizeNwSe),
        _ => None,
    }
}

pub fn to_owned(s: &str) -> Option<String> {
    Some(s.to_owned())
}

pub const fn parse_font_weight(s: &str) -> Option<Weight> {
    match s.as_bytes() {
        b"100" | b"thin" => Some(Weight(100)),
        b"200" => Some(Weight(200)),
        b"300" => Some(Weight(300)),
        b"400" | b"normal" => Some(Weight(400)),
        b"500" => Some(Weight(500)),
        b"600" => Some(Weight(600)),
        b"700" | b"bold" => Some(Weight(700)),
        b"800" => Some(Weight(800)),
        b"900" => Some(Weight(900)),
        _ => None,
    }
}

pub const fn parse_font_style(s: &str) -> Option<TextStyle> {
    match s.as_bytes() {
        b"normal" => Some(TextStyle::Normal),
        b"italic" => Some(TextStyle::Italic),
        b"oblique" => Some(TextStyle::Oblique),
        _ => None,
    }
}

pub const fn parse_text_overflow(s: &str) -> Option<TextOverflow> {
    match s.as_bytes() {
        b"clip" => Some(TextOverflow::Clip),
        b"ellipsis" => Some(TextOverflow::Ellipsis),
        b"wrap" => Some(TextOverflow::Wrap),
        _ => None,
    }
}

pub fn parse_gap(s: &str) -> Option<(PxPct, Option<PxPct>)> {
    let mut st = s.split_whitespace();
    let row_val = st.next()?;
    let row_px_pct = parse_px_pct(row_val)?;
    let col_val = st.next()?;
    let col_px_pct = parse_px_pct(col_val);
    Some((row_px_pct, col_px_pct))
}

#[allow(clippy::many_single_char_names)]
pub fn parse_box_shadow(s: &str) -> Option<BoxShadow> {
    let mut parts = SmallVec::<[&str; 5]>::new_const();
    let mut start = 0;
    let mut after_wp = false;
    for (i, c) in s.char_indices() {
        if c.is_whitespace() {
            parts.push(&s[start..i]);
            after_wp = true;
            start = i + 1;
        } else if after_wp && c.is_alphabetic() {
            break;
        } else {
            after_wp = false;
        }
    }
    parts.push(&s[start..]);
    match parts.as_slice() {
        ["none"] => None,
        [a, b] => parse_box_shadow_2([a, b]),
        [a, b, c] => parse_box_shadow_3([a, b, c]),
        [a, b, c, d] => parse_box_shadow_4([a, b, c, d]),
        [a, b, c, d, e] => parse_box_shadow_5([a, b, c, d, e]),
        _ => None,
    }
}

fn parse_box_shadow_2([a, b]: [&str; 2]) -> Option<BoxShadow> {
    if let (Some(h_offset), Some(v_offset)) = (parse_px_pct(a), parse_px_pct(b)) {
        return Some(BoxShadow {
            h_offset,
            v_offset,
            ..BoxShadow::default()
        });
    };
    None
}

fn parse_box_shadow_3([a, b, c]: [&str; 3]) -> Option<BoxShadow> {
    // <h_offset> <v_offset> <color>
    if let (Some(h_offset), Some(v_offset), Some(color)) =
        (parse_px_pct(a), parse_px_pct(b), parse_color(c))
    {
        return Some(BoxShadow {
            color,
            h_offset,
            v_offset,
            ..BoxShadow::default()
        });
    }

    // <color> <h_offset> <v_offset>
    if let (Some(color), Some(h_offset), Some(v_offset)) =
        (parse_color(a), parse_px_pct(b), parse_px_pct(c))
    {
        return Some(BoxShadow {
            color,
            h_offset,
            v_offset,
            ..BoxShadow::default()
        });
    }
    // <h_offset> <v_offset> <blur>
    if let (Some(h_offset), Some(v_offset), Some(blur_radius)) =
        (parse_px_pct(a), parse_px_pct(b), parse_px_pct(c))
    {
        return Some(BoxShadow {
            blur_radius,
            h_offset,
            v_offset,
            ..BoxShadow::default()
        });
    }

    None
}
#[allow(clippy::many_single_char_names)]
fn parse_box_shadow_4([a, b, c, d]: [&str; 4]) -> Option<BoxShadow> {
    // <h_offset> <v_offset> <blur_radius> <color>
    if let (Some(h_offset), Some(v_offset), Some(blur_radius), Some(color)) = (
        parse_px_pct(a),
        parse_px_pct(b),
        parse_px_pct(c),
        parse_color(d),
    ) {
        return Some(BoxShadow {
            color,
            blur_radius,
            h_offset,
            v_offset,
            ..BoxShadow::default()
        });
    }
    // <color> <h_offset> <v_offset> <blur_radius>
    if let (Some(color), Some(h_offset), Some(v_offset), Some(blur_radius)) = (
        parse_color(a),
        parse_px_pct(b),
        parse_px_pct(c),
        parse_px_pct(d),
    ) {
        return Some(BoxShadow {
            color,
            blur_radius,
            h_offset,
            v_offset,
            ..BoxShadow::default()
        });
    }
    // <h_offset> <v_offset> <blur_radius> <blur_spread>
    if let (Some(h_offset), Some(v_offset), Some(blur_radius), Some(spread)) = (
        parse_px_pct(a),
        parse_px_pct(b),
        parse_px_pct(c),
        parse_px_pct(d),
    ) {
        return Some(BoxShadow {
            spread,
            blur_radius,
            h_offset,
            v_offset,
            ..BoxShadow::default()
        });
    }
    None
}
#[allow(clippy::many_single_char_names)]
fn parse_box_shadow_5([a, b, c, d, e]: [&str; 5]) -> Option<BoxShadow> {
    // <h_offset> <v_offset> <blur_radius> <blur_spread> <color>
    if let (Some(h_offset), Some(v_offset), Some(blur_radius), Some(spread), Some(color)) = (
        parse_px_pct(a),
        parse_px_pct(b),
        parse_px_pct(c),
        parse_px_pct(d),
        parse_color(e),
    ) {
        return Some(BoxShadow {
            color,
            spread,
            blur_radius,
            h_offset,
            v_offset,
        });
    }
    // <color> <h_offset> <v_offset> <blur_radius> <blur_spread>
    if let (Some(color), Some(h_offset), Some(v_offset), Some(blur_radius), Some(spread)) = (
        parse_color(a),
        parse_px_pct(b),
        parse_px_pct(c),
        parse_px_pct(d),
        parse_px_pct(e),
    ) {
        return Some(BoxShadow {
            color,
            spread,
            blur_radius,
            h_offset,
            v_offset,
        });
    }
    None
}

pub fn parse_transition(s: &str) -> Option<(String, Transition)> {
    let mut parts = s.split_whitespace();
    let key = parts.next()?;
    let duration_str = parts.next()?;
    let duration = parse_duration(duration_str)?;

    let easing = parts.next().and_then(parse_easing_keyword).unwrap_or(EasingFunction::Linear);
    let delay = parts.next().and_then(parse_duration).unwrap_or(0);

    Some((key.to_string(), Transition { duration, delay, easing }))
}

fn parse_easing_keyword(s: &str) -> Option<EasingFunction> {
    match s {
        "linear" => Some(EasingFunction::Linear),
        "ease" => Some(EasingFunction::Ease),
        "ease-in" => Some(EasingFunction::EaseIn),
        "ease-out" => Some(EasingFunction::EaseOut),
        "ease-in-out" => Some(EasingFunction::EaseInOut),
        s if s.starts_with("cubic-bezier(") && s.ends_with(')') => {
            let inner = &s[13..s.len() - 1];
            let nums: Vec<f64> = inner
                .split(',')
                .filter_map(|n| n.trim().parse::<f64>().ok())
                .collect();
            if nums.len() == 4 {
                Some(EasingFunction::CubicBezier(nums[0], nums[1], nums[2], nums[3]))
            } else {
                None
            }
        }
        s if s.starts_with("steps(") && s.ends_with(')') => {
            let inner = &s[6..s.len() - 1];
            let mut ps = inner.splitn(2, ',');
            let n: u32 = ps.next()?.trim().parse().ok()?;
            let pos = match ps.next().map(str::trim) {
                Some("start") => StepPosition::Start,
                Some("both") => StepPosition::Both,
                Some("none") => StepPosition::None,
                _ => StepPosition::End,
            };
            Some(EasingFunction::Steps(n, pos))
        }
        _ => None,
    }
}

fn parse_duration(s: &str) -> Option<u64> {
    if let Some(ms) = s.strip_suffix("ms") {
        return ms.parse::<u64>().ok();
    }
    if let Some(seconds) = s.strip_suffix('s') {
        if let Ok(f) = seconds.parse::<f64>() {
            if f > 0. {
                let ms = (f * 1000.) as u64;
                return Some(ms);
            }
        }
    }
    None
}

pub const fn parse_user_select(s: &str) -> Option<bool> {
    match s.as_bytes() {
        b"none" => Some(false),
        b"auto" => Some(true),
        _ => None,
    }
}

pub fn parse_border(s: &str) -> Option<BorderDef> {
    let mut parts = s.split_whitespace();
    let first = parts.next();
    let second = parts.next();
    let mut retval = BorderDef {
        width: None,
        color: None,
    };
    let mut parse_val = |val: &str| {
        if let Some(px) = parse_px_pct(val) {
            retval.width = Some(px);
            return Some(());
        } else if let Some(color) = parse_color(val) {
            retval.color = Some(color);
            return Some(());
        }
        None
    };
    match (first, second) {
        (Some(val), None) => {
            parse_val(val)?;
        }
        (Some(f), Some(s)) => {
            parse_val(f)?;
            parse_val(s)?;
        }
        _ => return None,
    }
    Some(retval)
}

// ---------------------------------------------------------------------------
// New CSS3 parse functions
// ---------------------------------------------------------------------------

pub fn parse_visibility(s: &str) -> Option<Visibility> {
    match s {
        "visible" => Some(Visibility::Visible),
        "hidden" => Some(Visibility::Hidden),
        "collapse" => Some(Visibility::Collapse),
        _ => None,
    }
}

pub fn parse_pointer_events(s: &str) -> Option<PointerEvents> {
    match s {
        "none" => Some(PointerEvents::None),
        "auto" => Some(PointerEvents::Auto),
        "all" => Some(PointerEvents::All),
        _ => None,
    }
}

pub fn parse_overflow_axis(s: &str) -> Option<OverflowAxis> {
    match s {
        "visible" => Some(OverflowAxis::Visible),
        "hidden" => Some(OverflowAxis::Hidden),
        "scroll" => Some(OverflowAxis::Scroll),
        "auto" => Some(OverflowAxis::Auto),
        "clip" => Some(OverflowAxis::Clip),
        _ => None,
    }
}

pub fn parse_overflow_shorthand(s: &str) -> Option<OverflowAxes> {
    let mut parts = s.split_whitespace();
    let first = parse_overflow_axis(parts.next()?)?;
    let second = parts.next().and_then(parse_overflow_axis).unwrap_or(first);
    Some(OverflowAxes { x: first, y: second })
}

pub fn parse_text_align(s: &str) -> Option<TextAlign> {
    match s {
        "left" => Some(TextAlign::Left),
        "right" => Some(TextAlign::Right),
        "center" => Some(TextAlign::Center),
        "justify" => Some(TextAlign::Justify),
        "start" => Some(TextAlign::Start),
        "end" => Some(TextAlign::End),
        _ => None,
    }
}

pub fn parse_text_decoration(s: &str) -> Option<TextDecoration> {
    if s == "none" {
        return Some(TextDecoration { line: TextDecorationLine::None, color: None });
    }
    let mut parts = s.split_whitespace();
    let mut line = TextDecorationLine::None;
    let mut color = None;
    for part in parts.by_ref() {
        if let Some(l) = match part {
            "underline" => Some(TextDecorationLine::Underline),
            "overline" => Some(TextDecorationLine::Overline),
            "line-through" => Some(TextDecorationLine::LineThrough),
            _ => None,
        } {
            line = l;
        } else if let Some(c) = parse_color(part) {
            color = Some(c);
        }
    }
    Some(TextDecoration { line, color })
}

pub fn parse_text_transform(s: &str) -> Option<TextTransform> {
    match s {
        "none" => Some(TextTransform::None),
        "uppercase" => Some(TextTransform::Uppercase),
        "lowercase" => Some(TextTransform::Lowercase),
        "capitalize" => Some(TextTransform::Capitalize),
        _ => None,
    }
}

pub fn parse_white_space(s: &str) -> Option<WhiteSpace> {
    match s {
        "normal" => Some(WhiteSpace::Normal),
        "nowrap" | "no-wrap" => Some(WhiteSpace::Nowrap),
        "pre" => Some(WhiteSpace::Pre),
        "pre-wrap" => Some(WhiteSpace::PreWrap),
        "pre-line" => Some(WhiteSpace::PreLine),
        "break-spaces" => Some(WhiteSpace::BreakSpaces),
        _ => None,
    }
}

pub fn parse_word_break(s: &str) -> Option<WordBreak> {
    match s {
        "normal" => Some(WordBreak::Normal),
        "break-all" => Some(WordBreak::BreakAll),
        "keep-all" => Some(WordBreak::KeepAll),
        "break-word" => Some(WordBreak::BreakWord),
        _ => None,
    }
}

pub fn parse_box_sizing(s: &str) -> Option<BoxSizing> {
    match s {
        "content-box" => Some(BoxSizing::ContentBox),
        "border-box" => Some(BoxSizing::BorderBox),
        _ => None,
    }
}

pub fn parse_object_fit(s: &str) -> Option<ObjectFit> {
    match s {
        "fill" => Some(ObjectFit::Fill),
        "contain" => Some(ObjectFit::Contain),
        "cover" => Some(ObjectFit::Cover),
        "none" => Some(ObjectFit::None),
        "scale-down" => Some(ObjectFit::ScaleDown),
        _ => None,
    }
}

/// Parse a CSS angle like `45deg`, `0.5turn`, `1rad`
fn parse_angle_deg(s: &str) -> Option<f32> {
    if let Some(v) = s.strip_suffix("deg") {
        return v.trim().parse::<f32>().ok();
    }
    if let Some(v) = s.strip_suffix("turn") {
        return v.trim().parse::<f32>().ok().map(|t| t * 360.0);
    }
    if let Some(v) = s.strip_suffix("rad") {
        return v.trim().parse::<f32>().ok().map(|r| r.to_degrees());
    }
    // Plain number treated as degrees (e.g. in matrix)
    s.parse::<f32>().ok()
}

/// Parse a single CSS function call like `translateX(10px)`.
fn parse_transform_function(s: &str) -> Option<TransformFunction> {
    let s = s.trim();
    let (name, rest) = s.split_once('(')?;
    let args = rest.strip_suffix(')')?;
    let name = name.trim();
    let argv: Vec<&str> = args.split(',').map(str::trim).collect();

    match name {
        "translate" => {
            let x = parse_px_pct(argv.first()?)?;
            let y = argv.get(1).and_then(|v| parse_px_pct(v)).unwrap_or(PxPct::Px(0.0));
            Some(TransformFunction::Translate(x, y))
        }
        "translateX" | "translatex" => Some(TransformFunction::TranslateX(parse_px_pct(argv.first()?)?)),
        "translateY" | "translatey" => Some(TransformFunction::TranslateY(parse_px_pct(argv.first()?)?)),
        "translateZ" | "translatez" => Some(TransformFunction::TranslateZ(parse_px_pct(argv.first()?)?)),
        "scale" => {
            let sx = argv.first()?.parse::<f32>().ok()?;
            let sy = argv.get(1).and_then(|v| v.parse::<f32>().ok()).unwrap_or(sx);
            Some(TransformFunction::Scale(sx, sy))
        }
        "scaleX" | "scalex" => Some(TransformFunction::ScaleX(argv.first()?.parse::<f32>().ok()?)),
        "scaleY" | "scaley" => Some(TransformFunction::ScaleY(argv.first()?.parse::<f32>().ok()?)),
        "rotate" => Some(TransformFunction::Rotate(parse_angle_deg(argv.first()?)?)),
        "rotateX" | "rotatex" => Some(TransformFunction::RotateX(parse_angle_deg(argv.first()?)?)),
        "rotateY" | "rotatey" => Some(TransformFunction::RotateY(parse_angle_deg(argv.first()?)?)),
        "rotateZ" | "rotatez" => Some(TransformFunction::RotateZ(parse_angle_deg(argv.first()?)?)),
        "skewX" | "skewx" => Some(TransformFunction::SkewX(parse_angle_deg(argv.first()?)?)),
        "skewY" | "skewy" => Some(TransformFunction::SkewY(parse_angle_deg(argv.first()?)?)),
        "perspective" => Some(TransformFunction::Perspective(parse_px(argv.first()?)?)),
        "matrix" if argv.len() == 6 => {
            let vals: Option<Vec<f32>> = argv.iter().map(|v| v.parse::<f32>().ok()).collect();
            let v = vals?;
            Some(TransformFunction::Matrix(v[0], v[1], v[2], v[3], v[4], v[5]))
        }
        _ => None,
    }
}

pub fn parse_transform(s: &str) -> Option<Transform> {
    if s == "none" {
        return Some(Transform::default());
    }
    // Split on ')' boundaries to get individual function calls
    let mut funcs = Vec::new();
    let mut rest = s.trim();
    while !rest.is_empty() {
        let end = rest.find(')')? + 1;
        let func_str = rest[..end].trim();
        if let Some(f) = parse_transform_function(func_str) {
            funcs.push(f);
        }
        rest = rest[end..].trim_start();
    }
    if funcs.is_empty() { None } else { Some(Transform(funcs)) }
}

/// Parse a single filter function like `blur(4px)`.
fn parse_filter_function(s: &str) -> Option<FilterFunction> {
    let s = s.trim();
    let (name, rest) = s.split_once('(')?;
    let arg = rest.strip_suffix(')')?.trim();
    match name.trim() {
        "blur" => Some(FilterFunction::Blur(parse_px(arg)?)),
        "brightness" => Some(FilterFunction::Brightness(parse_filter_value(arg)?)),
        "contrast" => Some(FilterFunction::Contrast(parse_filter_value(arg)?)),
        "grayscale" => Some(FilterFunction::Grayscale(parse_filter_value(arg)?)),
        "hue-rotate" => Some(FilterFunction::HueRotate(parse_angle_deg(arg)?)),
        "invert" => Some(FilterFunction::Invert(parse_filter_value(arg)?)),
        "opacity" => Some(FilterFunction::Opacity(parse_filter_value(arg)?)),
        "saturate" => Some(FilterFunction::Saturate(parse_filter_value(arg)?)),
        "sepia" => Some(FilterFunction::Sepia(parse_filter_value(arg)?)),
        _ => None,
    }
}

fn parse_filter_value(s: &str) -> Option<f32> {
    // Accept percentage (50%) or decimal (0.5)
    if let Some(v) = s.strip_suffix('%') {
        v.parse::<f32>().ok().map(|v| v / 100.0)
    } else {
        s.parse::<f32>().ok()
    }
}

pub fn parse_filter(s: &str) -> Option<Filter> {
    if s == "none" {
        return Some(Filter::default());
    }
    let mut funcs = Vec::new();
    let mut rest = s.trim();
    while !rest.is_empty() {
        let end = rest.find(')')? + 1;
        let func_str = rest[..end].trim();
        if let Some(f) = parse_filter_function(func_str) {
            funcs.push(f);
        }
        rest = rest[end..].trim_start();
    }
    if funcs.is_empty() { None } else { Some(Filter(funcs)) }
}

pub fn parse_background_image(s: &str) -> Option<BackgroundImage> {
    let s = s.trim();
    if s == "none" {
        return Some(BackgroundImage::None);
    }
    if let Some(url) = s.strip_prefix("url(").and_then(|s| s.strip_suffix(')')) {
        let url = url.trim().trim_matches('"').trim_matches('\'');
        return Some(BackgroundImage::Url(url.to_owned()));
    }
    if s.starts_with("linear-gradient(") {
        return parse_linear_gradient(s).map(BackgroundImage::LinearGradient);
    }
    None
}

fn parse_linear_gradient(s: &str) -> Option<LinearGradient> {
    let inner = s.strip_prefix("linear-gradient(")?.strip_suffix(')')?;
    let mut parts = inner.splitn(2, ',');
    let first = parts.next()?.trim();
    let rest = parts.next().unwrap_or("").trim();

    let angle = if first.ends_with("deg") || first.ends_with("turn") || first.ends_with("rad") {
        parse_angle_deg(first).unwrap_or(180.0)
    } else if first.starts_with("to ") {
        match first {
            "to bottom" => 180.0,
            "to top" => 0.0,
            "to right" => 90.0,
            "to left" => 270.0,
            _ => 180.0,
        }
    } else {
        180.0
    };

    let mut stops = Vec::new();
    for stop_str in rest.split(',') {
        let stop_str = stop_str.trim();
        let mut tok = stop_str.splitn(2, ' ');
        let color_str = tok.next()?;
        let color = parse_color(color_str)?;
        let pos = tok.next().and_then(parse_px_pct);
        stops.push((color, pos));
    }

    Some(LinearGradient { angle, stops })
}

pub fn parse_background_size(s: &str) -> Option<BackgroundSize> {
    match s {
        "cover" => Some(BackgroundSize::Cover),
        "contain" => Some(BackgroundSize::Contain),
        "auto" => Some(BackgroundSize::Auto),
        _ => {
            let mut parts = s.split_whitespace();
            let w = parse_px_pct(parts.next()?)?;
            let h = parts.next().and_then(parse_px_pct).unwrap_or_else(|| w.clone());
            Some(BackgroundSize::Length(w, h))
        }
    }
}

pub fn parse_background_repeat(s: &str) -> Option<BackgroundRepeat> {
    match s {
        "repeat" => Some(BackgroundRepeat::Repeat),
        "repeat-x" => Some(BackgroundRepeat::RepeatX),
        "repeat-y" => Some(BackgroundRepeat::RepeatY),
        "no-repeat" => Some(BackgroundRepeat::NoRepeat),
        "space" => Some(BackgroundRepeat::Space),
        "round" => Some(BackgroundRepeat::Round),
        _ => None,
    }
}

pub fn parse_background_position(s: &str) -> Option<(PxPct, PxPct)> {
    let named_pos = |v: &str| match v {
        "left" => Some(PxPct::Pct(0.0)),
        "right" => Some(PxPct::Pct(100.0)),
        "top" => Some(PxPct::Pct(0.0)),
        "bottom" => Some(PxPct::Pct(100.0)),
        "center" => Some(PxPct::Pct(50.0)),
        _ => parse_px_pct(v),
    };
    let mut parts = s.split_whitespace();
    let x = named_pos(parts.next()?)?;
    let y = parts.next().and_then(named_pos).unwrap_or_else(|| x.clone());
    Some((x, y))
}

// ---------------------------------------------------------------------------
// Grid parse functions
// ---------------------------------------------------------------------------

pub fn parse_track_size(s: &str) -> Option<TrackSize> {
    let s = s.trim();
    if s == "auto" { return Some(TrackSize::Auto); }
    if s == "min-content" { return Some(TrackSize::MinContent); }
    if s == "max-content" { return Some(TrackSize::MaxContent); }
    if let Some(fr) = s.strip_suffix("fr") {
        return fr.trim().parse::<f32>().ok().map(TrackSize::Fr);
    }
    if let Some(pct) = s.strip_suffix('%') {
        return pct.trim().parse::<f32>().ok().map(TrackSize::Pct);
    }
    if let Some(px) = s.strip_suffix("px") {
        return px.trim().parse::<f32>().ok().map(TrackSize::Px);
    }
    if s.starts_with("minmax(") {
        let inner = s.strip_prefix("minmax(")?.strip_suffix(')')?;
        let mut it = inner.splitn(2, ',');
        let min = parse_track_size(it.next()?)?;
        let max = parse_track_size(it.next()?)?;
        return Some(TrackSize::MinMax(Box::new(min), Box::new(max)));
    }
    if s.starts_with("fit-content(") {
        let inner = s.strip_prefix("fit-content(")?.strip_suffix(')')?;
        let v = parse_px_pct(inner)?;
        return Some(TrackSize::FitContent(v));
    }
    None
}

pub fn parse_track_list(s: &str) -> Option<TrackList> {
    if s == "none" { return Some(TrackList::default()); }
    let tracks: Option<Vec<TrackSize>> = s.split_whitespace().map(parse_track_size).collect();
    tracks.map(TrackList)
}

pub fn parse_grid_line(s: &str) -> Option<GridPlacement> {
    let s = s.trim();
    if s == "auto" { return Some(GridPlacement::Auto); }
    if let Some(span_str) = s.strip_prefix("span ") {
        return span_str.trim().parse::<u32>().ok().map(GridPlacement::Span);
    }
    if s.contains('/') {
        let mut parts = s.splitn(2, '/');
        let line: i32 = parts.next()?.trim().parse().ok()?;
        let span_str = parts.next()?.trim();
        if let Some(s) = span_str.strip_prefix("span ") {
            let span: u32 = s.trim().parse().ok()?;
            return Some(GridPlacement::LineSpan(line, span));
        }
    }
    s.parse::<i32>().ok().map(GridPlacement::Line)
}

#[cfg(test)]
mod tests {
    use csscolorparser::Color;

    use crate::PxPct;

    use super::{parse_border, parse_box_shadow_5, parse_duration, BorderDef};

    #[test]
    fn duration() {
        let sec = parse_duration("1s").unwrap();
        assert!(sec == 1000);
        let tenth_sec = parse_duration("0.1s").unwrap();
        assert!(tenth_sec == 100);
        let ms = parse_duration("150ms").unwrap();
        assert!(ms == 150);
        // This should fail
        let value = parse_duration("1");
        assert!(value.is_none());
    }

    #[test]
    #[rustfmt::skip]
    fn border() {
        let v = parse_border("10px").unwrap();
        assert!(v == BorderDef { width: Some(PxPct::Px(10.0)), color: None });
        let v = parse_border("10px red").unwrap();
        assert!(v == BorderDef { width: Some(PxPct::Px(10.0)), color: Some(Color::new(1.0,0.0,0.0,1.0)) });
        let v = parse_border("red").unwrap();
        assert!(v == BorderDef { width: None, color: Some(Color::new(1.0, 0.0, 0.0, 1.0)) });
    }

    #[test]
    fn box_shadow_5() {
        let v = parse_box_shadow_5(["4px", "8px", "10px", "15px", "black"]).unwrap();
        assert!(v.h_offset == PxPct::Px(4.0));
        assert!(v.v_offset == PxPct::Px(8.0));
        assert!(v.blur_radius == PxPct::Px(10.0));
        assert!(v.spread == PxPct::Px(15.0));
        assert!(v.color == Color::new(0.0, 0.0, 0.0, 1.0));
        let v = parse_box_shadow_5(["green", "4px", "8px", "10px", "15px"]).unwrap();
        println!("{v:?}");
        assert!(v.h_offset == PxPct::Px(4.0));
        assert!(v.v_offset == PxPct::Px(8.0));
        assert!(v.blur_radius == PxPct::Px(10.0));
        assert!(v.spread == PxPct::Px(15.0));
        assert!(v.color == Color::from_rgba8(0, 128, 0, 255));
    }
}
