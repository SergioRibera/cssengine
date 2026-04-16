use std::borrow::Cow;
use std::collections::HashMap;
use std::mem::discriminant;

use smallvec::SmallVec;

use crate::declaration::Declaration;
use crate::parser::at_rules::{
    FontFace, ImportRule, Keyframes, KeyframeStop, MediaContext, MediaRule,
    parse_font_face, parse_import_rule, parse_media_query,
};
use crate::parser::lexer::Token;
use crate::parser::{Parser, replace_vars};
use crate::{PseudoClass, Selector, Specificity};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "tailwind_colors")]
use crate::{TAILWIND_COLORS, TAILWIND_NAME_COLORS};

/// Represents a CSS style sheet with parsed rules and declarations.
///
/// # Example
/// ```
/// let css = ".example { color: red; }";
/// let stylesheet = cssengine::StyleSheet::from_css(css);
/// ```
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct StyleSheet {
    /// Each entry is `(selector, source_index, declarations)`.
    /// `source_index` is the order in which rules were parsed (used for cascade).
    rules: Vec<(Selector, usize, Vec<Declaration>)>,
    media_rules: Vec<MediaRule>,
    keyframes: HashMap<String, Keyframes>,
    font_faces: Vec<FontFace>,
    /// Unresolved `@import` rules, in source order.
    import_rules: Vec<ImportRule>,
    #[allow(dead_code)]
    tmp_generated: Vec<Declaration>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            media_rules: Vec::new(),
            keyframes: HashMap::new(),
            font_faces: Vec::new(),
            import_rules: Vec::new(),
            tmp_generated: Vec::new(),
        }
    }

    /// Creates a `StyleSheet` from a CSS input string.
    ///
    /// # Example
    /// ```
    /// let css = ".example { color: red; }";
    /// let stylesheet = cssengine::StyleSheet::from_css(css);
    /// ```
    pub fn from_css(input: &str) -> Self {
        use crate::parser::lexer::Lexer;

        let tokens = Lexer::new(input).tokens();

        let mut source_index = 0usize;
        let mut media_rules_list: Vec<MediaRule> = Vec::new();
        let mut keyframes_map: HashMap<String, Keyframes> = HashMap::new();
        let mut font_faces_list: Vec<FontFace> = Vec::new();
        let mut import_rules_list: Vec<ImportRule> = Vec::new();
        let mut top_tokens: Vec<Token<'_>> = Vec::with_capacity(tokens.len());

        // Pass 1: walk through all tokens, routing at-rule blocks to parsers
        // and collecting top-level tokens for the normal CSS parser.
        let n = tokens.len();
        let mut i = 0;
        while i < n {
            match tokens[i] {
                Token::AtRule { keyword, prelude, .. } => {
                    i += 1;
                    if i < n && matches!(tokens[i], Token::BlockOpen { .. }) {
                        i += 1; // consume BlockOpen

                        // Find the matching BlockClose at depth 0
                        let inner_start = i;
                        let mut depth = 1u32;
                        while i < n {
                            match tokens[i] {
                                Token::BlockOpen { .. } => depth += 1,
                                Token::BlockClose { .. } => {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                }
                                _ => {}
                            }
                            i += 1;
                        }
                        let inner = &tokens[inner_start..i];
                        if i < n {
                            i += 1; // consume the outer BlockClose
                        }

                        process_at_rule(
                            keyword,
                            prelude,
                            inner,
                            &mut source_index,
                            &mut media_rules_list,
                            &mut keyframes_map,
                            &mut font_faces_list,
                        );
                    }
                    // Statement at-rule (no block): @import, @charset, @namespace, …
                    else {
                        if keyword == "import" {
                            if let Some(ir) = parse_import_rule(prelude) {
                                import_rules_list.push(ir);
                            }
                        }
                        // Skip to the terminating semicolon (already peeked past by scan_at_rule)
                        while i < n && !matches!(tokens[i], Token::Semicolon { .. } | Token::EOF) {
                            i += 1;
                        }
                        if i < n {
                            i += 1;
                        }
                    }
                }
                t => {
                    top_tokens.push(t);
                    i += 1;
                }
            }
        }

        // Pass 2: parse top-level tokens as regular CSS rules with var substitution.
        let normal_rules = Parser::new(top_tokens).parse();
        let normal_rules = replace_vars(normal_rules);
        let rules: Vec<(Selector, usize, Vec<Declaration>)> = normal_rules
            .into_iter()
            .flat_map(|rule| {
                let decls: Vec<Declaration> = rule
                    .iter_props()
                    .filter_map(Declaration::from_cow)
                    .collect();
                rule.selectors
                    .into_iter()
                    .map(|sel| {
                        let idx = source_index;
                        source_index += 1;
                        (sel, idx, decls.clone())
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        Self {
            rules,
            media_rules: media_rules_list,
            keyframes: keyframes_map,
            font_faces: font_faces_list,
            import_rules: import_rules_list,
            tmp_generated: Vec::new(),
        }
    }

    // ---------------------------------------------------------------------------
    // Accessors for at-rule data
    // ---------------------------------------------------------------------------

    pub fn media_rules(&self) -> &[MediaRule] {
        &self.media_rules
    }

    pub fn keyframes(&self) -> &HashMap<String, Keyframes> {
        &self.keyframes
    }

    pub fn font_faces(&self) -> &[FontFace] {
        &self.font_faces
    }

    /// Unresolved `@import` rules, in source order.
    ///
    /// Use `resolve_imports` to load their content and merge it into this stylesheet.
    pub fn import_rules(&self) -> &[ImportRule] {
        &self.import_rules
    }

    // ---------------------------------------------------------------------------
    // Mutation and composition
    // ---------------------------------------------------------------------------

    /// Merge another `StyleSheet` into this one.
    ///
    /// Rules from `other` are appended after the existing rules, preserving cascade
    /// ordering (rules from `other` win on equal specificity because they have higher
    /// source indices).
    ///
    /// Unresolved `@import` rules from `other` are also transferred so that a
    /// subsequent `resolve_imports` call handles them.
    pub fn merge(&mut self, other: StyleSheet) {
        let base = self.rules.len();
        for (sel, src_idx, decls) in other.rules {
            self.rules.push((sel, src_idx + base, decls));
        }
        self.media_rules.extend(other.media_rules);
        for (name, kf) in other.keyframes {
            self.keyframes.insert(name, kf);
        }
        self.font_faces.extend(other.font_faces);
        self.import_rules.extend(other.import_rules);
    }

    /// Resolve `@import` rules by loading their content with `loader`.
    ///
    /// `loader` receives the URL string exactly as written in the stylesheet and
    /// returns `Some(css_string)` if it can supply the content, or `None` to leave
    /// the import unresolved (it stays in `import_rules()`).
    ///
    /// Rules loaded from an unconditional import are merged directly.  Rules loaded
    /// from a conditional import (e.g. `@import "mobile.css" (max-width: 768px)`) are
    /// wrapped in a synthetic `@media` rule so they are only applied via
    /// `get_styles_with_media`.
    ///
    /// **Nested imports** (imports inside imported files) are also resolved
    /// automatically using the same loader.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// let mut files: HashMap<&str, &str> = HashMap::new();
    /// files.insert("base.css", ".root { color: red; }");
    ///
    /// let mut ss = cssengine::StyleSheet::from_css(r#"@import "base.css"; .a { color: blue; }"#);
    /// ss.resolve_imports(|url| files.get(url).map(|s| s.to_string()));
    ///
    /// // .root is now available
    /// assert!(!ss.get_styles(".root").is_empty());
    /// ```
    pub fn resolve_imports<F>(&mut self, loader: F)
    where
        F: Fn(&str) -> Option<String> + Copy,
    {
        let imports = std::mem::take(&mut self.import_rules);
        let mut unresolved = Vec::new();

        for import_rule in imports {
            let Some(css) = loader(&import_rule.url) else {
                unresolved.push(import_rule);
                continue;
            };

            let mut sub = StyleSheet::from_css(&css);
            // Recursively resolve nested imports in the sub-stylesheet
            sub.resolve_imports(loader);

            match import_rule.media {
                None => {
                    // Unconditional import: merge everything directly
                    self.merge(sub);
                }
                Some(query) => {
                    // Conditional import: top-level rules become a media rule;
                    // keyframes/font-faces are unconditional per the CSS spec.
                    if !sub.rules.is_empty() {
                        self.media_rules.push(MediaRule { query: query.clone(), rules: sub.rules });
                    }
                    // Sub-stylesheet's own media rules carry their own conditions
                    self.media_rules.extend(sub.media_rules);
                    for (name, kf) in sub.keyframes {
                        self.keyframes.entry(name).or_insert(kf);
                    }
                    self.font_faces.extend(sub.font_faces);
                }
            }
        }

        self.import_rules = unresolved;
    }

    /// Parse `input` as CSS and add the resulting rules to this stylesheet.
    pub fn add_css(&mut self, input: &str) {
        self.merge(StyleSheet::from_css(input));
    }

    /// Remove all rules for the given exact selector string. Returns `true` if any were removed.
    pub fn remove_selector(&mut self, selector: &str) -> bool {
        let before = self.rules.len();
        self.rules.retain(|(sel, _, _)| sel.selector != selector);
        self.rules.len() < before
    }

    // ---------------------------------------------------------------------------
    // Iterators
    // ---------------------------------------------------------------------------

    /// Iterate over all parsed top-level rules as `(selector, source_index, declarations)`.
    pub fn rules(&self) -> impl Iterator<Item = &(Selector, usize, Vec<Declaration>)> {
        self.rules.iter()
    }

    /// Iterate over all declarations matching the given selector (ignores pseudo-classes).
    pub fn declarations_for<'a>(
        &'a self,
        selector: &'a str,
    ) -> impl Iterator<Item = &'a Declaration> {
        self.rules
            .iter()
            .filter(move |(sel, _, _)| sel.selector.trim() == selector.trim())
            .flat_map(|(_, _, decls)| decls.iter())
    }

    // ---------------------------------------------------------------------------
    // Style lookup
    // ---------------------------------------------------------------------------

    /// Retrieves the styles for a given selector.
    ///
    /// Returns declarations grouped by pseudo-class, with CSS cascade applied:
    /// higher-specificity and later-source-order declarations override earlier ones.
    ///
    /// # Example
    /// ```
    /// let css = ".example { color: red; }";
    /// let mut stylesheet = cssengine::StyleSheet::from_css(css);
    /// let styles = stylesheet.get_styles(".example");
    /// if !styles.is_empty() {
    ///     println!("Styles: {:?}", styles);
    /// }
    /// ```
    pub fn get_styles(
        &mut self,
        selector: impl AsRef<str>,
    ) -> SmallVec<[(Option<PseudoClass>, Vec<Declaration>); 4]> {
        let selector = selector.as_ref();

        // Collect all matching rules with (specificity, source_index, pseudo_class, declarations).
        // Exact token matching: stored selector must equal the query token exactly.
        #[allow(unused_mut)]
        let mut matches: Vec<(Specificity, usize, Option<PseudoClass>, &Vec<Declaration>)> =
            self.rules
                .iter()
                .filter_map(|(sel, source_idx, decls)| {
                    let matches_selector = selector
                        .split_whitespace()
                        .any(|query_token| sel.selector.trim() == query_token.trim());
                    if matches_selector {
                        Some((sel.specificity, *source_idx, sel.pseudo_class.clone(), decls))
                    } else {
                        None
                    }
                })
                .collect();

        if matches.is_empty() {
            #[cfg(feature = "tailwind_colors")]
            return self.generate_tailwind(selector);
            #[cfg(not(feature = "tailwind_colors"))]
            return SmallVec::new();
        }

        Self::apply_cascade(matches)
    }

    /// Like `get_styles`, but also includes declarations from matching `@media` rules.
    ///
    /// Media-rule declarations override base declarations for the same property,
    /// matching real browser cascade behaviour.
    pub fn get_styles_with_media(
        &mut self,
        selector: impl AsRef<str>,
        ctx: &MediaContext,
    ) -> SmallVec<[(Option<PseudoClass>, Vec<Declaration>); 4]> {
        let selector = selector.as_ref();
        let mut result = self.get_styles(selector);

        for media_rule in &self.media_rules {
            if !media_rule.query.matches(ctx) {
                continue;
            }
            for (sel, _, decls) in &media_rule.rules {
                let matched = selector
                    .split_whitespace()
                    .any(|q| sel.selector.trim() == q.trim());
                if !matched {
                    continue;
                }
                let pseudo_class = sel.pseudo_class.clone();
                if let Some(group) = result.iter_mut().find(|(ps, _)| *ps == pseudo_class) {
                    for new_decl in decls.iter() {
                        let new_disc = discriminant(new_decl);
                        if let Some(existing) =
                            group.1.iter_mut().find(|d| discriminant(*d) == new_disc)
                        {
                            *existing = new_decl.clone();
                        } else {
                            group.1.push(new_decl.clone());
                        }
                    }
                } else {
                    result.push((pseudo_class, decls.clone()));
                }
            }
        }

        result
    }

    // ---------------------------------------------------------------------------
    // Internal helpers
    // ---------------------------------------------------------------------------

    fn apply_cascade(
        mut matches: Vec<(Specificity, usize, Option<PseudoClass>, &Vec<Declaration>)>,
    ) -> SmallVec<[(Option<PseudoClass>, Vec<Declaration>); 4]> {
        // Sort by (specificity, source_index) ascending so the last element wins per-property
        matches.sort_by_key(|(spec, src, _, _)| (*spec, *src));

        let mut result: SmallVec<[(Option<PseudoClass>, Vec<Declaration>); 4]> = SmallVec::new();

        for (_, _, pseudo_class, declarations) in matches {
            if let Some(group) = result.iter_mut().find(|(ps, _)| *ps == pseudo_class) {
                for new_decl in declarations.iter() {
                    let new_disc = discriminant(new_decl);
                    if let Some(existing) =
                        group.1.iter_mut().find(|d| discriminant(*d) == new_disc)
                    {
                        *existing = new_decl.clone();
                    } else {
                        group.1.push(new_decl.clone());
                    }
                }
            } else {
                result.push((pseudo_class, declarations.clone()));
            }
        }

        result
    }

    #[cfg(feature = "tailwind_colors")]
    fn generate_tailwind(
        &mut self,
        selector: &str,
    ) -> SmallVec<[(Option<PseudoClass>, Vec<Declaration>); 4]> {
        let parsed = Selector::from(selector);

        for token in parsed.selector.split_whitespace() {
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
                let Some(color) =
                    TAILWIND_COLORS.get(format!("{name}-{tone}").as_str()).cloned()
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

        let mut found = SmallVec::new();
        if !self.tmp_generated.is_empty() {
            let source_idx = self.rules.len();
            self.rules
                .push((parsed.clone(), source_idx, self.tmp_generated.clone()));
            found.push((parsed.pseudo_class, self.tmp_generated.clone()));
        }

        self.tmp_generated.clear();
        found
    }
}

impl Default for StyleSheet {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// At-rule token processing helpers
// ---------------------------------------------------------------------------

fn process_at_rule<'a>(
    keyword: &str,
    prelude: &str,
    inner: &[Token<'a>],
    source_index: &mut usize,
    media_rules: &mut Vec<MediaRule>,
    keyframes: &mut HashMap<String, Keyframes>,
    font_faces: &mut Vec<FontFace>,
) {
    match keyword {
        "media" => {
            let query = parse_media_query(prelude);
            let inner_parsed = Parser::new(inner.to_vec()).parse();
            let rules: Vec<_> = inner_parsed
                .into_iter()
                .flat_map(|rule| {
                    let decls: Vec<Declaration> = rule
                        .iter_props()
                        .filter_map(Declaration::from_cow)
                        .collect();
                    rule.selectors
                        .into_iter()
                        .map(|sel| {
                            let idx = *source_index;
                            *source_index += 1;
                            (sel, idx, decls.clone())
                        })
                        .collect::<Vec<_>>()
                })
                .collect();
            media_rules.push(MediaRule { query, rules });
        }
        "keyframes" | "-webkit-keyframes" => {
            let stops = parse_keyframe_stops(inner);
            keyframes.insert(
                prelude.to_owned(),
                Keyframes { name: prelude.to_owned(), stops },
            );
        }
        "font-face" => {
            let pairs = extract_decl_pairs(inner);
            if let Some(ff) = parse_font_face(&pairs) {
                font_faces.push(ff);
            }
        }
        _ => {} // unknown at-rules are silently ignored
    }
}

fn parse_keyframe_stops(tokens: &[Token<'_>]) -> Vec<KeyframeStop> {
    let mut stops = Vec::new();
    let n = tokens.len();
    let mut i = 0;

    while i < n {
        if let Token::Selector { value, .. } = tokens[i] {
            let at = parse_keyframe_position(value);
            i += 1;

            if i < n && matches!(tokens[i], Token::BlockOpen { .. }) {
                i += 1;
                let decl_start = i;
                while i < n && !matches!(tokens[i], Token::BlockClose { .. }) {
                    i += 1;
                }
                let decl_tokens = &tokens[decl_start..i];
                if i < n {
                    i += 1; // consume BlockClose
                }

                let declarations = parse_decls_from_tokens(decl_tokens);
                stops.push(KeyframeStop { at, declarations });
            }
        } else {
            i += 1;
        }
    }
    stops
}

fn parse_keyframe_position(s: &str) -> f32 {
    let s = s.trim();
    if s == "from" {
        return 0.0;
    }
    if s == "to" {
        return 1.0;
    }
    if let Some(pct) = s.strip_suffix('%') {
        return pct.trim().parse::<f32>().unwrap_or(0.0) / 100.0;
    }
    s.parse::<f32>().unwrap_or(0.0)
}

fn parse_decls_from_tokens(tokens: &[Token<'_>]) -> Vec<Declaration> {
    let mut result = Vec::new();
    let n = tokens.len();
    let mut i = 0;
    while i < n {
        if let Token::Property { value: prop, .. } = tokens[i] {
            if i + 2 < n {
                if let Token::Value { value: val, .. } = tokens[i + 2] {
                    let k = Cow::Borrowed(prop);
                    let v = Cow::Borrowed(val);
                    if let Some(decl) = Declaration::from_cow((&k, &v)) {
                        result.push(decl);
                    }
                    i += 3;
                    continue;
                }
            }
        }
        i += 1;
    }
    result
}

fn extract_decl_pairs(tokens: &[Token<'_>]) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let n = tokens.len();
    let mut i = 0;
    while i < n {
        if let Token::Property { value: prop, .. } = tokens[i] {
            if i + 2 < n {
                if let Token::Value { value: val, .. } = tokens[i + 2] {
                    pairs.push((prop.to_owned(), val.to_owned()));
                    i += 3;
                    continue;
                }
            }
        }
        i += 1;
    }
    pairs
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::at_rules::MediaContext;

    #[test]
    fn media_rule_stored() {
        let css = "@media (max-width: 600px) { .btn { color: blue; } }";
        let ss = StyleSheet::from_css(css);
        assert_eq!(ss.media_rules().len(), 1, "expected one @media rule");
        let mr = &ss.media_rules()[0];
        assert!(!mr.rules.is_empty(), "media rule should have inner rules");
    }

    #[test]
    fn get_styles_does_not_return_media_rules() {
        let css = "@media (max-width: 600px) { .btn { color: blue; } }";
        let mut ss = StyleSheet::from_css(css);
        // .btn is only in @media, so regular get_styles should return nothing
        let styles = ss.get_styles(".btn");
        assert!(styles.is_empty(), "get_styles should not include @media rules");
    }

    #[test]
    fn get_styles_with_media_applies_when_matches() {
        let css = ".btn { color: red; } @media (max-width: 600px) { .btn { color: blue; } }";
        let mut ss = StyleSheet::from_css(css);

        let small_ctx = MediaContext { width: 400.0, ..Default::default() };
        let large_ctx = MediaContext { width: 800.0, ..Default::default() };

        let small_styles = ss.get_styles_with_media(".btn", &small_ctx);
        let large_styles = ss.get_styles_with_media(".btn", &large_ctx);

        // For small viewport: media query matches, color should be blue
        let small_color = small_styles
            .iter()
            .find(|(ps, _)| ps.is_none())
            .and_then(|(_, decls)| decls.iter().find(|d| matches!(d, Declaration::Color(_))))
            .cloned();
        assert!(small_color.is_some(), "should have color for small viewport");
        if let Some(Declaration::Color(c)) = small_color {
            // blue = rgb(0,0,255)
            assert!((c.b - 1.0).abs() < 0.01, "color should be blue (b≈1.0), got {c:?}");
        }

        // For large viewport: media query doesn't match, color should be red
        let large_color = large_styles
            .iter()
            .find(|(ps, _)| ps.is_none())
            .and_then(|(_, decls)| decls.iter().find(|d| matches!(d, Declaration::Color(_))))
            .cloned();
        if let Some(Declaration::Color(c)) = large_color {
            assert!((c.r - 1.0).abs() < 0.01, "color should be red (r≈1.0), got {c:?}");
        }
    }

    #[test]
    fn keyframes_stored() {
        let css = "@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }";
        let ss = StyleSheet::from_css(css);
        let kf = ss.keyframes().get("fadeIn");
        assert!(kf.is_some(), "fadeIn keyframes should be stored");
        let kf = kf.unwrap();
        assert_eq!(kf.stops.len(), 2);
        assert!((kf.stops[0].at).abs() < 0.01);
        assert!((kf.stops[1].at - 1.0).abs() < 0.01);
    }

    #[test]
    fn font_face_stored() {
        let css = r#"@font-face { font-family: "MyFont"; src: url("/fonts/myfont.woff2"); }"#;
        let ss = StyleSheet::from_css(css);
        assert_eq!(ss.font_faces().len(), 1);
        assert_eq!(ss.font_faces()[0].family, "MyFont");
    }

    #[test]
    fn selector_exact_match_no_false_positives() {
        let css = ".btn { color: red; } .btn-group { color: blue; }";
        let mut ss = StyleSheet::from_css(css);
        let styles = ss.get_styles(".btn");
        // Should not include .btn-group declarations
        assert_eq!(styles.len(), 1, "only one pseudo-class group expected");
        assert_eq!(styles[0].1.len(), 1, "exactly one declaration for .btn");
    }

    #[test]
    fn merge_stylesheets() {
        let mut ss1 = StyleSheet::from_css(".a { color: red; }");
        let ss2 = StyleSheet::from_css(".b { color: blue; }");
        ss1.merge(ss2);

        assert!(!ss1.get_styles(".a").is_empty());
        assert!(!ss1.get_styles(".b").is_empty());
    }

    #[test]
    fn add_css_adds_rules() {
        let mut ss = StyleSheet::from_css(".a { color: red; }");
        ss.add_css(".b { color: blue; }");
        assert!(!ss.get_styles(".b").is_empty());
    }

    #[test]
    fn remove_selector_works() {
        let mut ss = StyleSheet::from_css(".a { color: red; } .b { color: blue; }");
        let removed = ss.remove_selector(".a");
        assert!(removed);
        assert!(ss.get_styles(".a").is_empty());
        assert!(!ss.get_styles(".b").is_empty());
    }

    #[test]
    fn declarations_for_iterator() {
        let ss = StyleSheet::from_css(".btn { color: red; font-size: 16px; }");
        let count = ss.declarations_for(".btn").count();
        assert_eq!(count, 2);
    }

    #[test]
    fn end_to_end() {
        let css = r#"
            :root { --brand: #4f46e5; }
            .btn { color: var(--brand); opacity: 1; }
            .btn:hover { opacity: 0.9; }
            @keyframes fadeIn {
                from { opacity: 0; }
                to   { opacity: 1; }
            }
            @media (max-width: 768px) {
                .btn { width: 100%; }
            }
        "#;
        let mut ss = StyleSheet::from_css(css);

        let decls = ss.get_styles(".btn");
        assert!(!decls.is_empty(), "should have .btn styles");

        let media = ss.media_rules();
        assert_eq!(media.len(), 1, "one @media rule");

        let anim = ss.keyframes().get("fadeIn");
        assert!(anim.is_some(), "@keyframes fadeIn should be stored");
    }
}
