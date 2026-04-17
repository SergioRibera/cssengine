use std::borrow::Cow;

use csscolorparser::Color;
use taffy::{AlignContent, AlignItems, Display, FlexDirection, FlexWrap, JustifyContent, Position};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    BackgroundImage, BackgroundRepeat, BackgroundSize, BorderDef, BoxShadow, BoxSizing,
    CursorIcon, Filter, GridPlacement, ObjectFit, OverflowAxes, OverflowAxis, PointerEvents, Px,
    PxPct, PxPctAuto, TextAlign, TextDecoration, TextTransform, TextOverflow, TextStyle,
    TrackList, Transform, Transition, Visibility, Weight, WhiteSpace, WordBreak,
};

macro_rules! gen_declaration {
    ($($prop:expr => $fun:ident $item:ident $t:ty;)*) => {
        #[derive(Clone, Debug, Default)]
        #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
        pub enum Declaration {
            #[default]
            None,
            $($item( $t ),)*
        }

        impl Declaration {
            pub fn from_cow((key, value): (&Cow<'_, str>, &Cow<'_, str>)) -> Option<Self> {
                let value = value.as_ref();
                match key.as_ref() {
                    $($prop => $crate::values::$fun(value).map(Declaration::$item),)*
                    _ => None,
                }
            }
        }

    };
}

gen_declaration! {
    "display" => parse_display Display Display;
    "position" => parse_position Position Position;
    "width" => parse_pxpctauto Width PxPctAuto;
    "height" => parse_pxpctauto Height PxPctAuto;
    "min-width" => parse_pxpctauto MinWidth PxPctAuto;
    "min-height" => parse_pxpctauto MinHeight PxPctAuto;
    "max-width" => parse_pxpctauto MaxWidth PxPctAuto;
    "max-height" => parse_pxpctauto MaxHeight PxPctAuto;
    "flex-direction" => parse_flex_direction FlexDirection FlexDirection;
    "flex-wrap" => parse_flex_wrap FlexWrap FlexWrap;
    "flex-grow" => parse_f32 FlexGrow f32;
    "flex-shrink" => parse_f32 FlexShrink f32;
    "flex-basis" => parse_pxpctauto FlexBasis PxPctAuto;
    "justify-content" => parse_justify_content JustifyContent JustifyContent;
    "justify-self" => parse_align_items JustifySelf AlignItems;
    "align-items" => parse_align_items AlignItems AlignItems;
    "align-content" => parse_align_content AlignContent AlignContent;
    "align-self" => parse_align_items AlignSelf AlignItems;
    "border" => parse_border Border BorderDef;
    "border-width" => parse_px BorderWidth Px;
    "border-left" => parse_px BorderLeft Px;
    "border-top" => parse_px BorderTop Px;
    "border-right" => parse_px BorderRight Px;
    "border-bottom" => parse_px BorderBottom Px;
    "border-radius" => parse_px_pct BorderRadius PxPct;
    "outline-color" => parse_color OutlineColor Color;
    "outline" => parse_px Outline Px;
    "border-color" => parse_color BorderColor Color;
    "padding" => parse_px_pct Padding PxPct;
    "padding-left" => parse_px_pct PaddingLeft PxPct;
    "padding-top" => parse_px_pct PaddingTop PxPct;
    "padding-right" => parse_px_pct PaddingRight PxPct;
    "padding-bottom" => parse_px_pct PaddingBottom PxPct;
    "margin" => parse_pxpctauto Margin PxPctAuto;
    "margin-left" => parse_pxpctauto MarginLeft PxPctAuto;
    "margin-top" => parse_pxpctauto MarginTop PxPctAuto;
    "margin-right" => parse_pxpctauto MarginRight PxPctAuto;
    "margin-bottom" => parse_pxpctauto MarginBottom PxPctAuto;
    "left" => parse_pxpctauto InsetLeft PxPctAuto;
    "top" => parse_pxpctauto InsetTop PxPctAuto;
    "right" => parse_pxpctauto InsetRight PxPctAuto;
    "bottom" => parse_pxpctauto InsetBottom PxPctAuto;
    "z-index" => parse_i32 ZIndex i32;
    "cursor" => parse_cursor_style Cursor CursorIcon;
    "color" => parse_color Color Color;
    "background-color" => parse_color BackgroundColor Color;
    "box-shadow" => parse_box_shadow BoxShadow BoxShadow;
    "font-size" => parse_px FontSize Px;
    "font-family" => to_owned FontFamily String;
    "font-weight" => parse_font_weight FontWeight Weight;
    "font-style" => parse_font_style FontStyle TextStyle;
    "caret-color" => parse_color CursorColor Color;
    "text-wrap" => parse_text_overflow TextOverflow TextOverflow;
    "line-height" => parse_f32 LineHeight f32;
    "aspect-ratio" => parse_f32 AspectRatio f32;
    "column-gap" => parse_px_pct ColGap PxPct;
    "row-gap" => parse_px_pct RowGap PxPct;
    "gap" => parse_gap Gap (PxPct, Option<PxPct>);
    "transition" => parse_transition Transition (String, Transition);
    "user-select" => parse_user_select UserSelect bool;
    // --- CSS3: Visibility & interaction ---
    "opacity" => parse_f32 Opacity f32;
    "visibility" => parse_visibility Visibility Visibility;
    "pointer-events" => parse_pointer_events PointerEvents PointerEvents;
    // --- CSS3: Overflow ---
    "overflow" => parse_overflow_shorthand Overflow OverflowAxes;
    "overflow-x" => parse_overflow_axis OverflowX OverflowAxis;
    "overflow-y" => parse_overflow_axis OverflowY OverflowAxis;
    // --- CSS3: Text ---
    "text-align" => parse_text_align TextAlign TextAlign;
    "text-decoration" => parse_text_decoration TextDecoration TextDecoration;
    "text-transform" => parse_text_transform TextTransform TextTransform;
    "white-space" => parse_white_space WhiteSpace WhiteSpace;
    "word-break" => parse_word_break WordBreak WordBreak;
    "letter-spacing" => parse_px LetterSpacing Px;
    "word-spacing" => parse_px WordSpacing Px;
    // --- CSS3: Transform & Filter ---
    "transform" => parse_transform Transform Transform;
    "filter" => parse_filter Filter Filter;
    "backdrop-filter" => parse_filter BackdropFilter Filter;
    // --- CSS3: Background ---
    "background-image" => parse_background_image BackgroundImage BackgroundImage;
    "background-size" => parse_background_size BackgroundSize BackgroundSize;
    "background-position" => parse_background_position BackgroundPosition (PxPct, PxPct);
    "background-repeat" => parse_background_repeat BackgroundRepeat BackgroundRepeat;
    // --- CSS3: Box model ---
    "box-sizing" => parse_box_sizing BoxSizing BoxSizing;
    "object-fit" => parse_object_fit ObjectFit ObjectFit;
    // --- CSS Grid ---
    "grid-template-columns" => parse_track_list GridTemplateColumns TrackList;
    "grid-template-rows" => parse_track_list GridTemplateRows TrackList;
    "grid-column" => parse_grid_line GridColumn GridPlacement;
    "grid-row" => parse_grid_line GridRow GridPlacement;
}
