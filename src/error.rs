#[derive(Debug)]
pub struct ParseError<'a> {
    pub error: &'static str,
    pub value: &'a str,
}

impl<'a> ParseError<'a> {
    pub const fn new(error: &'static str, value: &'a str) -> Self {
        Self { error, value }
    }
}

impl core::fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error, self.value)
    }
}
