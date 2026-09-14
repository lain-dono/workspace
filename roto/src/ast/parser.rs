use super::{
    Decl, FuncDecl, Keyword, Lexer, Meta, ParseError, ParseErrorKind, Path, Span, Spans,
    SyntaxTree, Test, Token, ident::Ident,
};

pub(crate) type Result<T> = core::result::Result<T, ParseError>;

pub struct Parser<'src, 'spans> {
    pub(crate) file: usize,
    pub(crate) file_length: usize,
    pub(crate) lexer: Lexer<'src>,
    pub spans: &'spans mut Spans,
}

/// # Helper methods
impl<'src> Parser<'src, '_> {
    /// Move the lexer forward and return the token
    pub(crate) fn next(&mut self) -> Result<(Token<'src>, Span)> {
        match self.lexer.next() {
            Some((Ok(token), span)) => Ok((token, Span::new(self.file, span))),
            Some((Err(()), span)) => Err(ParseError {
                kind: ParseErrorKind::InvalidToken,
                location: Span::new(self.file, span),
                note: None,
            }),
            None => Err(ParseError {
                kind: ParseErrorKind::EndOfInput,
                location: Span::new(self.file, self.file_length..self.file_length),
                note: None,
            }),
        }
    }

    /// Move the lexer forward if the next token matches the given token
    pub(crate) fn next_is(&mut self, token: Token) -> Option<Span> {
        if self.peek_is(token) {
            let (_, span) = self.next().unwrap();
            Some(span)
        } else {
            None
        }
    }

    /// Peek the next token
    pub(crate) fn peek(&mut self) -> Option<&Token<'src>> {
        self.lexer.peek().and_then(|(tok, _)| tok.as_ref().ok())
    }

    /// Peek the next token and return whether it matches the given token
    pub(crate) fn peek_is(&mut self, token: Token) -> bool {
        self.peek().is_some_and(|lexed| &token == lexed)
    }

    /// Move the lexer forward and assert that it matches the token
    pub(crate) fn take(&mut self, token: Token) -> Result<Span> {
        let (next, span) = self.next()?;
        if next == token {
            Ok(span)
        } else {
            Err(ParseError::expected(token, next, span))
        }
    }

    /// Parse a separated and delimited list of items
    ///
    /// Assuming that `{`, `}` and `,` are the opening, closing and separating
    /// tokens, respectively. And the given parser passes `FOO`, then this
    /// function corresponds to the following grammar rule:
    ///
    /// ```ebnf
    /// '{' (FOO (',' FOO)* ',')? '}'
    /// ```
    ///
    /// So, the list is allowed to be empty and a trailing separator is allowed.
    pub(crate) fn separated<T>(
        &mut self,
        (open, close): (Token, Token),
        sep: Token,
        mut parser: impl FnMut(&mut Self) -> Result<T>,
    ) -> Result<Meta<Vec<T>>> {
        let start_span = self.take(open)?;

        let mut items = Vec::new();

        // If there are no fields, return the empty vec.
        if let Some(end_span) = self.next_is(close.clone()) {
            let span = start_span.merge(end_span);
            return Ok(self.add_span(span, items));
        }

        // Parse the first field
        items.push(parser(self)?);

        // Now each field must be separated by a comma
        while self.next_is(sep.clone()).is_some() {
            // If we have found the curly right, we have just
            // parsed the trailing comma.
            if self.peek_is(close.clone()) {
                break;
            }

            items.push(parser(self)?);
        }

        let end_span = self.take(close)?;
        let span = start_span.merge(end_span);
        Ok(self.add_span(span, items))
    }
}

/// # Parsing the syntax tree
impl<'src, 'spans> Parser<'src, 'spans> {
    pub fn parse(file: usize, spans: &'spans mut Spans, input: &'src str) -> Result<SyntaxTree> {
        Self::run_parser(Self::tree, file, spans, input)
    }

