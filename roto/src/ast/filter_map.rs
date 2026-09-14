use super::{
    ParseError,
    ident::Ident,
    meta::Meta,
    parser::{Parser, Result},
    token::{Keyword, Token},
    {EnumDecl, FilterMap, FilterType, Params, StructDecl, TypeExpr, Variant},
};

/// # Parsing `filtermap` and `filter` sections
impl Parser<'_, '_> {
    /// Parse a filtermap or filter expression
    ///
    /// ```ebnf
    /// FilterMap ::= ( 'filtermap' | 'filter' ) Identifier Params RetType Block
    /// ```
    pub(super) fn filter_map(&mut self) -> Result<FilterMap> {
        let (token, span) = self.next()?;
        let filter_type = match token {
            Token::Keyword(Keyword::FilterMap) => FilterType::FilterMap,
            Token::Keyword(Keyword::Filter) => FilterType::Filter,
            _ => return Err(ParseError::expected("`filtermap` or `filter`", token, span)),
        };

        let ident = self.ident()?;
        let params = self.params()?;
        let body = self.block()?;

        Ok(FilterMap {
            filter_type,
            ident,
            params,
            body,
        })
    }

    /// Parse an optional with clause for filtermap, define and apply
    ///
    /// ```ebnf
    /// Params ::= '(' TypeIdentField (',' TypeIdentField)* ')'
    /// ```
    pub fn params(&mut self) -> Result<Meta<Params>> {
        let m = self.separated(Token::ROUND, Token::Comma, Self::type_ident_field)?;
        Ok(m.map_node(Params))
    }

    /// Parse an identifier and a type identifier separated by a colon
    ///
    /// ```ebnf
    /// TypeIdentField ::= Identifier ':' TypeExpr
    /// ```
    fn type_ident_field(&mut self) -> Result<(Meta<Ident>, Meta<TypeExpr>)> {
        let field_name = self.ident()?;
        self.take(Token::Colon)?;
        let ty = self.type_expr()?;
        Ok((field_name, ty))
    }

    fn type_parameters(&mut self) -> Result<Vec<Meta<Ident>>> {
        let params = if self.peek_is(Token::SquareLeft) {
            self.separated(Token::SQUARE, Token::Comma, Self::ident)?
                .into_inner()
        } else {
            Vec::new()
        };
        Ok(params)
    }

    /// Parse a record type declaration
    ///
    /// ```ebnf
    /// Type ::= 'record' Identifier RecordType
    /// ```
    pub(super) fn record_type_assignment(&mut self) -> Result<StructDecl> {
        self.take(Token::Keyword(Keyword::Record))?;
        Ok(StructDecl {
            ident: self.ident()?,
            params: self.type_parameters()?,
            fields: self.record_type()?,
        })
    }

    pub(super) fn variant_declaration(&mut self) -> Result<EnumDecl> {
        self.take(Token::Keyword(Keyword::Variant))?;
        Ok(EnumDecl {
            ident: self.ident()?,
            params: self.type_parameters()?,
            variants: self.separated(Token::CURLY, Token::Comma, Self::enum_variant)?,
        })
    }

    fn enum_variant(&mut self) -> Result<Variant> {
        Ok(Variant {
            ident: self.ident()?,
            fields: if self.peek_is(Token::RoundLeft) {
                self.separated(Token::ROUND, Token::Comma, Self::type_expr)?
                    .into_inner()
            } else {
                Vec::new()
            },
        })
    }
}
