mod animation;
mod parser;
mod style;
mod unit;

pub mod error;

pub use animation::*;
pub use declaration::Declaration;
pub use error::ParseError;
pub use parser::PseudoClass;
pub use style::StyleSheet;

pub(crate) use parser::*;
pub use unit::*;

use lexer::Token;

/// Parses a CSS string into a vector of rules.
///
/// # Parameters
/// - `input`: The CSS input string.
///
/// # Returns
/// - `Ok(Vec<Rule>)` if the input is successfully parsed.
/// - `Err(Vec<SyntaxError>)` if there are syntax errors in the input.
///
/// # Example
/// ```
/// let css = ".example { color: red; }";
/// let rules = cssengine::css_to_rules(css).expect("Failed to parse CSS");
/// ```
#[must_use]
pub fn css_to_rules(input: &str) -> Result<Vec<Rule<'_>>, Vec<SyntaxError>> {
    let tokens = Lexer::new(input).tokens();
    analyze_tokens(&tokens, input)?;
    let rules = Parser::new(tokens).parse();
    Ok(replace_vars(rules))
}

pub fn analyze(input: &str) -> Result<(), Vec<SyntaxError>> {
    let tokens = Lexer::new(input).tokens();
    let res = parser::analyze_tokens(&tokens, input);

    if !res.is_empty() {
        return Err(res);
    }

    Ok(())
}

pub fn analyze_tokens<'a>(
    tokens: &[Token<'a>],
    input: &'a str,
) -> Result<(), Vec<SyntaxError<'a>>> {
    let res = parser::analyze_tokens(&tokens, input);

    if !res.is_empty() {
        return Err(res);
    }

    Ok(())
}
