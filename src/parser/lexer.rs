#[derive(Clone, Copy)]
pub enum Token<'a> {
    Selector { value: &'a str, line: usize, col: usize },
    Property { value: &'a str, line: usize, col: usize },
    Value { value: &'a str, line: usize, col: usize },
    BlockOpen { line: usize, col: usize },
    BlockClose { line: usize, col: usize },
    Colon { line: usize, col: usize },
    Semicolon { line: usize, col: usize },
    /// At-rule token: e.g. `@media (max-width: 600px)` before the `{`
    /// `keyword` = word after `@`; `prelude` = text between keyword and `{`/`;`
    AtRule { keyword: &'a str, prelude: &'a str, line: usize, col: usize },
    EOF,
}

impl std::fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Selector { value, .. } => write!(f, "Selector({value})"),
            Token::Property { value, .. } => write!(f, "Property({value})"),
            Token::Value { value, .. } => write!(f, "Value({value})"),
            Token::BlockOpen { .. } => write!(f, "BlockOpen"),
            Token::BlockClose { .. } => write!(f, "BlockClose"),
            Token::Colon { .. } => write!(f, "Colon"),
            Token::Semicolon { .. } => write!(f, "Semicolon"),
            Token::AtRule { keyword, prelude, .. } => write!(f, "AtRule(@{keyword} {prelude})"),
            Token::EOF => write!(f, "EOF"),
        }
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub const fn new(input: &'a str) -> Self {
        Lexer { input, position: 0, line: 1, col: 0 }
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input[self.position..].chars().next()?;
        self.position += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn advance_until_comment_end(&mut self) {
        loop {
            let Some(c) = self.advance() else { break };
            if c == '*' {
                if self.peek_char() == Some('/') {
                    self.advance(); // consume the closing '/'
                    break;
                }
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(c) if c.is_whitespace()) {
            self.advance();
        }
    }

    /// Scan `@keyword prelude` starting at `at_start` (right after `@`).
    /// Stops at the first `{` or `;` at depth 0 without consuming it.
    /// Returns `(keyword, prelude)` as slices into the original input.
    fn scan_at_rule(&mut self, at_start: usize) -> (&'a str, &'a str) {
        // Collect keyword: chars until whitespace or `{`
        let kw_start = at_start;
        while matches!(self.peek_char(), Some(c) if !c.is_whitespace() && c != '{' && c != ';') {
            self.advance();
        }
        let keyword = self.input[kw_start..self.position].trim();

        // Collect prelude: chars until `{` or `;` at depth 0
        let prelude_start = self.position;
        let mut depth: usize = 0;
        loop {
            match self.peek_char() {
                None => break,
                Some('{') if depth == 0 => break,
                Some(';') if depth == 0 => break,
                Some('{') => { depth += 1; self.advance(); }
                Some('}') => { depth = depth.saturating_sub(1); self.advance(); }
                _ => { self.advance(); }
            }
        }
        let prelude = self.input[prelude_start..self.position].trim();

        (keyword, prelude)
    }

    #[must_use]
    pub fn tokens(&mut self) -> Vec<Token<'a>> {
        let mut tokens = Vec::with_capacity(1024 * 16);

        // Each entry on the stack corresponds to an open `{`.
        // `true`  = declaration block (inside a selector rule: property: value)
        // `false` = container block (inside an at-rule: contains selectors+rules)
        let mut block_stack: Vec<bool> = Vec::new();

        // Will the NEXT `{` open a declaration block?
        // Set to true after emitting a Selector, set to false after an AtRule.
        let mut next_block_is_decl = false;

        loop {
            let Some(c) = self.advance() else {
                tokens.push(Token::EOF);
                break;
            };

            let in_decl = block_stack.last().copied().unwrap_or(false);

            match c {
                '{' => {
                    tokens.push(Token::BlockOpen { line: self.line, col: self.col });
                    block_stack.push(next_block_is_decl);
                    next_block_is_decl = false;
                }
                '}' => {
                    tokens.push(Token::BlockClose { line: self.line, col: self.col });
                    block_stack.pop();
                }
                ':' if in_decl => {
                    tokens.push(Token::Colon { line: self.line, col: self.col });
                }
                ';' => {
                    tokens.push(Token::Semicolon { line: self.line, col: self.col });
                }
                '/' if self.peek_char() == Some('*') => {
                    self.advance(); // consume '*'
                    self.advance_until_comment_end();
                }
                '@' if !in_decl => {
                    let line = self.line;
                    let col = self.col;
                    let at_start = self.position;
                    let (keyword, prelude) = self.scan_at_rule(at_start);
                    tokens.push(Token::AtRule { keyword, prelude, line, col });
                    // @font-face contains property:value pairs directly (declaration block).
                    // All other at-rules (@media, @keyframes, @supports, …) use container blocks.
                    next_block_is_decl = keyword == "font-face";
                }
                c if c.is_whitespace() => {
                    self.skip_whitespace();
                }
                _ => {
                    let start_pos = self.position - c.len_utf8();
                    if in_decl {
                        // Inside a rule block: collect property name or value
                        while matches!(self.peek_char(), Some(p) if p != ':' && p != ';' && p != '}') {
                            self.advance();
                        }
                        let value = self.input[start_pos..self.position].trim_end();
                        match self.peek_char() {
                            Some(':') => tokens.push(Token::Property {
                                line: self.line, col: self.col, value,
                            }),
                            Some(';') | Some('}') => tokens.push(Token::Value {
                                line: self.line, col: self.col, value,
                            }),
                            _ => {}
                        }
                    } else {
                        // Selector context: collect until `{`
                        while matches!(self.peek_char(), Some(p) if p != '{') {
                            self.advance();
                        }
                        let raw = self.input[start_pos..self.position].trim_end();
                        tokens.push(Token::Selector { line: self.line, col: self.col, value: raw });
                        next_block_is_decl = true; // next `{` opens a declaration block
                    }
                }
            }
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_names(tokens: &[Token<'_>]) -> Vec<String> {
        tokens.iter().map(|t| format!("{t:?}")).collect()
    }

    #[test]
    fn simple_rule_tokens() {
        let mut lexer = Lexer::new("button { color: red; }");
        let tokens = lexer.tokens();
        let names = token_names(&tokens);
        assert!(names.iter().any(|n| n.starts_with("Selector(button")), "{names:?}");
        assert!(names.iter().any(|n| n.starts_with("Property(color")), "{names:?}");
        assert!(names.iter().any(|n| n.starts_with("Value(red")), "{names:?}");
    }

    #[test]
    fn at_rule_media() {
        let css = "@media (max-width: 600px) { .foo { color: red; } }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokens();
        let names = token_names(&tokens);
        assert!(
            names.iter().any(|n| n.starts_with("AtRule(@media")),
            "Expected AtRule token, got: {names:?}"
        );
        assert!(names.iter().any(|n| n.starts_with("Selector(.foo")), "{names:?}");
        assert!(names.iter().any(|n| n.starts_with("Property(color")), "{names:?}");
        assert!(names.iter().any(|n| n.starts_with("Value(red")), "{names:?}");
    }

    #[test]
    fn at_rule_keyframes() {
        let css = "@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokens();
        let names = token_names(&tokens);
        assert!(
            names.iter().any(|n| n.starts_with("AtRule(@keyframes")),
            "{names:?}"
        );
        assert!(names.iter().any(|n| n.starts_with("Selector(from")), "{names:?}");
        assert!(names.iter().any(|n| n.starts_with("Property(opacity")), "{names:?}");
    }

    #[test]
    fn comment_consumed_cleanly() {
        let css = "/* hello world */ .a { color: red; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokens();
        let names = token_names(&tokens);
        assert!(names.iter().any(|n| n.starts_with("Selector(.a")), "{names:?}");
        assert!(!names.iter().any(|n| n.contains("hello")), "{names:?}");
    }

    #[test]
    fn block_depth_counter() {
        let css = "@media screen { .x { color: blue; } }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokens();
        let names = token_names(&tokens);
        assert!(names.iter().any(|n| n.starts_with("AtRule(@media")), "{names:?}");
        assert!(names.iter().any(|n| n.starts_with("Selector(.x")), "{names:?}");
        assert!(names.iter().any(|n| n.starts_with("Property(color")), "{names:?}");
        assert!(names.iter().any(|n| n.starts_with("Value(blue")), "{names:?}");
    }

    #[test]
    fn col_tracking() {
        let css = ".btn { color: red; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokens();
        for token in &tokens {
            match token {
                Token::Selector { col, .. }
                | Token::Property { col, .. }
                | Token::Value { col, .. }
                | Token::BlockOpen { col, .. }
                | Token::BlockClose { col, .. }
                | Token::Colon { col, .. }
                | Token::Semicolon { col, .. }
                | Token::AtRule { col, .. } => { let _ = col; }
                Token::EOF => {}
            }
        }
    }

    #[test]
    fn css_variables_in_root() {
        let css = ":root { --color: blue; } .btn { color: var(--color); }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokens();
        let names = token_names(&tokens);
        assert!(names.iter().any(|n| n.contains(":root")), "{names:?}");
        assert!(names.iter().any(|n| n.contains("--color")), "{names:?}");
    }
}
