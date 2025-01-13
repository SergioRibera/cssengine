use smallvec::SmallVec;

use crate::declaration::Declaration;
use crate::{css_to_rules, PseudoClass, Selector};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "tailwind_colors")]
use crate::{TAILWIND_COLORS, TAILWIND_NAME_COLORS};

/// Represents a CSS style sheet with parsed rules and declarations.
///
/// # Fields
/// - `rules`: A collection of selectors and their corresponding declarations.
///
/// # Example
/// ```
/// let css = ".example { color: red; }";
/// let stylesheet = cssengine::StyleSheet::from_css(css);
/// ```
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct StyleSheet {
    rules: SmallVec<[(Selector, SmallVec<[Declaration; 32]>); 32]>,
    tmp_generated: SmallVec<[Declaration; 32]>,
}

impl StyleSheet {
    pub const fn new_const() -> Self {
        Self {
            rules: SmallVec::new_const(),
            tmp_generated: SmallVec::new_const(),
        }
    }

    /// Creates a `StyleSheet` from a CSS input string.
    ///
    /// # Parameters
    /// - `input`: The CSS input string.
    ///
    /// # Returns
    /// A new `StyleSheet` instance containing the parsed rules.
    ///
    /// # Example
    /// ```
    /// let css = ".example { color: red; }";
    /// let stylesheet = cssengine::StyleSheet::from_css(css);
    /// ```
    pub fn from_css<'a>(input: &'a str) -> Self {
        let rules = css_to_rules(input)
            .unwrap_or_default()
            .into_iter()
            .map(|rule| {
                let declarations: SmallVec<[Declaration; 32]> = rule
                    .iter_props()
                    .filter_map(Declaration::from_cow)
                    .collect();
                rule.selectors
                    .into_iter()
                    .map(|selector| (selector, declarations.clone()))
                    .collect::<SmallVec<[(Selector, SmallVec<_>); 32]>>()
            })
            .flatten()
            .collect();

        Self {
            rules,
            tmp_generated: SmallVec::new(),
        }
    }

    /// Retrieves the styles for a given selector.
    ///
    /// # Parameters
    /// - `selector`: A CSS selector (e.g., `.class`, `#id`).
    ///
    /// # Returns
    /// - `Some(SmallVec<Declaration>)` if styles exist for the selector.
    /// - `None` if no matching styles are found.
    ///
    /// # Example
    /// ```
    /// let css = ".example { color: red; }";
    /// let mut stylesheet = cssengine::StyleSheet::from_css(css);
    /// if let Some(styles) = stylesheet.get_styles(".example") {
    ///     println!("Styles: {:?}", styles);
    /// }
    /// ```
    pub fn get_styles(
        &mut self,
        selector: impl AsRef<str>,
    ) -> SmallVec<[(Option<PseudoClass>, SmallVec<[Declaration; 32]>); 4]> {
        let selector = selector.as_ref();
        // If I don't have this I would have to do things with the features macro and it doesn't look as nice.
        #[allow(unused_mut)]
        let mut found = self
            .rules
            .iter()
            .filter_map(|(sel, decls)| {
                if selector
                    .split_whitespace()
                    .any(|s| sel.selector.contains(s.trim()))
                {
                    Some((sel.pseudo_class.clone(), decls.clone()))
                } else {
                    None
                }
            })
            .fold(SmallVec::new(), |mut acc, (pseudo_class, declarations)| {
                if let Some(existing) = acc
                    .iter_mut()
                    .find(|(ps_class, _): &&mut (Option<_>, SmallVec<_>)| ps_class == &pseudo_class)
                {
                    // If the pseudo-class already exists, extend its declarations.
                    existing.1.extend(declarations);
                } else {
                    // Otherwise, add it as a new entry.
                    acc.push((pseudo_class, declarations));
                }
                acc
            });

        #[cfg(feature = "tailwind_colors")]
        if !found.is_empty() {
            return found;
        }

        #[cfg(feature = "tailwind_colors")]
        {
            let selector = Selector::from(selector);

            for token in selector.selector.split_whitespace() {
                let Some(class) = token.strip_prefix('.') else {
                    continue;
                };

                for prefix in ["bg", "text", "border", "outline"] {
                    let Some(class) = class.strip_prefix(&format!("{prefix}-")) else {
                        continue;
                    };
                    let class = class.to_ascii_lowercase();
                    let Some((name, tone_str)) = class.split_once('-') else {
                        continue;
                    };

                    if !TAILWIND_NAME_COLORS.contains(&name) {
                        continue;
                    }
                    let Ok(tone) = tone_str.parse::<usize>() else {
                        continue;
                    };
                    let Some(color) = TAILWIND_COLORS
                        .get(format!("{name}-{tone}").as_str())
                        .cloned()
                    else {
                        continue;
                    };

                    let declaration = match prefix {
                        "bg" => Declaration::BackgroundColor(color),
                        "text" => Declaration::Color(color),
                        "border" => Declaration::BorderColor(color),
                        "outline" => Declaration::OutlineColor(color),
                        _ => continue,
                    };
                    self.tmp_generated.push(declaration);
                }
            }

            if !self.tmp_generated.is_empty() {
                self.rules
                    .push((selector.clone(), self.tmp_generated.clone()));
                found.push((selector.pseudo_class, self.tmp_generated.clone()));
            }

            self.tmp_generated.clear();
        }

        found
    }
}
