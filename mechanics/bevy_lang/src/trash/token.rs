use chumsky::prelude::*;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    NULL,
    Bool(bool),
    Num(String),
    Str(String),
    Ident(String),

    Print,

    ADD, // +
    SUB, // -
    MUL, // *
    QUO, // /
    REM, // %

    EQ, // ==
    NE, // !=

    LE, // <=
    LT, // <
    GE, // >=
    GT, // >

    LAND, // &&
    LOR,  // ||

    ASSIGN, // =

    LPAREN, // (
    RPAREN, // )
    LBRACK, // [
    RBRACK, // ]
    LBRACE, // {
    RBRACE, // }

    //PERIOD, // .
    //COLON, // :
    COMMA,     // ,
    SEMICOLON, // ;

    // Keywords
    //BREAK,
    //CONTINUE,
    //MATCH,
    FN,
    LET,
    IF,
    ELSE,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NULL => write!(f, "null"),
            Self::Bool(x) => write!(f, "{}", x),

            Self::Num(s) | Self::Str(s) | Self::Ident(s) => write!(f, "{}", s),

            Self::FN => write!(f, "fn"),
            Self::LET => write!(f, "let"),
            Self::Print => write!(f, "print"),
            Self::IF => write!(f, "if"),
            Self::ELSE => write!(f, "else"),

            Self::ADD => write!(f, "+"),
            Self::SUB => write!(f, "-"),
            Self::MUL => write!(f, "*"),
            Self::QUO => write!(f, "/"),
            Self::REM => write!(f, "%"),
            Self::EQ => write!(f, "=="),
            Self::NE => write!(f, "!="),
            Self::LT => write!(f, "<"),
            Self::LE => write!(f, "<="),
            Self::GT => write!(f, ">"),
            Self::GE => write!(f, ">="),
            Self::LAND => write!(f, "&&"),
            Self::LOR => write!(f, "||"),
            Self::ASSIGN => write!(f, "="),

            Self::LPAREN => write!(f, "("),
            Self::RPAREN => write!(f, ")"),
            Self::LBRACK => write!(f, "["),
            Self::RBRACK => write!(f, "]"),
            Self::LBRACE => write!(f, "{{"),
            Self::RBRACE => write!(f, "}}"),

            Self::COMMA => write!(f, ","),
            Self::SEMICOLON => write!(f, ";"),
        }
    }
}

pub fn keyword(keyword: &str) -> impl Parser<char, (), Error = Simple<char>> + Clone + '_ {
    filter(|c: &char| c.is_ascii_alphabetic())
        .repeated()
        .collect::<String>()
        .try_map(move |s, span| {
            if s == keyword {
                Ok(())
            } else {
                Err(Simple::expected_input_found(span, None, None))
            }
        })
}

pub fn lexer() -> impl Parser<char, Vec<super::Spanned<Token>>, Error = Simple<char>> {
    // A parser for numbers
    let number = text::int(10)
        .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
        .collect::<String>()
        .map(Token::Num);

    // A parser for strings
    let string = just('"')
        .ignore_then(filter(|c| *c != '"').repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .map(Token::Str);

    // A parser for operators
    let operators = choice((
        just('+').to(Token::ADD),
        just('-').to(Token::SUB),
        just('*').to(Token::MUL),
        just('/').to(Token::QUO),
        just('%').to(Token::REM),
        just("==").to(Token::EQ),
        just("!=").to(Token::NE),
        just("<=").to(Token::LE),
        just('<').to(Token::LT),
        just(">=").to(Token::GE),
        just('>').to(Token::GT),
        just("&&").to(Token::LAND),
        just("||").to(Token::LOR),
        just('=').to(Token::ASSIGN),
    ));

    // A parser for control characters (delimiters, semicolons, etc.)
    let control = choice((
        just('(').to(Token::LPAREN),
        just(')').to(Token::RPAREN),
        just('[').to(Token::LBRACK),
        just(']').to(Token::RBRACK),
        just('{').to(Token::LBRACE),
        just('}').to(Token::RBRACE),
        just(',').to(Token::COMMA),
        just(';').to(Token::SEMICOLON),
    ));

    // A parser for identifiers and keywords
    let ident = text::ident().map(|ident: String| match ident.as_str() {
        "fn" => Token::FN,
        "let" => Token::LET,
        "print" => Token::Print,
        "if" => Token::IF,
        "else" => Token::ELSE,
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),
        "null" => Token::NULL,
        _ => Token::Ident(ident),
    });

    // A single token can be one of the above
    let token = choice((number, string, operators, control, ident));

    //let comment = just("//").then(take_until(just('\n'))).padded();
    let comment = just("//").then(take_until(text::newline())).padded();

    token
        .recover_with(skip_then_retry_until([]))
        .map_with_span(|tok, span| (tok, span))
        .padded_by(comment.repeated())
        .padded()
        .repeated()
}
