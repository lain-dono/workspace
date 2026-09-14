use super::{Handle, Token, UniqueArena};
use chumsky::{input::ValueInput, prelude::*};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Scalar {
    Bool,

    F32,
    F64,

    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,

    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
}

// #[repr(u8)]
// #[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
// pub enum ArraySize {
//     Constant(u32),
//     // Pending(Handle<Override>),
//     // Dynamic,
// }

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct StructMember<'src>(pub &'src str, pub Handle<Type<'src>>);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Variant<'src> {
    Unit,
    Struct(Vec<StructMember<'src>>),
    TupleStruct(Vec<Handle<Type<'src>>>),
}

pub type Ty<'src> = Handle<Type<'src>>;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Type<'src> {
    Error,
    Scalar(Scalar),
    Struct(Vec<StructMember<'src>>),
    Tuple(Vec<Handle<Type<'src>>>),
    Enum(Vec<Variant<'src>>),
    // Array { base: Ty<'src>, size: ArraySize },
    // array,pointer,enum,etc
}

pub type Extra<'src, T = Token<'src>, S = SimpleSpan> =
    extra::Full<Rich<'src, T, S>, extra::SimpleState<UniqueArena<Type<'src>>>, ()>;

impl<'src> Type<'src> {
    pub fn parser<I: ValueInput<'src, Token = Token<'src>, Span = S>, S: 'static>()
    -> impl Parser<'src, I, Handle<Self>, Extra<'src, Token<'src>, S>> + Clone {
        recursive(|element| {
            let scalar = Self::scalar();
            let unnamed = just(Token::Struct).ignore_then(Self::tuple_members(element.clone()));
            let named = just(Token::Struct).ignore_then(Self::struct_members(element));
            choice((
                scalar.map_with(|ty, e| e.state().insert(Self::Scalar(ty))),
                unnamed.map_with(|members, e| e.state().insert(Self::Tuple(members))),
                named.map_with(|members, e| e.state().insert(Self::Struct(members))),
            ))
        })
    }

    fn scalar<I: ValueInput<'src, Token = Token<'src>, Span = S>, S: 'static>()
    -> impl Parser<'src, I, Scalar, Extra<'src, Token<'src>, S>> + Clone {
        select! {
            Token::Ident("bool") => Scalar::Bool,

            Token::Ident("f32") => Scalar::F32,
            Token::Ident("f64") => Scalar::F64,

            Token::Ident("i8") => Scalar::I8,
            Token::Ident("i16") => Scalar::I16,
            Token::Ident("i32") => Scalar::I32,
            Token::Ident("i64") => Scalar::I64,
            Token::Ident("i128") => Scalar::I128,
            Token::Ident("isize") => Scalar::Isize,

            Token::Ident("u8") => Scalar::U8,
            Token::Ident("u16") => Scalar::U16,
            Token::Ident("u32") => Scalar::U32,
            Token::Ident("u64") => Scalar::U64,
            Token::Ident("u128") => Scalar::U128,
            Token::Ident("usize") => Scalar::Usize,
        }
    }

    fn struct_members<I: ValueInput<'src, Token = Token<'src>, Span = S>, S: 'static>(
        element: impl Parser<'src, I, Handle<Self>, Extra<'src, Token<'src>, S>> + Clone,
    ) -> impl Parser<'src, I, Vec<StructMember<'src>>, Extra<'src, Token<'src>, S>> + Clone {
        let name = select! { Token::Ident(ident) => ident };
        let field = name.then(element).map(|(name, ty)| StructMember(name, ty));

        field
            .separated_by(just(Token::Semi))
            .allow_leading()
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::Lbrace), just(Token::Rbrace))
    }

    fn tuple_members<I: ValueInput<'src, Token = Token<'src>, Span = S>, S: 'static>(
        element: impl Parser<'src, I, Handle<Self>, Extra<'src, Token<'src>, S>> + Clone,
    ) -> impl Parser<'src, I, Vec<Handle<Type<'src>>>, Extra<'src, Token<'src>, S>> + Clone {
        element
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::Lparen), just(Token::Rparen))
    }
}

#[test]
fn parse() {
    let (input, eoi) = Token::scan("i32").unwrap();
    let input = input.map(eoi, |(t, s)| (t, s));
    let mut arena = extra::SimpleState(UniqueArena::<Type<'_>>::default());
    let ty = Type::parser().parse_with_state(input, &mut arena).unwrap();
    assert_eq!(arena[ty], Type::Scalar(Scalar::I32));

    let (input, eoi) = Token::scan("struct { a i32; b usize }").unwrap();
    let input = input.map(eoi, |(t, s)| (t, s));
    let mut arena = extra::SimpleState(UniqueArena::<Type<'_>>::default());
    let ty = Type::parser().parse_with_state(input, &mut arena).unwrap();
    assert_eq!(ty, Handle::from_usize(2));
    assert_eq!(arena[Handle::from_usize(0)], Type::Scalar(Scalar::I32));
    assert_eq!(arena[Handle::from_usize(1)], Type::Scalar(Scalar::Usize));
    assert_eq!(
        arena[Handle::from_usize(2)],
        Type::Struct(vec![
            StructMember("a", Handle::from_usize(0)),
            StructMember("b", Handle::from_usize(1)),
        ])
    );
}
