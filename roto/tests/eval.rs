use roto::{
    FileTree, Library, Runtime,
    lir::{self, Memory},
    pipeline::LoweredToLir,
};
use std::sync::Arc;

macro_rules! src {
    ($code:literal) => {
        roto::FileTree::test_file(file!(), $code, line!() as usize - 1)
    };
}

#[track_caller]
fn compile(s: FileTree, rt: &Runtime) -> LoweredToLir<'_> {
    // We run this multiple times and only want to init the
    // first time, so ignore failures.
    #[cfg(feature = "logger")]
    let _ = env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .try_init();

    s.parse()
        .unwrap()
        .typecheck(rt)
        .unwrap()
        .lower_to_mir()
        .lower_to_lir()
}

/// Helper for constructing complex values for Roto
#[allow(unused)]
#[derive(Clone, Debug)]
enum MemVal {
    Struct(Vec<MemVal>),
    // Unfortunately, we can't (with this setup) compute the size and
    // alignment of an enum, so we have to pass that explicitly.
    Enum(u8, usize, usize, Option<Box<MemVal>>),
    Val(lir::Value),
}

#[allow(unused)]
impl MemVal {
    fn serialize(self, into: &mut Vec<u8>) {
        into.resize(self.padding(into.len()) + into.len(), 0);
        match self {
            MemVal::Struct(fields) => {
                for f in fields {
                    f.serialize(into);
                }
            }
            MemVal::Enum(d, size, _, v) => {
                let old_size = into.len();
                into.push(d);
                if let Some(v) = v {
                    v.serialize(into);
                }
                into.resize(old_size + size, 0);
            }
            MemVal::Val(v) => into.extend_from_slice(v.as_slice()),
        }
    }

    fn size(&self) -> usize {
        match self {
            MemVal::Struct(fields) => fields
                .iter()
                .fold(0, |size, f| size + f.padding(size) + f.size()),
            MemVal::Enum(_, size, _, _) => *size,
            MemVal::Val(v) => v.ty().size(),
        }
    }

    fn align(&self) -> usize {
        match self {
            MemVal::Struct(fields) => fields.iter().fold(1, |align, f| align.max(f.align())),
            MemVal::Enum(_, _, alignment, _) => *alignment,
            MemVal::Val(v) => v.ty().align(),
        }
    }

    fn padding(&self, offset: usize) -> usize {
        let align = self.align();
        align - (offset % align)
    }
}

#[test]
fn verdict() {
    {
        let s = src!("filtermap main(msg: u32) { accept }");

        let (mut mem, rt) = (Memory::default(), Runtime::builtin());
        let program = compile(s, &rt);
        let pointer = mem.allocate(1);
        program.eval(&mut mem, vec![lir::Value::Ptr(pointer), lir::Value::I32(0)]);
        let res = mem.read_array::<1>(pointer);
        assert_eq!(0, u8::from_ne_bytes(res));
    }

    {
        let s = src!("filtermap main() { reject }");

        let (mut mem, rt) = (Memory::default(), Runtime::builtin());
        let program = compile(s, &rt);
        let pointer = mem.allocate(1);
        program.eval(&mut mem, vec![lir::Value::Ptr(pointer)]);
        let res = mem.read_array::<1>(pointer);
        assert_eq!(1, u8::from_ne_bytes(res));
    }
}

#[test]
fn if_else() {
    let s = src!(
        "
        filtermap main() {
            if true && true {
                accept
            } else {
                reject
            }
        }
    "
    );
    let (mut mem, rt) = (Memory::default(), Runtime::builtin());
    let program = compile(s, &rt);
    let pointer = mem.allocate(1);
    program.eval(&mut mem, vec![lir::Value::Ptr(pointer)]);
    let res = mem.read_array::<1>(pointer);
    assert_eq!(0, u8::from_ne_bytes(res));
}

#[test]
fn react_to_rx() {
    let s = src!(
        "
        filtermap main(x: u32) {
            if x <= 4 {
                accept
            } else {
                reject
            }
        }
    "
    );

    let rt = Runtime::builtin();
    let program = compile(s, &rt);

    for i in 0..6 {
        let mut mem = Memory::default();
        let pointer = mem.allocate(1);
        program.eval(&mut mem, vec![lir::Value::Ptr(pointer), lir::Value::I32(i)]);
        let res = mem.read_array::<1>(pointer);
        assert_eq!(u8::from_ne_bytes(res), u8::from(i > 4), "failed at: {i}");
    }
}

#[test]
fn variable() {
    let s = src!(
        "
    filtermap main() {
        let a = 5;
        if a == 5 {
            accept
        } else {
            reject
        }
    }
    "
    );

    let (mut mem, rt) = (Memory::default(), Runtime::builtin());
    let program = compile(s, &rt);
    let pointer = mem.allocate(1);
    program.eval(&mut mem, vec![lir::Value::Ptr(pointer)]);
    let res = mem.read_array::<1>(pointer);
    assert_eq!(0, u8::from_ne_bytes(res));
}

