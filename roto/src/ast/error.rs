use super::meta::Span;
use rustc_literal_escaper::{EscapeError, Mode};

#[derive(Clone, Debug, thiserror::Error)]
#[error("{kind}")]
pub struct ParseError {
    pub location: Span,
    pub kind: ParseErrorKind,
    pub note: Option<String>,
}

impl ParseError {
    pub(super) fn expected(
        expected: impl core::fmt::Display,
        got: impl core::fmt::Display,
        span: Span,
    ) -> Self {
        Self {
            kind: ParseErrorKind::Expected {
                expected: expected.to_string(),
                got: got.to_string(),
            },
            location: span,
            note: None,
        }
    }

    pub(super) fn invalid_literal(
        description: impl core::fmt::Display,
        token: impl core::fmt::Display,
        inner: impl core::fmt::Display,
        span: Span,
    ) -> Self {
        Self {
            kind: ParseErrorKind::InvalidLiteral {
                description: description.to_string(),
                token: token.to_string(),
                inner_error: inner.to_string(),
            },
            location: span,
            note: None,
        }
    }

    pub(super) fn escape(escape_error: &EscapeError, span: Span) -> Self {
        let description = escape_error_to_msg(escape_error, Mode::Str);
        Self {
            kind: ParseErrorKind::Custom {
                description: description.clone(),
                label: description,
            },
            location: span,
            note: None,
        }
    }

    pub(super) fn custom(
        description: impl core::fmt::Display,
        label: impl core::fmt::Display,
        span: Span,
    ) -> Self {
        Self {
            kind: ParseErrorKind::Custom {
                description: description.to_string(),
                label: label.to_string(),
            },
            location: span,
            note: None,
        }
    }

    pub(super) fn with_note(self, note: impl Into<String>) -> Self {
        Self {
            note: Some(note.into()),
            ..self
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum ParseErrorKind {
    #[error("unexpected end of input")]
    EndOfInput,
    #[error("failed to parse entire input")]
    FailedToParseEntireInput,
    #[error("invalid token")]
    InvalidToken,
    #[error("expected {expected} but got '{got}'")]
    Expected { expected: String, got: String },
    #[error("found an invalid {description} literal '{token}': {inner_error}")]
    InvalidLiteral {
        description: String,
        token: String,
        inner_error: String,
    },
    #[error("{description}")]
    Custom { description: String, label: String },
}

impl ParseErrorKind {
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::EndOfInput => "reached end of input".into(),
            Self::FailedToParseEntireInput => "parser got stuck here".into(),
            Self::InvalidToken => "invalid token".into(),
            Self::Expected { expected, .. } => format!("expected {expected}"),
            Self::InvalidLiteral { description, .. } => format!("invalid {description}"),
            Self::Custom { label, .. } => label.clone(),
        }
    }
}

// Adapted from Rust Analyzer at src/tools/rust-analyzer/crates/parser/src/lexed_str.rs
fn escape_error_to_msg(error: &EscapeError, mode: Mode) -> String {
    #[allow(clippy::match_same_arms)]
    match error {
        EscapeError::ZeroChars => "empty character literal",
        EscapeError::MoreThanOneChar => "character literal may only contain one codepoint",
        EscapeError::LoneSlash => "",
        EscapeError::InvalidEscape if mode == Mode::Byte || mode == Mode::ByteStr => {
            "unknown byte escape"
        }
        EscapeError::InvalidEscape => "unknown character escape",
        EscapeError::BareCarriageReturn => "bare carriage return found without '\\n'",
        EscapeError::BareCarriageReturnInRawString => {
            "bare carriage return found in raw string without '\\n'"
        }
        EscapeError::EscapeOnlyChar if mode == Mode::Byte => "byte constant must be escaped",
        EscapeError::EscapeOnlyChar => "character constant must be escaped",
        EscapeError::TooShortHexEscape => "numeric character escape is too short",
        EscapeError::InvalidCharInHexEscape => "invalid character in numeric character escape",
        EscapeError::OutOfRangeHexEscape => "out of range hex escape",
        EscapeError::NoBraceInUnicodeEscape => "incorrect unicode escape sequence",
        EscapeError::InvalidCharInUnicodeEscape => "invalid character in unicode escape",
        EscapeError::EmptyUnicodeEscape => "empty unicode escape",
        EscapeError::UnclosedUnicodeEscape => "unterminated unicode escape",
        EscapeError::LeadingUnderscoreUnicodeEscape => "invalid start of unicode escape",
        EscapeError::OverlongUnicodeEscape => "overlong unicode escape",
        EscapeError::LoneSurrogateUnicodeEscape => "invalid unicode character escape",
        EscapeError::OutOfRangeUnicodeEscape => "invalid unicode character escape",
        EscapeError::UnicodeEscapeInByte => "unicode escape in byte string",
        EscapeError::NonAsciiCharInByte if mode == Mode::Byte => {
            "non-ASCII character in byte literal"
        }
        EscapeError::NonAsciiCharInByte if mode == Mode::ByteStr => {
            "non-ASCII character in byte string literal"
        }
        EscapeError::NonAsciiCharInByte => "non-ASCII character in raw byte string literal",
        EscapeError::NulInCStr => "null character in C string literal",
        EscapeError::UnskippedWhitespaceWarning => "",
        EscapeError::MultipleSkippedLinesWarning => "",
    }
    .into()
}
