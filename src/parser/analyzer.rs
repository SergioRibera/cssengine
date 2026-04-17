use super::lexer::Token;

#[derive(Debug)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug)]
pub struct SyntaxError<'a> {
    pub span: Span,
    pub error: &'static str,
    pub value: &'a str,
    pub hint: Option<&'static str>,
}

impl std::fmt::Display for SyntaxError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error: {}\n {}:{} | {}",
            self.error, self.span.line, self.span.col, self.value
        )?;
        if let Some(hint) = self.hint {
            write!(f, "\n  hint: {hint}")?;
        }
        Ok(())
    }
}

fn eof_error(error: &'static str) -> SyntaxError<'static> {
    SyntaxError {
        span: Span { line: 0, col: 0 },
        error,
        value: "<EOF>",
        hint: None,
    }
}

/// Validates the token stream for basic syntax correctness.
/// Returns a (possibly empty) list of syntax errors found.
/// On error, attempts to recover and continue rather than stopping.
#[must_use]
pub fn analyze_tokens<'a>(tokens: &[Token<'a>], _input: &'a str) -> Vec<SyntaxError<'a>> {
    let mut errors: Vec<SyntaxError> = Vec::new();
    let mut iter = tokens.iter().peekable();

    loop {
        let Some(token) = iter.next() else {
            break;
        };

        match token {
            Token::Selector { line, col, value } => {
                match iter.peek() {
                    None => {
                        errors.push(SyntaxError {
                            span: Span { line: *line, col: *col },
                            error: "Unexpected EOF after selector",
                            value,
                            hint: None,
                        });
                    }
                    Some(next) if !matches!(next, Token::BlockOpen { .. } | Token::Selector { .. }) => {
                        errors.push(SyntaxError {
                            span: Span { line: *line, col: *col },
                            error: "Expected { after selector",
                            value,
                            hint: None,
                        });
                    }
                    _ => {}
                }
            }
            Token::Property { line, col, value } => {
                match iter.peek() {
                    None => {
                        errors.push(eof_error("Unexpected EOF after property"));
                    }
                    Some(next) if !matches!(next, Token::Colon { .. }) => {
                        errors.push(SyntaxError {
                            span: Span { line: *line, col: *col },
                            error: "Expected : after property name",
                            value,
                            hint: None,
                        });
                    }
                    _ => {}
                }
            }
            Token::Value { line, col, value } => {
                match iter.peek() {
                    None => {
                        errors.push(eof_error("Unexpected EOF after value"));
                    }
                    Some(next) if !matches!(next, Token::Semicolon { .. } | Token::BlockClose { .. }) => {
                        errors.push(SyntaxError {
                            span: Span { line: *line, col: *col },
                            error: "Expected ; after value",
                            value,
                            hint: None,
                        });
                    }
                    _ => {}
                }
            }
            Token::Semicolon { line, col } => {
                match iter.peek() {
                    None => {
                        errors.push(eof_error("Unexpected EOF after ;"));
                    }
                    Some(next) if !matches!(next, Token::BlockClose { .. } | Token::Property { .. }) => {
                        errors.push(SyntaxError {
                            span: Span { line: *line, col: *col },
                            error: "Expected property or } after ;",
                            value: ";",
                            hint: None,
                        });
                    }
                    _ => {}
                }
            }
            Token::Colon { line, col } => {
                match iter.peek() {
                    None => {
                        errors.push(eof_error("Unexpected EOF after :"));
                    }
                    Some(next) if !matches!(next, Token::Value { .. }) => {
                        errors.push(SyntaxError {
                            span: Span { line: *line, col: *col },
                            error: "Expected value after :",
                            value: ":",
                            hint: None,
                        });
                    }
                    _ => {}
                }
            }
            // AtRule tokens are valid in any position at depth 0 — no validation needed here
            Token::AtRule { .. } | Token::BlockOpen { .. } | Token::BlockClose { .. } | Token::EOF => {}
        }
    }

    errors
}
