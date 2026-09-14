use crate::arena::{Arena, FastHashMap, FastIndexMap, Handle};
use crate::const_eval::{ExpressionKind, ExpressionKindTracker};
use crate::{Span, ast, ir, ty};
use crate::{index::Index, layouter::Layouter, ty::Typifier};
use core::marker::PhantomData;

// #[derive(thiserror::Error)]
pub struct Error {}

/// State for constructing a `crate::Module`.
pub struct GlobalContext<'source, 'temp, 'out> {
    /// The `TranslationUnit`'s expressions arena.
    ast_expressions: &'temp Arena<ast::Expr<'source>>,
    /// The `TranslationUnit`'s types arena.
    types: &'temp Arena<ast::Type<'source>>,

    // Naga IR values.
    /// The map from the names of module-scope declarations to the Naga IR
    /// `Handle`s we have built for them, owned by `Lowerer::lower`.
    globals: &'temp mut FastHashMap<&'source str, LoweredGlobalDecl>,
    /// The module we're constructing.
    module: &'out mut ir::Module,
    const_typifier: &'temp mut Typifier,
    layouter: &'temp mut Layouter,
    global_expression_kind_tracker: &'temp mut ExpressionKindTracker,
}

impl<'source> GlobalContext<'source, '_, '_> {
    fn as_const(&mut self) -> ExpressionContext<'source, '_, '_> {
        ExpressionContext {
            ast_expressions: self.ast_expressions,
            globals: self.globals,
            types: self.types,
            module: self.module,
            const_typifier: self.const_typifier,
            layouter: self.layouter,
            expr_type: ExpressionContextType::Constant(None),
            global_expression_kind_tracker: self.global_expression_kind_tracker,
        }
    }
}
pub struct ExpressionContext<'source, 'temp, 'out> {
    // WGSL AST values.
    ast_expressions: &'temp Arena<ast::Expr<'source>>,
    types: &'temp Arena<ast::Type<'source>>,

    // Naga IR values.
    /// The map from the names of module-scope declarations to the Naga IR
    /// `Handle`s we have built for them, owned by `Lowerer::lower`.
    globals: &'temp mut FastHashMap<&'source str, LoweredGlobalDecl>,

    /// The IR [`Module`] we're constructing.
    ///
    /// [`Module`]: crate::Module
    module: &'out mut ir::Module,

    /// Type judgments for [`module::global_expressions`].
    ///
    /// [`module::global_expressions`]: crate::Module::global_expressions
    const_typifier: &'temp mut Typifier,
    layouter: &'temp mut Layouter,
    global_expression_kind_tracker: &'temp mut ExpressionKindTracker,

    /// Whether we are lowering a constant expression or a general
    /// runtime expression, and the data needed in each case.
    expr_type: ExpressionContextType<'temp, 'out>,
}

/// The type of Naga IR expression we are lowering an [`ast::Expression`] to.
pub enum ExpressionContextType<'temp, 'out> {
    /// We are lowering to an arbitrary runtime expression, to be
    /// included in a function's body.
    ///
    /// The given [`LocalExpressionContext`] holds information about local
    /// variables, arguments, and other definitions available only to runtime
    /// expressions, not constant or override expressions.
    Runtime(LocalExpressionContext<'temp, 'out>),

    /// We are lowering to a constant expression, to be included in the module's
    /// constant expression arena.
    ///
    /// Everything global constant expressions are allowed to refer to is
    /// available in the [`ExpressionContext`], but local constant expressions can
    /// also refer to other
    Constant(Option<LocalExpressionContext<'temp, 'out>>),
}

pub struct LocalExpressionContext<'temp, 'out> {
    /*
    /// A map from [`ast::Local`] handles to the Naga expressions we've built for them.
    ///
    /// This is always [`StatementContext::local_table`] for the
    /// enclosing statement; see that documentation for details.
    local_table: &'temp FastHashMap<Handle<ast::Local>, Declared<Typed<Handle<crate::Expression>>>>,
    function: &'out mut crate::Function,
    block: &'temp mut crate::Block,
    emitter: &'temp mut Emitter,
    typifier: &'temp mut Typifier,
    /// Which `Expression`s in `self.naga_expressions` are const expressions, in
    /// the WGSL sense.
    ///
    /// See [`StatementContext::local_expression_kind_tracker`] for details.
    local_expression_kind_tracker: &'temp mut crate::proc::ExpressionKindTracker,
    */
    marker: PhantomData<(&'temp (), &'out ())>,
}

/// Whether a declaration accepts abstract types, or concretizes.
enum AbstractRule {
    /// This declaration concretizes its initialization expression.
    Concretize,
    /// This declaration can accept initializers with abstract types.
    Allow,
}

pub struct Lowerer<'source, 'temp> {
    index: &'temp Index<'source>,
}

impl<'source, 'temp> Lowerer<'source, 'temp> {
    pub const fn new(index: &'temp Index<'source>) -> Self {
        Self { index }
    }

