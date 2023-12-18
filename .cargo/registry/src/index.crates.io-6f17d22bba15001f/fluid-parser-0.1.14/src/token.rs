#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    Unknown,
    Word,
    OpenBrace,
    CloseBrace,
    Eof,
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Unknown
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Token<'a> {
    pub typ: TokenType,
    pub word: &'a str,
    pub start: usize,
    pub end: usize,
}
