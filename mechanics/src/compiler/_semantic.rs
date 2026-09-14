use crate::compiler::Span;
use crate::compiler::arena::{Arena, BadHandle, Handle, Range, Unique};

pub type Name = std::borrow::Cow<'static, str>;

pub enum Flow {
    Placeholder,

    Mark(Marker),
    Jump(Marker),

    Choice(Marker, Marker),
}

type Marker = Handle<usize>;

pub struct Function {
    pub name: Name,
    pub args: Vec<(String, Handle<Ty>)>,
    pub body: Handle<State>,
    pub result: Handle<Ty>,

    pub flow: std::ops::Range<usize>,
}

#[derive(thiserror::Error, Debug)]
pub enum SemanticError {
    #[error("label not found {0:?}")]
    LabelNotFound(Option<Name>),

    #[error("expect unnamed block {0:?}")]
    ExpectUnnamedBlock(Handle<Item>),

    #[error("bad handle {0:?}")]
    BadHandle(#[from] BadHandle),
}

#[derive(Default)]
pub struct Semantic {
    functions: Arena<Function>,

    marks: Arena<usize>,
    types: Unique<Ty>,
    state: Arena<State>,
    items: Arena<Item>,

    flow: Arena<Flow>,
}

impl Semantic {
    pub fn ty(&mut self, ty: Ty) -> Handle<Ty> {
        self.types.add(ty)
    }

    pub fn emit(&mut self, items: impl IntoIterator<Item = Item>) -> Range<Item> {
        let old_length = self.items.len();
        for item in items.into_iter() {
            self.items.add(item);
        }
        self.items.range_from(old_length)
    }

    pub fn function(
        &mut self,
        name: Name,
        args: Vec<(String, Handle<Ty>)>,
        body: Range<Item>,
        result: Handle<Ty>,
    ) -> Result<Handle<Function>, SemanticError> {
        for &(_, handle) in &args {
            self.types.check_contains_handle(handle)?;
        }

        self.types.check_contains_handle(result)?;

        let init = self.flow.len();
        let body = self.block(Some(name.clone()), Kind::Root, None, body)?;
        let exit = self.flow.len();

        /*
        for (index, flow) in self.flow[init..].iter().enumerate() {
            if let &Flow::Mark(label) = flow {
                self.marks[label] = init + index;
            }
        }
        */

        Ok(self.functions.add(Function {
            body,
            args,
            name,
            result,

            flow: init..exit,
        }))
    }

    fn create_mark(&mut self) -> Handle<usize> {
        self.marks.add(usize::MAX)
    }

    fn place_mark(&mut self, mark: Handle<usize>) {
        self.flow.add(Flow::Mark(mark));
    }

    fn placeholder(&mut self) -> Handle<Flow> {
        self.flow.add(Flow::Placeholder)
    }

    fn block(
        &mut self,
        name: Option<Name>,
        kind: Kind,
        parent: Option<Handle<State>>,
        body: Range<Item>,
    ) -> Result<Handle<State>, SemanticError> {
        let start = self.create_mark();
        let end = self.create_mark();

        let state = self.state.add(State {
            parent,
            start,
            end,
            name,
            kind,
            range: Range::full_range_from_size(0),
        });

        let flow_start = self.flow.len();

        self.place_mark(self.state[state].start);
        for item in body {
            self.item(item, state)?;
        }
        self.place_mark(self.state[state].end);

        self.state[state].range = self.flow.range_from(flow_start);

        Ok(state)
    }