    pub fn lower(&mut self, tu: ast::TranslationUnit<'source>) -> Result<ir::Module, Error> {
        let mut module = ir::Module::default();

        let mut ctx = GlobalContext {
            ast_expressions: &tu.exprs,
            globals: &mut FastHashMap::default(),
            types: &tu.types,
            module: &mut module,
            const_typifier: &mut Typifier::new(),
            layouter: &mut Layouter::default(),
            global_expression_kind_tracker: &mut ExpressionKindTracker::new(),
        };

        for decl_handle in self.index.visit_ordered() {
            let span = tu.decls.get_span(decl_handle);
            let (decl, deps) = &tu.decls[decl_handle];

            match decl {
                ast::GlobalDecl::Func(func) => {
                    let lowered_decl = self.function(func, span, &mut ctx)?;
                    ctx.globals.insert(func.name.name, lowered_decl);
                }
                ast::GlobalDecl::Const(c) => {
                    let mut ectx = ctx.as_const();

                    let explicit_ty =
                        c.ty.map(|ast| self.resolve_ast_type(ast, &mut ectx))
                            .transpose()?;

                    let (ty, init) = self.type_and_init(
                        c.name,
                        Some(c.init),
                        explicit_ty,
                        AbstractRule::Allow,
                        &mut ectx,
                    )?;

                    let init = init.expect("Global const must have init");

                    let handle = ctx.module.constants.append(
                        ir::Constant {
                            name: Some(c.name.name.to_string()),
                            ty,
                            init,
                        },
                        span,
                    );

                    ctx.globals
                        .insert(c.name.name, LoweredGlobalDecl::Const(handle));
                }
                ast::GlobalDecl::Struct(s) => {
                    let handle = self.structure(s, span, &mut ctx)?;
                    ctx.globals
                        .insert(s.name.name, LoweredGlobalDecl::Type(handle));
                }
                ast::GlobalDecl::Type(alias) => {
                    let ty = self.resolve_ast_type(
                        alias.ty,
                        Some(alias.name.name.to_string()),
                        &mut ctx.as_const(),
                    )?;

                    ctx.globals
                        .insert(alias.name.name, LoweredGlobalDecl::Type(ty));
                }
            }
        }

        todo!("{:?}", tu)

        // Constant evaluation may leave abstract-typed literals and
        // compositions in expression arenas, so we need to compact the module
        // to remove unused expressions and types.
        // crate::compact::compact(&mut module);
        // Ok(module)
    }

    fn function(
        &mut self,
        f: &ast::Func<'source>,
        span: Span,
        ctx: &mut GlobalContext<'source, '_, '_>,
    ) -> Result<LoweredGlobalDecl, Error> {
        let mut local_table = FastHashMap::default();
        let mut expressions = Arena::new();
        let mut named_expressions = FastIndexMap::default();
        let mut local_expression_kind_tracker = ExpressionKindTracker::new();

        let arguments = f
            .params
            .iter()
            .enumerate()
            .map(|(i, arg)| -> Result<_, Error> {
                let ty = self.resolve_ast_type(arg.ty, None, &mut ctx.as_const())?;
                let expr = expressions.append(ir::Expr::FunctionArgument(i as u32), arg.name.span);
                local_table.insert(arg.handle, Declared::Runtime(Typed::Plain(expr)));
                named_expressions.insert(expr, (arg.name.name.to_string(), arg.name.span));
                local_expression_kind_tracker.insert(expr, ExpressionKind::Runtime);
                let name = Some(arg.name.name.to_string());
                Ok(ir::FuncParam { name, ty })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        let result = f
            .result
            .as_ref()
            .map(|ty| -> Result<_, Error> { self.resolve_ast_type(ty, None, &mut ctx.as_const()) })
            .transpose()?;

        let mut function = ir::Func {
            name: Some(f.name.name.to_string()),
            params: arguments,
            result,
            vars: Arena::new(),
            expr: expressions,
            named_expressions: Default::default(),
            body: ir::Block::default(),
        };

        let mut typifier = Typifier::default();

        let mut stmt_ctx = StatementContext {
            local_table: &mut local_table,
            globals: ctx.globals,
            ast_expressions: ctx.ast_expressions,
            const_typifier: ctx.const_typifier,
            typifier: &mut typifier,
            layouter: ctx.layouter,
            function: &mut function,
            named_expressions: &mut named_expressions,
            types: ctx.types,
            module: ctx.module,
            local_expression_kind_tracker: &mut local_expression_kind_tracker,
            global_expression_kind_tracker: ctx.global_expression_kind_tracker,
        };

        let mut body = self.block(&f.body, false, &mut stmt_ctx)?;

        ensure_block_returns(&mut body);

        function.body = body;

        function.named_expressions = named_expressions
            .into_iter()
            .map(|(key, (name, _))| (key, name))
            .collect();

        Ok(LoweredGlobalDecl::Function(
            ctx.module.funcs.append(function, span),
        ))
    }

    fn resolve_ast_type(
        &mut self,
        handle: Handle<ast::Type<'source>>,
        name: Option<String>,
        ctx: &mut ExpressionContext<'source, '_, '_>,
    ) -> Result<Handle<ty::Type>, Error> {
        let inner = match ctx.types[handle] {
            ast::Type::Scalar(scalar) => scalar.to_inner_scalar(),
            ast::Type::Pointer(base) => ty::TypeInner::Pointer(self.resolve_ast_type(base, ctx)?),

            ast::Type::Array(base, size) => {
                let base = self.resolve_ast_type(base, None, &mut ctx.as_const())?;
                let size = self.array_size(size, ctx)?;
                ctx.layouter.update(ctx.module.to_ctx()).unwrap();
                let stride = ctx.layouter[base].to_stride();
                ty::TypeInner::Array(base, size, stride)
            }

            ast::Type::User(ref ident) => {
                return match ctx.globals.get(ident.name) {
                    Some(&LoweredGlobalDecl::Type(handle)) => Ok(handle),
                    Some(_) => Err(Error::Unexpected(ident.span, ExpectedToken::Type)),
                    None => Err(Error::UnknownType(ident.span)),
                };
            }
        };

        Ok(ctx.as_global().ensure_type_exists(name, inner))
    }
}

/// An `ast::GlobalDecl` for which we have built the Naga IR equivalent.
enum LoweredGlobalDecl {
    Function(Handle<ir::Func>),
    Const(Handle<ir::Constant>),
    Type(Handle<ty::Type>),
    EntryPoint,
}
