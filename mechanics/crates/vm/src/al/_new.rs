use core::{
    alloc::Layout,
    mem::{needs_drop, transmute},
    ptr::{NonNull, drop_in_place},
};

const _: () = assert!(core::mem::size_of::<usize>() == 8);
const _: () = assert!(core::mem::align_of::<usize>() == 8);

struct Vv {
    a: bevy::math::Vec3A,
    // b: std::sync::Arc<String>,
}

#[derive(Clone, Copy)]
pub struct Instruction(u16);

struct Command {
    command: CommandFn,
    layout: Layout,
}

pub struct CodeAlloc {
    opcodes: Vec<Command>,
    code: Option<NonNull<u8>>,
}

impl CodeAlloc {
    fn add_command(&mut self, command: CommandFn, layout: Layout) -> Instruction {
        let id = u16::try_from(self.opcodes.len()).unwrap();
        self.opcodes.push(Command { command, layout });
        Instruction(id)
    }

    fn add_instruction<T: Sized>(&mut self, instruction: Instruction, argument: T) {
        let Instruction(tag) = instruction;
        let command = &self.opcodes[tag as usize];
        assert_eq!(Layout::new::<T>(), command.layout);

        let drop_fn: Option<DropFn> = if needs_drop::<T>() {
            Some(|p: NonNull<u8>| unsafe { drop_in_place(p.cast::<T>().as_ptr()) })
        } else {
            None
        };

        let packed = pack_niche(tag, drop_fn);
    }
}

type CommandFn = fn(mem: NonNull<u8>);
type DropFn = fn(NonNull<u8>);

const SHIFT: usize = 48;
const MASK: usize = 0x0000_FFFF_FFFF_FFFF;

fn pack_niche(tag: u16, drop_fn: Option<DropFn>) -> usize {
    let ptr = drop_fn.map_or(0, |drop_fn| drop_fn as usize);
    assert!(ptr == ptr & MASK);
    ptr | (tag as usize) << SHIFT
}

unsafe fn unpack_niche(ptr: usize) -> (u16, Option<DropFn>) {
    let addr = ptr & 0x0000_FFFF_FFFF_FFFF;
    let drop_fn = unsafe { (addr != 0).then(|| transmute::<usize, fn(NonNull<u8>)>(ptr & MASK)) };
    ((ptr >> SHIFT) as u16, drop_fn)
}

#[test]
fn niche() {
    fn droper(_: NonNull<u8>) {}

    unsafe {
        let tag = 0x1234;
        let niche = pack_niche(tag, Some(droper));
        assert_eq!(tag, unpack_niche(niche).0);
    }
}