    pub fn run_parser<T>(
        mut parser: impl FnMut(&mut Self) -> Result<T>,
        file: usize,
        spans: &'spans mut Spans,
        input: &'src str,
    ) -> Result<T> {
        let mut p = Self {
            file,
            file_length: input.len(),
            lexer: Lexer::new(input),
            spans,
        };
        let out = parser(&mut p)?;
        if let Some((_, s)) = p.lexer.next() {
            Err(ParseError {
                kind: ParseErrorKind::FailedToParseEntireInput,
                location: Span::new(file, s),
                note: None,
            })
        } else {
            Ok(out)
        }
    }

    pub fn tree(&mut self) -> Result<SyntaxTree> {
        let mut declarations = Vec::new();
        while self.peek().is_some() {
            declarations.push(self.root()?);
        }
        Ok(SyntaxTree { declarations })
    }

    /// Parse a root expression
    ///
    /// ```ebnf
    /// Root ::= FilterMap | Function | Type
    /// ```
    pub fn root(&mut self) -> Result<Decl> {
        let end_of_input = ParseError {
            kind: ParseErrorKind::EndOfInput,
            location: Span::new(self.file, self.file_length..self.file_length),
            note: None,
        };
        let expr = match self.peek().ok_or(end_of_input)? {
            Token::Keyword(Keyword::FilterMap | Keyword::Filter) => Decl::Fmap(self.filter_map()?),
            Token::Keyword(Keyword::Record) => Decl::Struct(self.record_type_assignment()?),
            Token::Keyword(Keyword::Variant) => Decl::Enum(self.variant_declaration()?),
            Token::Keyword(Keyword::Fn) => Decl::Func(self.function()?),
            Token::Keyword(Keyword::Test) => Decl::Test(self.test()?),
            Token::Keyword(Keyword::Import) => Decl::Import(self.import()?),
            _ => {
                let (token, span) = self.next()?;
                return Err(ParseError::expected(
                    "a function, filter, filtermap or import",
                    token,
                    span,
                ));
            }
        };
        Ok(expr)
    }

    /// Parse a term section
    ///
    /// ```ebnf
    /// Function ::= 'fn' Identifier '{' Body '}'
    /// ```
    fn function(&mut self) -> Result<FuncDecl> {
        self.take(Token::Keyword(Keyword::Fn))?;

        Ok(FuncDecl {
            ident: self.ident()?,
            params: self.params()?,
            result: if self.next_is(Token::Arrow).is_some() {
                Some(self.type_expr()?)
            } else {
                None
            },
            body: self.block()?,
        })
    }

    fn test(&mut self) -> Result<Test> {
        self.take(Token::Keyword(Keyword::Test))?;
        let ident = self.ident()?;
        let body = self.block()?;
        Ok(Test { ident, body })
    }

    pub(crate) fn import(&mut self) -> Result<Vec<Meta<Path>>> {
        self.take(Token::Keyword(Keyword::Import))?;
        let path = self.path_expr()?;
        self.take(Token::Semi)?;
        Ok(path)
    }
}

/// # Parsing identifiers
impl Parser<'_, '_> {
    /// Parse an identifier
    ///
    /// The `contains` and `type` keywords are treated as identifiers,
    /// because we already have tests that use these as names for methods.
    pub(crate) fn ident(&mut self) -> Result<Meta<Ident>> {
        let (token, span) = self.next()?;
        let ident = match token {
            Token::Ident(s) => s,
            Token::Keyword(_) => {
                return Err(
                    ParseError::expected("an identifier", &token, span).with_note(format!(
                        "`{token}` is a keyword and cannot be used as an identifier."
                    )),
                );
            }
            _ => return Err(ParseError::expected("an identifier", token, span)),
        };
        Ok(self.add_span(span, Ident::from(ident)))
    }
}

impl Parser<'_, '_> {
    pub(crate) fn add_span<T>(&mut self, span: Span, x: T) -> Meta<T> {
        self.spans.add(span, x)
    }

    pub(crate) fn get_span<T>(&mut self, x: &Meta<T>) -> Span {
        self.spans.get(x)
    }

    pub(crate) fn merge_spans<T, U>(&mut self, x: &Meta<T>, y: &Meta<U>) -> Span {
        self.spans.merge(x, y)
    }
}
