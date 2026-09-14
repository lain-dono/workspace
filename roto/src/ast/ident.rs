use symbol_table::GlobalSymbol;

/// An identifier is the name of variables or other things.
///
/// It is a word composed of a leading alphabetic Unicode character, followed
/// by alphanumeric Unicode characters or underscore or hyphen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident(GlobalSymbol);

impl core::fmt::Display for Ident {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl Ident {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.0.as_str()
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for Ident {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<&String> for Ident {
    fn from(value: &String) -> Self {
        Self(value.into())
    }
}

impl From<String> for Ident {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}
