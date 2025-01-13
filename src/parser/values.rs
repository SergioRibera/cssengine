use csscolorparser::Color;
use smallvec::SmallVec;
use taffy::{AlignContent, AlignItems, Display, FlexDirection, FlexWrap, JustifyContent, Position};

#[cfg(feature = "tailwind_colors")]
use crate::TAILWIND_COLORS;
use crate::{
    BorderDef, BoxShadow, CursorIcon, Pct, Px, PxPct, PxPctAuto, TextOverflow, TextStyle,
    Transition, Weight, NAMED_COLORS,
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

pub fn parse_px_pct(s: &str) -> Option<PxPct> {
    parse_px(s)
        .map(|px| PxPct::Px(px.0))
        .or_else(|| parse_pct(s).map(|px| PxPct::Pct(px.0)))
}

pub fn parse_pxpctauto(s: &str) -> Option<PxPctAuto> {
    if s == "auto" {
        return Some(PxPctAuto::Auto);
    }
    match parse_px_pct(s) {
        Some(PxPct::Px(px)) => Some(PxPctAuto::Px(px)),
        Some(PxPct::Pct(pct)) => Some(PxPctAuto::Pct(pct)),
        None => None,
    }
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
    let transition = Transition::new(duration);
    Some((key.to_string(), transition))
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
