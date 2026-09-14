use chumsky::input::{SliceInput, StrInput};
use chumsky::prelude::*;

pub type SpannedToken<'src> = (Token<'src>, SimpleSpan);

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Token<'src> {
    Newline,
    Ident(&'src str),
    Number(&'src str),

    AddAssign, // '+='
    SubAssign, // '-='
    DivAssign, // '/='
    MulAssign, // '*='
    RemAssign, // '%='
    AndAssign, // '&='
    EorAssign, // '^='
    IorAssign, // '|='
    ShlAssign, // '<<='
    ShrAssign, // '>>='

    Add, // '+'
    Sub, // '-'
    Div, // '/'
    Mul, // '*'
    Rem, // '%'
    And, // '&'
    Eor, // '^'
    Ior, // '|'
    Shl, // '<<'
    Shr, // '>>'

    Assign, // '='
    Not,    // '!'
    Eq,     // '=='
    Ne,     // '!='
    Lt,     // '<'
    Le,     // '<='
    Gt,     // '>'
    Ge,     // '>='

    Lparen, // '('
    Lbrack, // '['
    Lbrace, // '{'

    Rparen, // ')'
    Rbrack, // ']'
    Rbrace, // '}'

    Comma, // ','
    Dot,   // '.'
    Semi,  // ';'
    Colon, // ':'
    Sep,   // '::'

    Fn,       // fn
    If,       // if
    For,      // for
    Let,      // let
    Else,     // else
    Return,   // return
    Break,    // break
    Continue, // continue

    Type,   // struct
    Struct, // struct
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Newline => write!(f, "\\n"),
            Self::Ident(ident) => write!(f, "{ident}"),
            Self::Number(number) => write!(f, "{number}"),

            Self::AddAssign => write!(f, "+="),
            Self::SubAssign => write!(f, "-="),
            Self::DivAssign => write!(f, "/="),
            Self::MulAssign => write!(f, "*="),
            Self::RemAssign => write!(f, "%="),
            Self::AndAssign => write!(f, "&="),
            Self::EorAssign => write!(f, "^="),
            Self::IorAssign => write!(f, "|="),
            Self::ShlAssign => write!(f, "<<="),
            Self::ShrAssign => write!(f, ">>="),

            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Div => write!(f, "/"),
            Self::Mul => write!(f, "*"),
            Self::Rem => write!(f, "%"),
            Self::And => write!(f, "&"),
            Self::Eor => write!(f, "^"),
            Self::Ior => write!(f, "|"),
            Self::Shl => write!(f, "<<"),
            Self::Shr => write!(f, ">>"),

            Self::Assign => write!(f, "="),
            Self::Not => write!(f, "!"),
            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::Ge => write!(f, ">="),

            Self::Lparen => write!(f, "("),
            Self::Lbrack => write!(f, "["),
            Self::Lbrace => write!(f, "{{"),
            Self::Comma => write!(f, ","),
            Self::Dot => write!(f, "."),
            Self::Rparen => write!(f, ")"),
            Self::Rbrack => write!(f, "]"),
            Self::Rbrace => write!(f, "}}"),
            Self::Semi => write!(f, ";"),
            Self::Colon => write!(f, ":"),
            Self::Sep => write!(f, "::"),

            Self::Fn => write!(f, "fn"),
            Self::If => write!(f, "if"),
            Self::For => write!(f, "for"),
            Self::Let => write!(f, "let"),
            Self::Else => write!(f, "else"),
            Self::Return => write!(f, "return"),
            Self::Break => write!(f, "break"),
            Self::Continue => write!(f, "continue"),

            Self::Type => write!(f, "type"),
            Self::Struct => write!(f, "struct"),
        }
    }
}

