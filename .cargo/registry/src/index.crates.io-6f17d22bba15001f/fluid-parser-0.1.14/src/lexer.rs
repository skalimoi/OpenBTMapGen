use crate::token::{Token, TokenType};

pub struct Lexer<'a> {
    pub s: &'a str,
    size: usize,
    cursor: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &str) -> Lexer {
        let size = source.len();
        let mut cursor = 0;
        if source.starts_with("# ") {
            while cursor < size && source.as_bytes()[cursor] != b'\n' {
                cursor += 1;
            }
        }
        Lexer {
            s: source,
            size,
            cursor,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Token<'a> {
        let mut t = Token::default();
        if self.cursor == self.size {
            t.typ = TokenType::Eof;
            t.start = self.size;
            t.end = self.size;
            return t;
        }
        while self.cursor < self.size
            && (self.s.as_bytes()[self.cursor] == b' '
                || self.s.as_bytes()[self.cursor] == b'\n'
                || self.s.as_bytes()[self.cursor] == b'\r'
                || self.s.as_bytes()[self.cursor] == b'\t')
        {
            self.cursor += 1;
        }
        if self.cursor == self.size {
            t.typ = TokenType::Eof;
            t.start = self.size;
            t.end = self.size;
            return t;
        }
        match self.s.as_bytes()[self.cursor] {
            b'{' => {
                t.typ = TokenType::OpenBrace;
                t.start = self.cursor;
                t.end = self.cursor + 1;
                self.cursor += 1;
                t
            }
            b'}' => {
                t.typ = TokenType::CloseBrace;
                t.start = self.cursor;
                t.end = self.cursor + 1;
                self.cursor += 1;
                t
            }
            _ => {
                t.typ = TokenType::Word;

                t.start = self.cursor;
                while self.cursor < self.size
                    && self.s.as_bytes()[self.cursor] != b' '
                    && self.s.as_bytes()[self.cursor] != b'\n'
                    && self.s.as_bytes()[self.cursor] != b'\r'
                    && self.s.as_bytes()[self.cursor] != b'\t'
                    && self.s.as_bytes()[self.cursor] != b'}'
                {
                    self.cursor += 1;
                }
                t.end = self.cursor;
                t.word = &self.s[t.start..t.end];
                t
            }
        }
    }
}
