use chumsky::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub enum LitKind {
    Bool,
    Int,
    Float,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Literal {
    pub kind: LitKind,
    pub symbol: String,
    pub suffix: String,
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.symbol, self.suffix)
    }
}

impl Literal {
    pub fn boolean(value: impl ToString) -> Self {
        Self {
            kind: LitKind::Bool,
            symbol: value.to_string(),
            suffix: String::new(),
        }
    }

    pub fn int(symbol: impl ToString, suffix: impl ToString) -> Self {
        Self {
            kind: LitKind::Int,
            symbol: symbol.to_string(),
            suffix: suffix.to_string(),
        }
    }

    pub fn float(symbol: impl ToString, suffix: impl ToString) -> Self {
        Self {
            kind: LitKind::Float,
            symbol: symbol.to_string(),
            suffix: suffix.to_string(),
        }
    }

    pub fn parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        choice((
            find_keyword().try_map(move |symbol, span| match symbol.as_ref() {
                "true" | "false" => Ok(Literal::boolean(symbol)),
                _ => Err(Simple::expected_input_found(span, None, None)),
            }),
            float_symbol()
                .then(float_suffix().or_not().map(Option::unwrap_or_default))
                .map(|(symbol, suffix)| Literal::float(symbol, suffix)),
            int_symbol()
                .then(int_suffix().or_not().map(Option::unwrap_or_default))
                .map(|(symbol, suffix)| Literal::int(symbol, suffix)),
        ))
    }
}

fn find_keyword() -> impl Parser<char, String, Error = Simple<char>> {
    filter(char::is_ascii_alphabetic)
        .map(Some)
        .chain::<char, Vec<_>, _>(filter(char::is_ascii_alphanumeric).repeated())
        .collect::<String>()
}

fn int_symbol() -> impl Parser<char, String, Error = Simple<char>> {
    let dec = filter(|c: &char| matches!(c, '1'..='9'))
        .chain(filter(|c: &char| matches!(c, '_' | '0'..='9')).repeated());

    let bin = just('0')
        .chain(just('b'))
        .chain(just('_').repeated())
        .chain(filter(|c| matches!(c, '0'..='1')))
        .chain(filter(|c| matches!(c, '0'..='1' | '_')).repeated());

    let oct = just('0')
        .chain(just('o'))
        .chain(just('_').repeated())
        .chain(filter(|c| matches!(c, '0'..='7')))
        .chain(filter(|c| matches!(c, '0'..='7' | '_')).repeated());

    let hex = just('0')
        .chain(just('x'))
        .chain(just('_').repeated())
        .chain(filter(|c| matches!(c, '0'..='9' | 'A'..='F' | 'a'..='f')))
        .chain(filter(|c| matches!(c, '0'..='9' | 'A'..='F' | 'a'..='f' | '_')).repeated());

    let zero = just('0').map(|c| core::iter::once(c).collect());
    choice((dec, bin, oct, hex, zero)).collect()
}

fn int_suffix() -> impl Parser<char, String, Error = Simple<char>> {
    find_keyword().try_map(move |s, span| match s.as_ref() {
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => Ok(s),
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => Ok(s),
        _ => Err(Simple::expected_input_found(span, None, None)),
    })
}

fn float_symbol() -> impl Parser<char, String, Error = Simple<char>> {
    let dec = filter(|c: &char| matches!(c, '0'..='9'))
        .chain(filter(|c: &char| matches!(c, '0'..='9' | '_')).repeated());

    let dec_ext = just('_').repeated().chain(dec);

    let exponent = one_of(['e', 'E']).chain(one_of(['+', '-']).or_not());

    let exp = dec
        .chain(just('.').chain(dec_ext).or_not().flatten())
        .chain(exponent)
        .chain(dec_ext);

    let base = dec.chain(just('.')).chain(dec_ext);
    let simple = dec.chain(just('.'));

    choice((exp, base, simple)).collect()
}

fn float_suffix() -> impl Parser<char, String, Error = Simple<char>> {
    find_keyword().try_map(move |s: String, span| match s.as_ref() {
        "f32" | "f64" => Ok(s),
        _ => Err(Simple::expected_input_found(span, None, None)),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn integer() {
        let lit = Literal::parser();

        assert_eq!(lit.parse("0"), Ok(Literal::int("0", "")));
        assert_eq!(lit.parse("1"), Ok(Literal::int("1", "")));
        assert_eq!(
            lit.parse("0x1234567890abcdefABCDEF"),
            Ok(Literal::int("0x1234567890abcdefABCDEF", ""))
        );
        assert_eq!(lit.parse("1234567890"), Ok(Literal::int("1234567890", "")));
        assert_eq!(lit.parse("0o1234567"), Ok(Literal::int("0o1234567", "")));
        assert_eq!(lit.parse("0b1010"), Ok(Literal::int("0b1010", "")));

        assert_eq!(lit.parse("0b__10_10"), Ok(Literal::int("0b__10_10", "")));
        assert_eq!(lit.parse("0b_10_10_"), Ok(Literal::int("0b_10_10_", "")));
        assert_eq!(lit.parse("0b10_10__"), Ok(Literal::int("0b10_10__", "")));

        assert_eq!(lit.parse("0o__10_10"), Ok(Literal::int("0o__10_10", "")));
        assert_eq!(lit.parse("0o_10_10_"), Ok(Literal::int("0o_10_10_", "")));
        assert_eq!(lit.parse("0o10_10__"), Ok(Literal::int("0o10_10__", "")));

        assert_eq!(lit.parse("0x__10_10"), Ok(Literal::int("0x__10_10", "")));
        assert_eq!(lit.parse("0x_10_10_"), Ok(Literal::int("0x_10_10_", "")));
        assert_eq!(lit.parse("0x10_10__"), Ok(Literal::int("0x10_10__", "")));

        assert_eq!(lit.parse("5usize"), Ok(Literal::int("5", "usize")));
        assert_eq!(lit.parse("5i8"), Ok(Literal::int("5", "i8")));
    }

    #[test]
    fn float() {
        let lit = Literal::parser();

        assert_eq!(lit.parse("123.0"), Ok(Literal::float("123.0", "")));
        assert_eq!(lit.parse("123."), Ok(Literal::float("123.", "")));
        assert_eq!(lit.parse("12E+99_"), Ok(Literal::float("12E+99_", "")));
        assert_eq!(lit.parse("12.3E+99_"), Ok(Literal::float("12.3E+99_", "")));
    }

    #[test]
    fn boolean() {
        let lit = Literal::parser();

        assert_eq!(lit.parse("true"), Ok(Literal::boolean(true)));
        assert_eq!(lit.parse("false"), Ok(Literal::boolean(false)));
    }
}