    fn item(&mut self, item: Handle<Item>, parent: Handle<State>) -> Result<(), SemanticError> {
        match self.items.try_get(item)? {
            Item::Block(name, body) => self
                .block(name.clone(), Kind::Open, Some(parent), body.clone())
                .map(|_| ()),

            &Item::Bind(ref lhs, ty, rhs, mutable) => {
                //
                Ok(())
            }
            &Item::Assign(lhs, rhs) => todo!(),

            Item::Continue(label) => {
                let target = self.jump_target(parent, label.as_ref())?;
                self.flow.add(Flow::Jump(target.start));
                Ok(())
            }
            Item::Break(label) => {
                let target = self.jump_target(parent, label.as_ref())?;
                self.flow.add(Flow::Jump(target.end));
                Ok(())
            }

            &Item::Select {
                cond,
                accept,
                reject,
            } => {
                let _cond = self.expect_unnamed_block(cond, parent, Kind::Cond)?;
                let select = self.placeholder();

                let accept = self.expect_unnamed_block(accept, parent, Kind::Open)?;
                let (accept, reject) = if let Some(reject) = reject {
                    let exit = self.placeholder();
                    let reject = self.expect_unnamed_block(reject, parent, Kind::Open)?;
                    self.flow[exit] = Flow::Jump(self.state[reject].end);
                    (self.state[accept].start, self.state[reject].start)
                } else {
                    (self.state[accept].start, self.state[accept].end)
                };

                self.flow[select] = Flow::Choice(accept, reject);

                Ok(())
            }
            &Item::Loop { cond, body } => {
                let cond = self.expect_unnamed_block(cond, parent, Kind::Cond)?;
                let exit = self.placeholder();

                let body = self.expect_unnamed_block(body, parent, Kind::Loop)?;

                self.flow[exit] = Flow::Jump(self.state[body].end);
                self.flow.add(Flow::Jump(self.state[cond].start));

                Ok(())
            }
        }
    }

    fn expect_unnamed_block(
        &mut self,
        item: Handle<Item>,
        parent: Handle<State>,
        kind: Kind,
    ) -> Result<Handle<State>, SemanticError> {
        let Item::Block(None, body) = self.items.try_get(item)? else {
            return Err(SemanticError::ExpectUnnamedBlock(item));
        };
        self.block(None, kind, Some(parent), body.clone())
    }

    fn find_root(&self, mut parent: Handle<State>) -> Option<&State> {
        loop {
            if matches!(self.state[parent].kind, Kind::Root) {
                break Some(&self.state[parent]);
            } else if let Some(next) = self.state[parent].parent {
                parent = next
            } else {
                break None;
            }
        }
    }

    fn jump_target(
        &self,
        mut parent: Handle<State>,
        label: Option<&Name>,
    ) -> Result<&State, SemanticError> {
        loop {
            let cond = if let Some(label) = label {
                let name = self.state[parent].name.as_ref();
                name.is_some_and(|name| name == label)
            } else {
                matches!(self.state[parent].kind, Kind::Loop)
            };

            if cond {
                break Ok(&self.state[parent]);
            } else if let Some(next) = self.state[parent].parent {
                parent = next
            } else {
                break Err(SemanticError::LabelNotFound(label.cloned()));
            }
        }
    }
}

pub struct State {
    parent: Option<Handle<Self>>,
    start: Marker,
    end: Marker,

    name: Option<Name>,
    kind: Kind,
    range: Range<Flow>,
}

enum Kind {
    Root,
    Open,
    Loop,
    Cond,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum Ty {
    Literal(TyLiteral),
    ReferenceRef(Handle<Self>),
    ReferenceMut(Handle<Self>),
    Struct(Option<String>, Vec<Handle<Self>>),
    Tuple(Option<String>, Vec<Handle<Self>>),
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum TyLiteral {
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

    F32,
    F64,
}

pub enum Item {
    Block(Option<Name>, Range<Self>),

    Bind(Name, Option<Handle<Ty>>, Handle<Self>, bool),
    Assign(Handle<Self>, Handle<Self>),

    Continue(Option<Name>),
    Break(Option<Name>),

    Select {
        cond: Handle<Self>,
        accept: Handle<Self>,
        reject: Option<Handle<Self>>,
    },

    Loop {
        cond: Handle<Self>,
        body: Handle<Self>,
    },
}

impl<T: Eq + core::hash::Hash> Unique<T> {
    pub fn add(&mut self, value: T) -> Handle<T> {
        self.insert(value, Span)
    }
}

impl<T> Arena<T> {
    pub fn add(&mut self, value: T) -> Handle<T> {
        self.append(value, Span)
    }
}