impl<'src> Token<'src> {
    pub fn scan(
        input: &'src str,
    ) -> Result<(Vec<SpannedToken<'src>>, SimpleSpan), Vec<Rich<'src, char>>> {
        let tokens = Token::scanner().parse(input).into_result()?;
        Ok((tokens, SimpleSpan::from(input.len()..input.len())))
    }

    pub fn scanner<I>()
    -> impl Parser<'src, I, Vec<(Token<'src>, I::Span)>, extra::Err<Rich<'src, char, SimpleSpan>>>
    where
        I: SliceInput<'src, Token = char, Span = SimpleSpan, Slice = &'src str>,
        I: StrInput<'src, Token = char, Span = SimpleSpan>,
        <I as SliceInput<'src>>::Slice: AsRef<[u8]>,
    {
        let padded = choice((
            text::keyword("fn").to(Token::Fn),
            text::keyword("let").to(Token::Let),
            text::keyword("else").to(Token::Else),
            text::keyword("if").to(Token::If),
            text::keyword("for").to(Token::For),
            text::keyword("type").to(Token::Type),
            text::keyword("struct").to(Token::Struct),
            just(',').to(Token::Comma),
            just('.').to(Token::Dot),
            just(';').to(Token::Semi),
            just("::").to(Token::Colon),
            just(':').to(Token::Colon),
            // ops assign
            choice((
                just("+=").to(Token::AddAssign),
                just("-=").to(Token::SubAssign),
                just("/=").to(Token::DivAssign),
                just("*=").to(Token::MulAssign),
                just("%=").to(Token::RemAssign),
                just("&=").to(Token::AndAssign),
                just("^=").to(Token::EorAssign),
                just("|=").to(Token::IorAssign),
                just("<<=").to(Token::ShlAssign),
                just(">>=").to(Token::ShrAssign),
            )),
            // ops
            choice((
                just('+').to(Token::Add),
                just('-').to(Token::Sub),
                just('/').to(Token::Div),
                just('*').to(Token::Mul),
                just('%').to(Token::Rem),
                just('&').to(Token::And),
                just('^').to(Token::Eor),
                just('|').to(Token::Ior),
                just("<<").to(Token::Shl),
                just(">>").to(Token::Shr),
            )),
            // cmp
            choice((
                just("<=").to(Token::Le),
                just(">=").to(Token::Ge),
                just("!=").to(Token::Ne),
                just("==").to(Token::Eq),
                just('<').to(Token::Lt),
                just('>').to(Token::Gt),
                just('!').to(Token::Not),
                just('=').to(Token::Assign),
            )),
        ));

        let nl = text::newline()
            .to(Token::Semi)
            .map_with(|t, e| (t, e.span()))
            .then_ignore(text::newline().repeated().padded_by(text::whitespace()));

        let number = {
            let digits = text::digits(10).to_slice();
            let frac = just('.').then(digits);
            let e = choice((just('e'), just('E')));
            let exp = e.then(one_of("+-").or_not()).then(digits);

            text::int(10)
                .then(frac.or_not())
                .then(exp.or_not())
                .to_slice()
            // .map(|s: &str| s.parse().unwrap())
        };

        let inline = choice((
            text::keyword("break").to(Token::Break),
            text::keyword("continue").to(Token::Continue),
            text::keyword("return").to(Token::Return),
            number.map(Token::Number),
            text::ident().map(Token::Ident),
            just(')').to(Token::Rparen),
            just(']').to(Token::Rbrack),
            just('}').to(Token::Rbrace),
            //
            just('(').to(Token::Lparen).then_ignore(nl.or_not()),
            just('[').to(Token::Lbrack).then_ignore(nl.or_not()),
            just('{').to(Token::Lbrace).then_ignore(nl.or_not()),
        ));

        choice((
            padded
                .map_with(|t, e| (t, e.span()))
                .padded_by(text::whitespace()),
            inline
                .map_with(|t, e| (t, e.span()))
                .padded_by(text::inline_whitespace()),
            nl,
        ))
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
    }
}

#[test]
fn scanner() {
    use Token::*;

    fn lex_print(input: &str) {
        eprintln!("input: {input:?}:");
        let tokens = Token::scanner().parse(input).unwrap();
        for (t, span) in tokens {
            let [s, e] = [span.start, span.end];
            eprintln!("{s:3}:{e:<3}    {t}  {:?}", &input[s..e]);
            // match t {
            //     Token::Lbrace => println!("{t}"),
            //     Token::Semicolon => println!("{t}"),
            //     _ => print!("{t} "),
            // }
        }
        eprintln!();
    }

    fn scan(input: &str) -> Vec<Token> {
        let (input, _eoi) = Token::scan(input).unwrap();
        input.into_iter().map(|(token, _)| token).collect()
    }

    assert_eq!(scan("if\n"), vec![If]);
    assert_eq!(scan("let\n"), vec![Let]);
    assert_eq!(scan("for\n"), vec![For]);
    assert_eq!(scan("else\n"), vec![Else]);

    assert_eq!(scan("foo\n"), vec![Ident("foo"), Semi]);
    assert_eq!(scan("break\n"), vec![Break, Semi]);
    assert_eq!(scan("return\n"), vec![Return, Semi]);
    assert_eq!(scan("continue\n"), vec![Continue, Semi]);

    assert_eq!(scan("{}\n"), vec![Lbrace, Rbrace, Semi]);
    assert_eq!(scan("{}\n{}"), vec![Lbrace, Rbrace, Semi, Lbrace, Rbrace,]);

    let src = "
if a <= b {
}
    ";

    lex_print(src);

    // lex_print("{break}");
    // lex_print("{break\n}");

    // lex_print("{lol}");
    // lex_print("{lol\n}");

    // lex_print("{lol}{lol}");

    //panic!();
}
