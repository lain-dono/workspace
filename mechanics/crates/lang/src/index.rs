use crate::arena::{FastHashMap, FastIndexSet, Handle};
use crate::{Span, ast};

pub type HandleDecl<'a> = Handle<(ast::GlobalDecl<'a>, FastIndexSet<ast::Dependency<'a>>)>;

pub enum IndexError {
    /// Redefinition of an identifier (used for both module-scope and local redefinitions).
    Redefinition {
        /// Span of the identifier in the previous definition.
        previous: Span,

        /// Span of the identifier in the new definition.
        current: Span,
    },
    /// A declaration refers to itself directly.
    RecursiveDeclaration {
        /// The location of the name of the declaration.
        ident: Span,
        /// The point at which it is used.
        usage: Span,
    },

    /// A declaration refers to itself indirectly, through one or more other
    /// definitions.
    CyclicDeclaration {
        /// The location of the name of some declaration in the cycle.
        ident: Span,

        /// The edges of the cycle of references.
        ///
        /// Each `(decl, reference)` pair indicates that the declaration whose
        /// name is `decl` has an identifier at `reference` whose definition is
        /// the next declaration in the cycle. The last pair's `reference` is
        /// the same identifier as `ident`, above.
        path: Box<[(Span, Span)]>,
    },
}

/// A `GlobalDecl` list in which each definition occurs before all its uses.
pub struct Index<'a> {
    dependency_order: Vec<HandleDecl<'a>>,
}

impl<'a> Index<'a> {
    /// Generate an `Index` for the given translation unit.
    ///
    /// Perform a topological sort on `tu`'s global declarations, placing
    /// referents before the definitions that refer to them.
    ///
    /// Return an error if the graph of references between declarations contains
    /// any cycles.
    pub fn generate(tu: &ast::TranslationUnit<'a>) -> Result<Self, IndexError> {
        // Produce a map from global definitions' names to their `Handle<GlobalDecl>`s.
        // While doing so, reject conflicting definitions.
        let mut globals = FastHashMap::with_capacity_and_hasher(tu.decls.len(), Default::default());

        for (handle, (decl, _deps)) in tu.decls.iter() {
            if let Some(ident) = decl_ident(decl) {
                let name = ident.name;

                if let Some(old) = globals.insert(name, handle) {
                    let prev = decl_ident(&tu.decls[old].0)
                        .expect("decl should have ident for redefinition");
                    return Err(IndexError::Redefinition {
                        previous: prev.span,
                        current: ident.span,
                    });
                }
            }
        }

        let len = tu.decls.len();
        let solver = DependencySolver {
            globals: &globals,
            module: tu,
            visited: vec![false; len],
            temp_visited: vec![false; len],
            path: Vec::new(),
            out: Vec::with_capacity(len),
        };

        let dependency_order = solver.solve()?;

        Ok(Self { dependency_order })
    }

    /// Iterate over `GlobalDecl`s, visiting each definition before all its uses.
    ///
    /// Produce handles for all of the `GlobalDecl`s of the `TranslationUnit`
    /// passed to `Index::generate`, ordered so that a given declaration is
    /// produced before any other declaration that uses it.
    pub fn visit_ordered(&self) -> impl Iterator<Item = HandleDecl<'a>> + '_ {
        self.dependency_order.iter().copied()
    }
}

/// An edge from a reference to its referent in the current depth-first
/// traversal.
///
/// This is like `ast::Dependency`, except that we've determined which
/// `GlobalDecl` it refers to.
struct ResolvedDependency<'a> {
    /// The referent of some identifier used in the current declaration.
    decl: HandleDecl<'a>,
    /// Where that use occurs within the current declaration.
    usage: Span,
}

