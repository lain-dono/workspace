//! Lexer for Roto scripts

use core::{ops::Range, str};
use std::ops::ControlFlow;
use unicode_ident::{is_xid_continue, is_xid_start};

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'s> {
    Ident(&'s str),

    // punctuation
    Arrow,  // `->`
    Bang,   // `!`
    Colon,  // `:`
    Comma,  // `,`
    Assign, // `=`
    Hyphen, // `-`
    Period, // `.`
    Pipe,   // `|`
    Plus,   // `+`
    Try,    // `?`
    Semi,   // `;`
    Slash,  // `/`
    Star,   // `*`

    // logic
    AmpAmp,   // `&&`
    PipePipe, // `||`

    // cmp
    Eq, // `==`
    Ne, // `!=`
    Lt, // `<`
    Le, // `<=`
    Gt, // `>`
    Ge, // `>=`

    // delimiters
    CurlyLeft,   // `{`
    CurlyRight,  // `}`
    RoundLeft,   // `(`
    RoundRight,  // `)`
    SquareLeft,  // `[`
    SquareRight, // `]`

    // Keywords
    Keyword(Keyword),

    // Literals
    Bool(bool),
    Float(&'s str),
    String(&'s str),
    Hex(&'s str),
    Integer(&'s str),

    /// An f-string start token signals to the parser that an f-string is coming up
    FStringStart,
}

impl Token<'_> {
    pub const CURLY: (Self, Self) = (Self::CurlyLeft, Self::CurlyRight);
    pub const ROUND: (Self, Self) = (Self::RoundLeft, Self::RoundRight);
    pub const SQUARE: (Self, Self) = (Self::SquareLeft, Self::SquareRight);
}

pub enum FStringToken<'s> {
    /// The final part of a string.
    ///
    /// This is the token from the current position to the end of the string.
    /// For non-f-strings, this will be the entire string.
    StringEnd(&'s str),

    /// An intermediate part of an f-string, until the next `{` token.
    StringIntermediate(&'s str),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    Accept,
    Dep,
    Else,
    Filter,
    FilterMap,
    Fn,
    If,
    Import,
    In,
    Let,
    Match,
    Not,
    Pkg,
    Record,
    Reject,
    Return,
    Std,
    Super,
    Test,
    Variant,
    While,
}

pub struct Lexer<'a> {
    input: &'a str,
    original_length: usize,
    peeked: Option<(Result<Token<'a>, ()>, Range<usize>)>,
}

impl<'a> Lexer<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<(Result<Token<'a>, ()>, Range<usize>)> {
        self.peeked.take().or_else(|| self.next_inner())
    }

    pub fn peek(&mut self) -> Option<&(Result<Token<'a>, ()>, Range<usize>)> {
        if self.peeked.is_none() {
            self.peeked = self.next_inner();
        }
        self.peeked.as_ref()
    }

    fn next_inner(&mut self) -> Option<(Result<Token<'a>, ()>, Range<usize>)> {
        match self.next_token() {
            ControlFlow::Continue(()) => {
                if self.input.is_empty() {
                    None
                } else {
                    let start = self.original_length - self.input.len();
                    Some((Err(()), start..start + 1))
                }
            }
            ControlFlow::Break((tok, span)) => Some((Ok(tok), span)),
        }
    }
}

impl<'s> Lexer<'s> {
    #[must_use]
    pub fn new(input: &'s str) -> Self {
        Self {
            input,
            original_length: input.len(),
            peeked: None,
        }
    }

    fn advance(&mut self, n: usize) -> (&'s str, Range<usize>) {
        let start = self.original_length - self.input.len();
        let (a, b) = self.input.split_at(n);
        self.input = b;
        let end = self.original_length - self.input.len();
        (a, start..end)
    }

    fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    fn next_token(&mut self) -> ControlFlow<(Token<'s>, Range<usize>)> {
        self.skip_whitespace();

        if self.is_empty() {
            return ControlFlow::Continue(());
        }

        self.two_char_punctuation()?;
        self.one_char_punctuation()?;
        self.hex_number()?;
        self.float()?;
        self.integer()?;
        self.f_string()?;
        self.string()?;
        self.keyword_or_ident()?;

        ControlFlow::Continue(())
    }

    fn skip_whitespace(&mut self) {
        loop {
            self.input = self.input.trim_start();
            if self.input.starts_with('#') {
                let n = scan(self.input, |ch| ch == '\n');
                self.advance(n);
            } else {
                return;
            }
        }
    }

    fn two_char_punctuation(&mut self) -> ControlFlow<(Token<'s>, Range<usize>)> {
        let Some(x) = self.input.as_bytes().first_chunk::<2>() else {
            return ControlFlow::Continue(());
        };

        let tok = match x {
            b"==" => Token::Eq,
            b"!=" => Token::Ne,
            b"&&" => Token::AmpAmp,
            b"||" => Token::PipePipe,
            b">=" => Token::Ge,
            b"<=" => Token::Le,
            b"->" => Token::Arrow,
            _ => return ControlFlow::Continue(()),
        };

        ControlFlow::Break((tok, self.advance(2).1))
    }

    fn one_char_punctuation(&mut self) -> ControlFlow<(Token<'s>, Range<usize>)> {
        let Some(x) = self.input.as_bytes().first() else {
            return ControlFlow::Continue(());
        };

        let tok = match x {
            b'=' => Token::Assign,
            b'|' => Token::Pipe,
            b'-' => Token::Hyphen,
            b':' => Token::Colon,
            b';' => Token::Semi,
            b',' => Token::Comma,
            b'.' => Token::Period,
            b'+' => Token::Plus,
            b'*' => Token::Star,
            b'/' => Token::Slash,
            b'!' => Token::Bang,
            b'{' => Token::CurlyLeft,
            b'}' => Token::CurlyRight,
            b'?' => Token::Try,
            b'[' => Token::SquareLeft,
            b']' => Token::SquareRight,
            b'(' => Token::RoundLeft,
            b')' => Token::RoundRight,
            b'<' => Token::Lt,
            b'>' => Token::Gt,
            _ => return ControlFlow::Continue(()),
        };

        ControlFlow::Break((tok, self.advance(1).1))
    }

    fn hex_number(&mut self) -> ControlFlow<(Token<'s>, Range<usize>)> {
        let Some(rest) = self.input.strip_prefix("0x") else {
            return ControlFlow::Continue(());
        };
        let cursor = scan(rest, not_ascii_hexdigit);
        let (tok, span) = self.advance(2 + cursor);
        ControlFlow::Break((Token::Hex(tok), span))
    }

    fn float(&mut self) -> ControlFlow<(Token<'s>, Range<usize>)> {
        let mut cursor = scan(self.input, not_ascii_digit);
        if cursor == 0 {
            return ControlFlow::Continue(());
        }

        let mut rest = &self.input[cursor..];
        if rest.starts_with('.') {
            cursor += 1;
            rest = &self.input[cursor..];

            // If we have `10..` or `10._hello` or `10.hello` we should treat this as an integer
            if let Some(ch) = rest.chars().next()
                && (is_xid_start(ch) || ch == '.' || ch == '_')
            {
                return ControlFlow::Continue(());
            }

            cursor += scan(rest, not_ascii_digit);
            rest = &self.input[cursor..];

            if rest.starts_with(['e', 'E']) {
                cursor += 1;
                rest = &self.input[cursor..];
                if rest.starts_with(['+', '-']) {
                    cursor += 1;
                    rest = &self.input[cursor..];
                }
                cursor += scan(rest, not_ascii_digit);
            }
        } else if rest.starts_with(['e', 'E']) {
            cursor += 1;
            rest = &self.input[cursor..];
            if rest.starts_with(['+', '-']) {
                cursor += 1;
                rest = &self.input[cursor..];
            }
            cursor += scan(rest, not_ascii_digit);
        } else {
            return ControlFlow::Continue(());
        }

        let (tok, span) = self.advance(cursor);
        ControlFlow::Break((Token::Float(tok), span))
    }

    fn integer(&mut self) -> ControlFlow<(Token<'s>, Range<usize>)> {
        let cursor = scan(self.input, not_ascii_digit);
        if cursor == 0 {
            ControlFlow::Continue(())
        } else {
            let (tok, span) = self.advance(cursor);
            ControlFlow::Break((Token::Integer(tok), span))
        }
    }

    fn f_string(&mut self) -> ControlFlow<(Token<'s>, Range<usize>)> {
        if self.input.strip_prefix("f\"").is_some() {
            ControlFlow::Break((Token::FStringStart, self.advance(2).1))
        } else {
            ControlFlow::Continue(())
        }
    }

    pub fn f_string_part(&mut self) -> Option<(FStringToken<'s>, Range<usize>)> {
        let mut chars = self.input.chars().enumerate();
        while let Some((i, ch)) = chars.next() {
            match ch {
                '\\' => {
                    let (_, ch) = chars.next()?;
                    if ch == 'u' || ch == 'U' {
                        let (_, c) = chars.next()?;
                        // We need a `{` after `\u` and `\U`
                        if c != '{' || chars.all(|(_, ch)| ch != '}') {
                            return None;
                        }
                    }
                }
                '{' => {
                    let (i, ch) = chars.next()?;
                    if ch != '{' {
                        // We bump to _before_ the curly
                        let (tok, span) = self.advance(i - 1);
                        return Some((FStringToken::StringIntermediate(tok), span));
                    }
                }
                '"' => {
                    // Check for the end of the string, which is an unescaped quote
                    let (tok, span) = self.advance(i);
                    self.advance(1); // Eat the `"`
                    return Some((FStringToken::StringEnd(tok), span));
                }
                _ => {}
            }
        }

        // Reached the end of the input, but were still in an f-string!
        None
    }

    fn string(&mut self) -> ControlFlow<(Token<'s>, Range<usize>)> {
        let Some(rest) = self.input.strip_prefix('"') else {
            return ControlFlow::Continue(());
        };

        let mut last_is_backslash = false;
        let end_quote = rest.find(|c| {
            let result = !last_is_backslash && c == '"';
            last_is_backslash = !last_is_backslash && c == '\\';
            result
        });

        if let Some(end_quote) = end_quote {
            let (tok, span) = self.advance(2 + end_quote);
            ControlFlow::Break((Token::String(tok), span))
        } else {
            ControlFlow::Continue(())
        }
    }

    fn keyword_or_ident(&mut self) -> ControlFlow<(Token<'s>, Range<usize>)> {
        let Some(ch) = self.input.chars().next() else {
            return ControlFlow::Continue(());
        };

        if !is_xid_start(ch) && ch != '_' {
            return ControlFlow::Continue(());
        }

        let non_ident_idx = scan(self.input, |ch| !is_xid_continue(ch));

        let (ident, span) = self.advance(non_ident_idx);

        let kw = match ident {
            "accept" => Keyword::Accept,
            "dep" => Keyword::Dep,
            "else" => Keyword::Else,
            "filter" => Keyword::Filter,
            "filtermap" => Keyword::FilterMap,
            "fn" => Keyword::Fn,
            "if" => Keyword::If,
            "import" => Keyword::Import,
            "in" => Keyword::In,
            "let" => Keyword::Let,
            "match" => Keyword::Match,
            "not" => Keyword::Not,
            "pkg" => Keyword::Pkg,
            "record" => Keyword::Record,
            "reject" => Keyword::Reject,
            "return" => Keyword::Return,
            "std" => Keyword::Std,
            "super" => Keyword::Super,
            "test" => Keyword::Test,
            "variant" => Keyword::Variant,
            "while" => Keyword::While,
            "true" => return ControlFlow::Break((Token::Bool(true), span)),
            "false" => return ControlFlow::Break((Token::Bool(false), span)),
            x => return ControlFlow::Break((Token::Ident(x), span)),
        };
        ControlFlow::Break((Token::Keyword(kw), span))
    }
}

impl core::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Ident(ident) => ident,

            // Punctuation
            Self::AmpAmp => "&&",
            Self::Le => "<=",
            Self::Ge => ">=",
            Self::Arrow => "->",
            Self::Bang => "!",
            Self::Ne => "!=",
            Self::Colon => ":",
            Self::Comma => ",",
            Self::Assign => "=",
            Self::Eq => "==",
            Self::Hyphen => "-",
            Self::Period => ".",
            Self::Pipe => "|",
            Self::PipePipe => "||",
            Self::Plus => "+",
            Self::Try => "?",
            Self::Semi => ";",
            Self::Slash => "/",
            Self::Star => "*",

            // Delimiters
            Self::Lt => "<",
            Self::Gt => ">",
            Self::CurlyLeft => "{",
            Self::CurlyRight => "}",
            Self::RoundLeft => "(",
            Self::RoundRight => ")",
            Self::SquareLeft => "[",
            Self::SquareRight => "]",

            // Keywords
            Self::Keyword(Keyword::Accept) => "accept",
            Self::Keyword(Keyword::Dep) => "dep",
            Self::Keyword(Keyword::Else) => "else",
            Self::Keyword(Keyword::Filter) => "filter",
            Self::Keyword(Keyword::FilterMap) => "filtermap",
            Self::Keyword(Keyword::Fn) => "fn",
            Self::Keyword(Keyword::If) => "if",
            Self::Keyword(Keyword::Import) => "import",
            Self::Keyword(Keyword::In) => "in",
            Self::Keyword(Keyword::Let) => "let",
            Self::Keyword(Keyword::Match) => "match",
            Self::Keyword(Keyword::Not) => "not",
            Self::Keyword(Keyword::Pkg) => "pkg",
            Self::Keyword(Keyword::Record) => "record",
            Self::Keyword(Keyword::Reject) => "reject",
            Self::Keyword(Keyword::Return) => "return",
            Self::Keyword(Keyword::Std) => "std",
            Self::Keyword(Keyword::Super) => "super",
            Self::Keyword(Keyword::Test) => "test",
            Self::Keyword(Keyword::Variant) => "variant",
            Self::Keyword(Keyword::While) => "while",

            // Literals
            Self::String(s) => s,
            Self::Integer(i) => i,
            Self::Float(f) => f,
            Self::Hex(h) => h,
            Self::Bool(true) => "true",
            Self::Bool(false) => "false",

            Self::FStringStart => "f\"",
        })
    }
}

#[inline]
fn not_ascii_digit(ch: char) -> bool {
    !ch.is_ascii_digit()
}

#[inline]
fn not_ascii_hexdigit(ch: char) -> bool {
    !ch.is_ascii_hexdigit()
}

#[inline]
fn scan(s: &str, pat: fn(ch: char) -> bool) -> usize {
    s.find(pat).unwrap_or(s.len())
}