#[test]
fn calling_function() {
    let s = src!(
        "
        fn smaller_than(a: u32, b: u32) -> bool {
            a < b
        }

        fn small(x: u32) -> bool {
            smaller_than(10, x) && smaller_than(x, 20)
        }

        filtermap main(msg: u32) {
            if small(msg) { accept }
            reject
        }
    "
    );

    let rt = Runtime::builtin();
    let program = compile(s, &rt);

    for x in 0..30 {
        let mut mem = Memory::default();
        let pointer = mem.allocate(1);
        program.eval(&mut mem, vec![lir::Value::Ptr(pointer), lir::Value::I32(x)]);
        let res = mem.read_array::<1>(pointer);
        assert_eq!(u8::from(!(10 < x && x < 20)), u8::from_ne_bytes(res));
    }
}

#[test]
fn anonymous_record() {
    let s = src!(
        "
        fn in_range(x: u32, low: u32, high: u32) -> bool {
            low < x && x < high
        }

        filtermap main(msg: u32) {
            let a = { low: 10, high: 20 };
            if in_range(msg, a.low, a.high) { accept }
            reject
        }
    "
    );

    let rt = Runtime::builtin();
    let program = compile(s, &rt);

    for x in 0..30 {
        let mut mem = Memory::default();
        let pointer = mem.allocate(1);
        program.eval(&mut mem, vec![lir::Value::Ptr(pointer), lir::Value::I32(x)]);
        let res = mem.read_array::<1>(pointer);
        assert_eq!(u8::from(!(10 < x && x < 20)), u8::from_ne_bytes(res));
    }
}

#[test]
fn typed_record() {
    let s = src!(
        "
        record Range {
            low: u32,
            high: u32,
        }

        fn in_range(x: u32, c: Range) -> bool {
            c.low < x && x < c.high
        }

        filtermap main(msg: u32) {
            let a = Range { low: 10, high: 20 };
            let b = Range { low: a.low, high: a.high };
            let c = b;
            if in_range(msg, c) { accept }
            reject
        }
    "
    );

    let rt = Runtime::builtin();
    let program = compile(s, &rt);

    for x in 0..1 {
        let mut mem = Memory::default();
        let pointer = mem.allocate(1);
        program.eval(&mut mem, vec![lir::Value::Ptr(pointer), lir::Value::I32(x)]);
        let res = mem.read_array::<1>(pointer);
        assert_eq!(u8::from_ne_bytes(res), u8::from(!(10 < x && x < 20)));
    }
}

#[test]
fn nested_record() {
    let s = src!(
        "
        record Foo { x: Bar, y: Bar }
        record Bar { a: i32, b: i32 }

        filtermap main(x: i32) {
            let bar = Bar { a: 20, b: x };
            let foo = Foo { x: bar, y: bar };
            if foo.x.a == foo.y.b {
                accept
            } else {
                reject
            }
        }
    "
    );

    let rt = Runtime::builtin();
    let program = compile(s, &rt);

    for x in 20..21 {
        let mut mem = Memory::default();
        let pointer = mem.allocate(1);
        program.eval(&mut mem, vec![lir::Value::Ptr(pointer), lir::Value::I32(x)]);
        let res = mem.read_array::<1>(pointer);
        assert_eq!(u8::from(x != 20), u8::from_ne_bytes(res), "for x = {x}");
    }
}

#[test]
fn call_runtime_function() {
    let s = src!(
        "
        filtermap main(x: u32) {
            if pow(x, 2) > 100 {
                accept
            } else {
                reject
            }
        }
    "
    );

    let mut lib = Library::default();
    lib.define_func("pow", "", ["x", "y"], |x: u32, y: u32| -> u32 { x.pow(y) });

    let rt = Runtime::from_lib(lib).unwrap();

    let program = compile(s, &rt);

    for (value, expected) in [(5, 1), (11, 0)] {
        let mut mem = Memory::default();
        let ptr = mem.allocate(1);
        program.eval(&mut mem, vec![lir::Value::Ptr(ptr), lir::Value::I32(value)]);
        let res = mem.read_array::<1>(ptr);
        assert_eq!(expected as u8, u8::from_ne_bytes(res));
    }
}

#[test]
fn u32_method() {
    let s = src!(
        "
        filtermap main(x: u32) {
            if x.is_even() {
                accept
            } else {
                reject
            }
        }
    "
    );

    let mut impl_block = roto::Impl::new::<u32>();
    impl_block.define_func("is_even", "", ["x"], |x: u32| -> bool {
        x.is_multiple_of(2)
    });

    let rt = Runtime::from_lib(Library::default().with(impl_block)).unwrap();
    let program = compile(s, &rt);

    for (value, expected) in [(5, 1), (6, 0)] {
        let mut mem = Memory::default();
        let ptr = mem.allocate(1);
        program.eval(&mut mem, vec![lir::Value::Ptr(ptr), lir::Value::I32(value)]);
        let res = mem.read_array::<1>(ptr);
        assert_eq!(expected, u8::from_ne_bytes(res));
    }
}

#[test]
fn string_global() {
    let s = src!(r#"fn main() -> bool { FOO == "BAR" }"#);

    let mut lib = roto::Library::default();
    lib.define_const::<Arc<str>>("FOO", "", "BAR".into());

    let rt = Runtime::from_lib(lib).unwrap();

    let p = compile(s, &rt);

    let mut mem = Memory::default();
    let res = p.eval(&mut mem, Vec::new()).unwrap();

    assert_eq!(res, lir::Value::Bool(true));
}
