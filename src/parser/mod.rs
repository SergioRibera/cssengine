pub mod analyzer;
pub mod at_rules;
pub mod declaration;
pub mod lexer;
pub mod values;

use std::{borrow::Cow, iter::Zip};

use smallvec::SmallVec;

use lexer::Token;

pub use analyzer::{analyze_tokens, Span, SyntaxError};
pub use at_rules::{
    ColorScheme, FontFace, FontSource, ImportRule, Keyframes, KeyframeStop,
    MediaContext, MediaFeature, MediaQuery, MediaRule, Orientation,
};
pub use lexer::Lexer;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Parser that turns lexer tokens into `ParserToken` and builds a `Rule`
pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
}

pub enum ParserToken<'a> {
    Selector { value: &'a str },
    Property { value: &'a str },
    Value { value: &'a str },
}

impl<'a> ParserToken<'a> {
    #[must_use]
    pub fn from_token(token: &Token<'a>) -> Option<Self> {
        match token {
            Token::Selector { value, .. } => Some(ParserToken::Selector { value }),
            Token::Property { value, .. } => Some(ParserToken::Property { value }),
            Token::Value { value, .. } => Some(ParserToken::Value { value }),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Specificity
// ---------------------------------------------------------------------------

/// CSS specificity as `(id, class, element)` — implements `Ord` so that
/// tuples can be compared directly for cascade ordering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Specificity(
    pub u32, // id selectors (#foo)
    pub u32, // class / attr / pseudo-class (.foo, [attr], :hover)
    pub u32, // element selectors (div, span) and pseudo-elements (::before)
);

impl Specificity {
    /// Compute specificity from a single selector string (no comma-separated list).
    pub fn from_selector(s: &str) -> Self {
        let mut id: u32 = 0;
        let mut class: u32 = 0;
        let mut element: u32 = 0;

        let bytes = s.as_bytes();
        let mut i = 0;

        // Advance past an CSS identifier (ident chars: [a-zA-Z0-9_-])
        let skip_ident = |i: &mut usize| {
            while *i < bytes.len()
                && (bytes[*i].is_ascii_alphanumeric() || bytes[*i] == b'-' || bytes[*i] == b'_')
            {
                *i += 1;
            }
        };

        while i < bytes.len() {
            match bytes[i] {
                b'#' => {
                    id += 1;
                    i += 1;
                    skip_ident(&mut i); // skip the id name
                }
                b'.' => {
                    class += 1;
                    i += 1;
                    skip_ident(&mut i); // skip the class name
                }
                b'[' => {
                    class += 1;
                    while i < bytes.len() && bytes[i] != b']' {
                        i += 1;
                    }
                    if i < bytes.len() { i += 1; } // skip ']'
                }
                b':' => {
                    if i + 1 < bytes.len() && bytes[i + 1] == b':' {
                        element += 1;
                        i += 2;
                    } else {
                        class += 1;
                        i += 1;
                    }
                    skip_ident(&mut i);
                    if i < bytes.len() && bytes[i] == b'(' {
                        let mut depth = 1u32;
                        i += 1;
                        while i < bytes.len() && depth > 0 {
                            match bytes[i] {
                                b'(' => depth += 1,
                                b')' => depth -= 1,
                                _ => {}
                            }
                            i += 1;
                        }
                    }
                }
                b'*' | b' ' | b'>' | b'+' | b'~' | b',' => {
                    i += 1;
                }
                c if c.is_ascii_alphabetic() => {
                    element += 1;
                    skip_ident(&mut i);
                }
                _ => {
                    i += 1;
                }
            }
        }

        Specificity(id, class, element)
    }
}

// ---------------------------------------------------------------------------
// PseudoClass
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum PseudoClass {
    // Interaction
    Hover,
    Active,
    ActiveHover,
    Focus,
    FocusHover,
    FocusVisible,
    FocusWithin,
    // State
    Disabled,
    DisabledHover,
    Enabled,
    Checked,
    Indeterminate,
    // Tree-structural
    FirstChild,
    LastChild,
    OnlyChild,
    FirstOfType,
    LastOfType,
    OnlyOfType,
    NthChild(u32, u32), // an+b coefficients
    NthLastChild(u32, u32),
    // Content
    Empty,
    Root,
    // Not / functional
    Not(String), // serialized argument for simplicity
    // Pseudo-elements
    Placeholder,
    Selection,
    Before,
    After,
    FirstLine,
    FirstLetter,
    // Misc
    Any, // :is() / :where() simplified
    Link,
    Visited,
    Target,
}

impl PseudoClass {
    pub fn parse_str(s: &str) -> Option<Self> {
        // Exact matches first (fast path)
        let val = match s {
            ":hover" => Self::Hover,
            ":focus" => Self::Focus,
            ":focus:hover" => Self::FocusHover,
            ":focus-visible" => Self::FocusVisible,
            ":focus-within" => Self::FocusWithin,
            ":active" => Self::Active,
            ":active:hover" => Self::ActiveHover,
            ":disabled" => Self::Disabled,
            ":disabled:hover" => Self::DisabledHover,
            ":enabled" => Self::Enabled,
            ":checked" => Self::Checked,
            ":indeterminate" => Self::Indeterminate,
            ":first-child" => Self::FirstChild,
            ":last-child" => Self::LastChild,
            ":only-child" => Self::OnlyChild,
            ":first-of-type" => Self::FirstOfType,
            ":last-of-type" => Self::LastOfType,
            ":only-of-type" => Self::OnlyOfType,
            ":empty" => Self::Empty,
            ":root" => Self::Root,
            ":link" => Self::Link,
            ":visited" => Self::Visited,
            ":target" => Self::Target,
            "::placeholder" => Self::Placeholder,
            "::selection" => Self::Selection,
            "::before" => Self::Before,
            "::after" => Self::After,
            "::first-line" => Self::FirstLine,
            "::first-letter" => Self::FirstLetter,
            _ => {
                // Try functional pseudo-classes
                return Self::parse_functional(s);
            }
        };
        Some(val)
    }

    fn parse_functional(s: &str) -> Option<Self> {
        if let Some(arg) = s.strip_prefix(":nth-child(").and_then(|s| s.strip_suffix(')')) {
            return Some(Self::NthChild(
                parse_nth_a(arg),
                parse_nth_b(arg),
            ));
        }
        if let Some(arg) = s.strip_prefix(":nth-last-child(").and_then(|s| s.strip_suffix(')')) {
            return Some(Self::NthLastChild(
                parse_nth_a(arg),
                parse_nth_b(arg),
            ));
        }
        if let Some(arg) = s.strip_prefix(":not(").and_then(|s| s.strip_suffix(')')) {
            return Some(Self::Not(arg.to_owned()));
        }
        if s.starts_with(":is(") || s.starts_with(":where(") {
            return Some(Self::Any);
        }
        None
    }
}

/// Parse `a` from `an+b` notation (returns 0 for `odd`/`even`/plain `b`)
fn parse_nth_a(s: &str) -> u32 {
    let s = s.trim();
    if s == "odd" { return 2; }
    if s == "even" { return 2; }
    if let Some(n_pos) = s.find('n') {
        s[..n_pos].trim().parse::<u32>().unwrap_or(1)
    } else {
        0
    }
}

/// Parse `b` from `an+b` notation
fn parse_nth_b(s: &str) -> u32 {
    let s = s.trim();
    if s == "odd" { return 1; }
    if s == "even" { return 0; }
    if let Some(n_pos) = s.find('n') {
        let rest = s[n_pos + 1..].trim();
        rest.trim_start_matches('+').parse::<u32>().unwrap_or(0)
    } else {
        s.parse::<u32>().unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Selector
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Selector {
    pub selector: String,
    pub pseudo_class: Option<PseudoClass>,
    pub specificity: Specificity,
}

fn split_value(value: &str) -> Selector {
    if let Some(colon_col) = value.find(':') {
        let base = &value[..colon_col];
        let pseudo_str = &value[colon_col..];
        let pseudo = PseudoClass::parse_str(pseudo_str);
        Selector {
            specificity: Specificity::from_selector(value),
            selector: base.to_owned(),
            pseudo_class: pseudo,
        }
    } else {
        Selector {
            specificity: Specificity::from_selector(value),
            selector: value.to_owned(),
            pseudo_class: None,
        }
    }
}

#[cold]
fn split_double_colon(value: &str) -> Selector {
    // `::placeholder`, `::selection:hover`, etc.
    // The base selector is everything before the `::` start
    // Since value starts with `::`, the "base" selector is empty
    // (pseudo-element IS the selector here, like `::placeholder` on an input)
    //
    // But sometimes it's `.class::before` — in that case value won't start with `::`
    // This function is only called when value.starts_with("::"), so base is "".
    Selector {
        specificity: Specificity::from_selector(value),
        selector: String::new(),
        pseudo_class: PseudoClass::parse_str(value),
    }
}

impl<'a> From<&'a str> for Selector {
    #[inline]
    fn from(value: &'a str) -> Self {
        if value == ":root" {
            return Self {
                specificity: Specificity(0, 0, 0),
                selector: value.to_owned(),
                pseudo_class: None,
            };
        }
        if value.starts_with("::") {
            return split_double_colon(value);
        }
        split_value(value)
    }
}

// ---------------------------------------------------------------------------
// Rule
// ---------------------------------------------------------------------------

pub struct Rule<'a> {
    pub selectors: SmallVec<[Selector; 1]>,
    pub properties: SmallVec<[Cow<'a, str>; 1]>,
    pub values: SmallVec<[Cow<'a, str>; 1]>,
}

impl Rule<'_> {
    #[must_use]
    pub const fn new_const() -> Self {
        Self {
            selectors: SmallVec::<[Selector; 1]>::new_const(),
            properties: SmallVec::<[Cow<'_, str>; 1]>::new_const(),
            values: SmallVec::<[Cow<'_, str>; 1]>::new_const(),
        }
    }

    pub fn iter_props(
        &self,
    ) -> Zip<std::slice::Iter<'_, Cow<'_, str>>, std::slice::Iter<'_, Cow<'_, str>>> {
        self.properties.iter().zip(self.values.iter())
    }

    pub fn remove(&mut self, index: usize) {
        self.properties.remove(index);
        self.values.remove(index);
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

impl<'a> Parser<'a> {
    #[must_use]
    pub const fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens }
    }

    fn selector_count(&self) -> usize {
        self.tokens
            .iter()
            .filter(|t| matches!(t, Token::Selector { .. }))
            .count()
    }

    #[must_use]
    pub fn parse(self) -> Vec<Rule<'a>> {
        let mut rules = Vec::with_capacity(self.selector_count());
        let mut props = SmallVec::<[ParserToken; 16]>::new_const();
        let mut tokens = self
            .tokens
            .iter()
            .filter_map(ParserToken::from_token)
            .peekable();
        'main: loop {
            let Some(token) = tokens.next() else {
                break 'main;
            };

            let ParserToken::Selector { value: selector } = token else {
                continue 'main;
            };

            let mut rule = Rule::new_const();
            props.clear();
            rule.selectors
                .extend(selector.split(',').map(str::trim).map(Selector::from));
            'props: loop {
                let Some(peek) = tokens.peek() else {
                    break 'props;
                };
                if matches!(peek, ParserToken::Selector { .. }) {
                    break 'props;
                }
                if let Some(next) = tokens.next() {
                    props.push(next);
                }
            }
            for chunk in props.chunks_exact(2) {
                if let [ParserToken::Property { value: prop_value }, ParserToken::Value { value }] =
                    chunk
                {
                    rule.properties.push(Cow::Borrowed(prop_value));
                    rule.values.push(Cow::Borrowed(value));
                }
            }
            rules.push(rule);
        }
        rules
    }
}

// ---------------------------------------------------------------------------
// CSS Variable replacement
// ---------------------------------------------------------------------------

pub(crate) fn replace_vars(mut rules: Vec<Rule<'_>>) -> Vec<Rule<'_>> {
    let Some(root_idx) = rules
        .iter()
        .position(|rule| rule.selectors.iter().any(|s| s.selector == ":root"))
    else {
        return rules;
    };

    replace_root_vars(root_idx, &mut rules);
    rules
}

fn replace_root_vars(root_idx: usize, rules: &mut Vec<Rule>) {
    let root = &mut rules[root_idx];
    let mut indexes = SmallVec::<[usize; 16]>::new();
    let mut keys = SmallVec::<[String; 16]>::new();
    let mut values = SmallVec::<[String; 16]>::new();
    find_vars(root, &mut indexes, &mut keys, &mut values);
    remove_vars(root, &indexes);
    let mut replace_indexes = SmallVec::<[usize; 16]>::new();
    for rule in rules {
        replace_indexes.clear();
        let indexes_iter = rule
            .values
            .iter()
            .enumerate()
            .filter_map(|(i, v)| is_var_reference(v).then_some(i));
        replace_indexes.extend(indexes_iter);
        for idx in replace_indexes.iter() {
            replace_var(&mut rule.values[*idx], &keys, &values);
        }
    }
}

fn replace_var(value: &mut Cow<'_, str>, keys: &[String], values: &[String]) {
    let var_name = get_var_name(value);
    if let Some(idx) = keys.iter().position(|k| k == var_name) {
        *value = Cow::Owned(values[idx].clone());
    }
}

fn remove_vars(root: &mut Rule, indexes: &[usize]) {
    for index in indexes.iter().rev() {
        root.remove(*index);
    }
}

fn find_vars(
    root: &mut Rule,
    indexes: &mut SmallVec<[usize; 16]>,
    keys: &mut SmallVec<[String; 16]>,
    values: &mut SmallVec<[String; 16]>,
) {
    for (i, (k, v)) in root.iter_props().enumerate() {
        if is_var_definition(k) {
            indexes.push(i);
            keys.push(k.to_string());
            values.push(v.to_string());
        }
    }
}

fn is_var_definition(value: &str) -> bool {
    value.starts_with("--")
}

fn is_var_reference(value: &str) -> bool {
    value.ends_with(')') && value.starts_with("var(")
}

#[cold]
#[inline(never)]
fn get_var_name(value: &str) -> &str {
    // Handle `var(--name, fallback)` — use only the part before the first `,`
    let inner = &value[4..value.len() - 1];
    if let Some(comma) = inner.find(',') {
        inner[..comma].trim()
    } else {
        inner.trim()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_vars_ok() {
        let rules = vec![
            Rule {
                selectors: SmallVec::from_vec(vec![Selector {
                    selector: ":root".to_owned(),
                    pseudo_class: None,
                    specificity: Specificity::default(),
                }]),
                properties: SmallVec::from_buf([Cow::Borrowed("--main-color")]),
                values: SmallVec::from_buf([Cow::Borrowed("blue")]),
            },
            Rule {
                selectors: SmallVec::from_buf([Selector {
                    selector: ".button".to_owned(),
                    pseudo_class: None,
                    specificity: Specificity(0, 1, 0),
                }]),
                properties: SmallVec::from_buf([Cow::Borrowed("background-color")]),
                values: SmallVec::from_buf([Cow::Borrowed("var(--main-color)")]),
            },
        ];

        let updated_rules = replace_vars(rules);
        assert_eq!(updated_rules[1].values[0], Cow::Borrowed("blue"));
    }

    #[test]
    fn var_fallback_syntax() {
        // var(--missing, red) should use "red" as var name extraction returns "--missing"
        assert_eq!(get_var_name("var(--main-color, red)"), "--main-color");
        assert_eq!(get_var_name("var(--main-color)"), "--main-color");
    }

    #[test]
    fn specificity_basic() {
        assert_eq!(Specificity::from_selector("#id"), Specificity(1, 0, 0));
        assert_eq!(Specificity::from_selector(".class"), Specificity(0, 1, 0));
        assert_eq!(Specificity::from_selector("div"), Specificity(0, 0, 1));
        assert_eq!(Specificity::from_selector("*"), Specificity(0, 0, 0));
        assert_eq!(
            Specificity::from_selector("#id .class div"),
            Specificity(1, 1, 1)
        );
    }

    #[test]
    fn specificity_ordering() {
        let id = Specificity(1, 0, 0);
        let class = Specificity(0, 1, 0);
        let element = Specificity(0, 0, 1);
        assert!(id > class);
        assert!(class > element);
        assert!(id > element);
    }

    #[test]
    fn pseudo_class_parsing() {
        assert_eq!(PseudoClass::parse_str(":hover"), Some(PseudoClass::Hover));
        assert_eq!(PseudoClass::parse_str(":first-child"), Some(PseudoClass::FirstChild));
        assert_eq!(PseudoClass::parse_str(":checked"), Some(PseudoClass::Checked));
        assert_eq!(
            PseudoClass::parse_str(":nth-child(2n+1)"),
            Some(PseudoClass::NthChild(2, 1))
        );
        assert!(PseudoClass::parse_str(":unknown-pseudo").is_none());
    }
}
