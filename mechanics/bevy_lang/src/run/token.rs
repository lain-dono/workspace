use chumsky::prelude::*;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Bool(bool),
    Num(String),
    Str(String),
    Ident(String),

    ADD, // +
    SUB, // -
    MUL, // *
    QUO, // /
    REM, // %

    NOT, // !
    AND, // &
    IOR, // |
    XOR, // ^
    SHL, // <<
    SHR, // >>

    LAND, // &&
    LIOR, // ||

    EQ, // ==
    NE, // !=

    LT, // <
    LE, // <=
    GT, // >
    GE, // >=

    ASSIGN, // =

    LPAREN,    // (
    RPAREN,    // )
    LBRACK,    // [
    RBRACK,    // ]
    LBRACE,    // {
    RBRACE,    // }
    COMMA,     // ,
    SEMICOLON, // ;

    NULL,
    FN,
    LET,
    IF,
    ELSE,

    Print,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NULL => write!(f, "null"),
            Self::Bool(x) => write!(f, "{}", x),
            Self::Num(n) => write!(f, "{}", n),
            Self::Str(s) => write!(f, "{}", s),
            Self::Ident(s) => write!(f, "{}", s),
            Self::FN => write!(f, "fn"),
            Self::LET => write!(f, "let"),
            Self::Print => write!(f, "print"),
            Self::IF => write!(f, "if"),
            Self::ELSE => write!(f, "else"),

            // operators
            Self::ADD => write!(f, "+"),
            Self::SUB => write!(f, "-"),
            Self::MUL => write!(f, "*"),
            Self::QUO => write!(f, "/"),
            Self::REM => write!(f, "%"),

            Self::NOT => write!(f, "!"),
            Self::AND => write!(f, "&"),
            Self::IOR => write!(f, "|"),
            Self::XOR => write!(f, "^"),
            Self::SHL => write!(f, "<<"),
            Self::SHR => write!(f, ">>"),

            Self::EQ => write!(f, "=="),
            Self::NE => write!(f, "!="),
            Self::LT => write!(f, "<"),
            Self::LE => write!(f, "<="),
            Self::GT => write!(f, ">"),
            Self::GE => write!(f, ">="),

            Self::LAND => write!(f, "&&"),
            Self::LIOR => write!(f, "||"),

            Self::ASSIGN => write!(f, "="),

            // controls
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

pub fn lexer() -> impl Parser<char, Vec<(Token, std::ops::Range<usize>)>, Error = Simple<char>> {
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
        just("!").to(Token::NOT),
        just('+').to(Token::ADD),
        just('-').to(Token::SUB),
        just('*').to(Token::MUL),
        just('/').to(Token::QUO),
        just('%').to(Token::REM),
        just("==").to(Token::EQ),
        just("!=").to(Token::NE),
        just("<<").to(Token::SHL),
        just("<=").to(Token::LE),
        just('<').to(Token::LT),
        just(">>").to(Token::SHR),
        just(">=").to(Token::GE),
        just('>').to(Token::GT),
        just("&&").to(Token::LAND),
        just("&").to(Token::AND),
        just("||").to(Token::LIOR),
        just("|").to(Token::IOR),
        just("^").to(Token::XOR),
        just('=').to(Token::ASSIGN),
    ));

    // A parser for control characters (delimiters, semicolons, etc.)
    let controls = choice((
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
        "if" => Token::IF,
        "else" => Token::ELSE,
        "null" => Token::NULL,

        "print" => Token::Print,
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),

        _ => Token::Ident(ident),
    });

    // A single token can be one of the above
    let token = choice((number, string, operators, controls, ident));

    let comment = just("//").then(take_until(just('\n'))).padded();

    token
        .recover_with(skip_then_retry_until([]))
        .map_with_span(|tok, span| (tok, span))
        .padded_by(comment.repeated())
        .padded()
        .repeated()
}