/// Local state for ordering a `TranslationUnit`'s module-scope declarations.
///
/// Values of this type are used temporarily by `Index::generate`
/// to perform a depth-first sort on the declarations.
/// Technically, what we want is a topological sort, but a depth-first sort
/// has one key benefit - it's much more efficient in storing
/// the path of each node for error generation.
struct DependencySolver<'source, 'temp> {
    /// A map from module-scope definitions' names to their handles.
    globals: &'temp FastHashMap<&'source str, HandleDecl<'source>>,
    /// The translation unit whose declarations we're ordering.
    module: &'temp ast::TranslationUnit<'source>,
    /// For each handle, whether we have pushed it onto `out` yet.
    visited: Vec<bool>,
    /// For each handle, whether it is an predecessor in the current depth-first
    /// traversal. This is used to detect cycles in the reference graph.
    temp_visited: Vec<bool>,
    /// The current path in our depth-first traversal. Used for generating
    /// error messages for non-trivial reference cycles.
    path: Vec<ResolvedDependency<'source>>,
    /// The list of declaration handles, with declarations before uses.
    out: Vec<HandleDecl<'source>>,
}

impl<'a> DependencySolver<'a, '_> {
    /// Produce the sorted list of declaration handles, and check for cycles.
    fn solve(mut self) -> Result<Vec<HandleDecl<'a>>, IndexError> {
        for (id, _) in self.module.decls.iter() {
            if self.visited[id.index()] {
                continue;
            }
            self.dfs(id)?;
        }
        Ok(self.out)
    }

    /// Ensure that all declarations used by `id` have been added to the
    /// ordering, and then append `id` itself.
    fn dfs(&mut self, id: HandleDecl<'a>) -> Result<(), IndexError> {
        let (decl, dependencies) = &self.module.decls[id];
        let id_usize = id.index();
        self.temp_visited[id_usize] = true;
        for dep in dependencies {
            if let Some(&dep_id) = self.globals.get(dep.ident) {
                self.path.push(ResolvedDependency {
                    decl: dep_id,
                    usage: dep.usage,
                });
                let dep_id_usize = dep_id.index();
                if self.temp_visited[dep_id_usize] {
                    // Found a cycle.
                    return if dep_id == id {
                        // A declaration refers to itself directly.
                        Err(IndexError::RecursiveDeclaration {
                            ident: decl_ident(decl).expect("decl should have ident").span,
                            usage: dep.usage,
                        })
                    } else {
                        // A declaration refers to itself indirectly, through
                        // one or more other definitions. Report the entire path
                        // of references.
                        let start_at = self
                            .path
                            .iter()
                            .rev()
                            .enumerate()
                            .find_map(|(i, dep)| (dep.decl == dep_id).then_some(i))
                            .unwrap_or(0);
                        Err(IndexError::CyclicDeclaration {
                            ident: decl_ident(&self.module.decls[dep_id].0)
                                .expect("decl should have ident")
                                .span,

                            path: self.path[start_at..]
                                .iter()
                                .map(|curr_dep| {
                                    let decl = decl_ident(&self.module.decls[curr_dep.decl].0);
                                    (decl.expect("decl should have ident").span, curr_dep.usage)
                                })
                                .collect(),
                        })
                    };
                } else if !self.visited[dep_id_usize] {
                    self.dfs(dep_id)?;
                }

                // Remove this edge from the current path.
                self.path.pop();
            }

            // Ignore unresolved identifiers; they may be predeclared objects.
        }

        // Remove this node from the current path.
        self.temp_visited[id_usize] = false;

        // Now everything this declaration uses has been visited, and is already
        // present in `out`. That means we we can append this one to the
        // ordering, and mark it as visited.
        self.out.push(id);
        self.visited[id_usize] = true;

        Ok(())
    }
}

const fn decl_ident<'a>(decl: &ast::GlobalDecl<'a>) -> Option<ast::Ident<'a>> {
    match decl {
        ast::GlobalDecl::Func(f) => Some(f.name),
        ast::GlobalDecl::Const(c) => Some(c.name),
        ast::GlobalDecl::Struct(s) => Some(s.name),
        ast::GlobalDecl::Type(t) => Some(t.name),
    }
}
