use super::{Extra, Stmt, Token};
use chumsky::{input::ValueInput, prelude::*};

#[derive(Debug, PartialEq)]
pub struct StructField<'src> {
    pub name: &'src str,
    pub ty: &'src str,
}

impl<'src> StructField<'src> {
    pub fn new(name: &'src str, ty: &'src str) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug, PartialEq)]
pub enum Decl<'src> {
    Func(FuncDecl<'src>),
    Ty {
        name: &'src str,
        fields: Vec<StructField<'src>>,
    },
}

impl<'src> Decl<'src> {
    pub fn parser<I: ValueInput<'src, Token = Token<'src>, Span = S>, S: 'static>()
    -> impl Parser<'src, I, Self, Extra<'src, S>> + Clone {
        let func = FuncDecl::parse().map(Self::Func);
        choice((func, Self::parse_struct()))
    }

    fn parse_struct<I: ValueInput<'src, Token = Token<'src>, Span = S>, S: 'static>()
    -> impl Parser<'src, I, Self, Extra<'src, S>> + Clone {
        let name = select! { Token::Ident(ident) => ident };

        let field = name
            .then(select! { Token::Ident(ident) => ident })
            .map(|(name, ty)| StructField { name, ty });

        let fields = field
            .separated_by(just(Token::Semi))
            .allow_leading()
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::Lbrace), just(Token::Rbrace));

        group((just(Token::Type), name, just(Token::Struct), fields))
            .map(|(_, name, _, fields)| Self::Ty { name, fields })
    }
}

#[derive(Debug, PartialEq)]
pub struct FuncParam<'src> {
    pub name: &'src str,
    pub ty: &'src str,
}

impl<'src> FuncParam<'src> {
    pub fn new(name: &'src str, ty: &'src str) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug, PartialEq)]
pub struct FuncDecl<'src> {
    pub name: &'src str,
    pub params: Vec<FuncParam<'src>>,
    pub result: Option<&'src str>,
    pub body: Vec<Stmt<'src>>,
}

impl<'src> FuncDecl<'src> {
    fn new(
        name: &'src str,
        params: Vec<FuncParam<'src>>,
        result: Option<&'src str>,
        body: Vec<Stmt<'src>>,
    ) -> Self {
        Self {
            name,
            params,
            result,
            body,
        }
    }

    pub fn parse<I: ValueInput<'src, Token = Token<'src>, Span = S>, S: 'static>()
    -> impl Parser<'src, I, Self, Extra<'src, S>> + Clone {
        let name = select! { Token::Ident(ident) => ident };

        let field = name
            .then(select! { Token::Ident(ident) => ident })
            .map(|(name, ty)| FuncParam { name, ty });

        let args = field
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::Lparen), just(Token::Rparen));

        let result = select! { Token::Ident(ident) => ident }.or_not();

        let body = Stmt::block(Stmt::parser());

        group((just(Token::Fn), name, args, result, body))
            .map(|(_, name, args, result, body)| Self::new(name, args, result, body))
    }
}

#[test]
fn decl_parse() {
    use super::Storage;

    let field = StructField::new;
    let arg = FuncParam::new;

    let mut arena = extra::SimpleState(Storage::<'_>::default());
    let (input, eoi) = Token::scan("fn foo(a i32, b isize) {}").unwrap();
    let input = input.map(eoi, |(t, s)| (t, s));
    assert_eq!(
        Decl::parser().parse_with_state(input, &mut arena).unwrap(),
        Decl::Func(FuncDecl::new(
            "foo",
            vec![arg("a", "i32"), arg("b", "isize")],
            None,
            vec![]
        ))
    );

    let mut arena = extra::SimpleState(Storage::<'_>::default());
    let (input, eoi) = Token::scan("fn foo(a i32) i32 {}").unwrap();
    let input = input.map(eoi, |(t, s)| (t, s));
    assert_eq!(
        Decl::parser().parse_with_state(input, &mut arena).unwrap(),
        Decl::Func(FuncDecl::new(
            "foo",
            vec![arg("a", "i32")],
            Some("i32"),
            vec![]
        ))
    );

    let mut arena = extra::SimpleState(Storage::<'_>::default());
    let (input, eoi) = Token::scan("type Foo struct { a i32 \n b isize }").unwrap();
    let input = input.map(eoi, |(t, s)| (t, s));
    assert_eq!(
        Decl::parser().parse_with_state(input, &mut arena).unwrap(),
        Decl::Ty {
            name: "Foo",
            fields: vec![field("a", "i32"), field("b", "isize")]
        }
    );

    let mut arena = extra::SimpleState(Storage::<'_>::default());
    let (input, eoi) = Token::scan("type Foo struct { a i32 ; b isize }").unwrap();
    let input = input.map(eoi, |(t, s)| (t, s));
    assert_eq!(
        Decl::parser().parse_with_state(input, &mut arena).unwrap(),
        Decl::Ty {
            name: "Foo",
            fields: vec![field("a", "i32"), field("b", "isize")]
        }
    );
}
