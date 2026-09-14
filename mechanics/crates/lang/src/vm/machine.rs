use super::memory::{MemoryBuilder, align_up};
use super::{ScriptError, ScriptResult, cast_command};
use bevy_ptr::PtrMut;
use std::{alloc::Layout, marker::PhantomData, mem::needs_drop, ptr::NonNull};

pub type Command = for<'a, 'b> unsafe fn(PtrMut<'b>, Context<'a>) -> ScriptResult;
pub type CommandCast<T> = for<'a, 'b> unsafe fn(&'b mut T, Context<'a>) -> ScriptResult;

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    func: Command,
    data: u32,
    next: u32,
}

impl Instruction {
    fn new(init: usize, func: Command, layout: Layout) -> Self {
        let data = align_up(init + size_of::<Self>(), layout.align());
        let next = align_up(data + layout.size(), align_of::<Self>());

        let data = u32::try_from(data).unwrap();
        let next = u32::try_from(next).unwrap();

        Self { func, data, next }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Cmd(u32);

impl Cmd {
    pub const PLACEHOLDER: Self = Self(u32::MAX);

    fn new(addr: usize) -> Self {
        Self(u32::try_from(addr).unwrap())
    }

    fn addr(self) -> usize {
        self.0 as usize
    }
}

#[repr(transparent)]
pub struct Imm<T>(pub(crate) u32, PhantomData<fn() -> T>);

impl<T> std::fmt::Debug for Imm<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let index = self.0;
        write!(f, "imm[{index}]")
    }
}

impl<T> Copy for Imm<T> {}
impl<T> Clone for Imm<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Imm<T> {
    pub const PLACEHOLDER: Self = Self(u32::MAX, PhantomData);

    fn new(addr: usize) -> Self {
        Self(u32::try_from(addr).unwrap(), PhantomData)
    }

    fn addr(self) -> usize {
        self.0 as usize
    }

    pub fn untyped(self) -> Imm<()> {
        Imm(self.0, PhantomData)
    }

    #[inline]
    pub unsafe fn cast<U>(self) -> Imm<U> {
        Imm(self.0, PhantomData)
    }

    #[inline]
    pub unsafe fn field<U>(self, offset: usize) -> Imm<U> {
        debug_assert!(offset < size_of::<T>());

        #[allow(clippy::cast_possible_truncation)]
        Imm((self.0 as usize + offset) as u32, PhantomData)
    }
}

#[derive(Debug, Clone)]
pub struct Machine {
    script: MemoryBuilder,
}

impl Default for Machine {
    fn default() -> Self {
        Self::new()
    }
}

impl Machine {
    pub const fn new() -> Self {
        Self {
            script: MemoryBuilder::new(),
        }
    }

    #[inline]
    unsafe fn get<T>(&self, offset: usize) -> NonNull<T> {
        unsafe { self.script.get::<T>(offset) }
    }

    #[inline]
    pub unsafe fn write<T>(&mut self, imm: Imm<T>, value: T) {
        unsafe { self.script.get::<T>(imm.addr()).write(value) }
    }

    #[inline]
    pub unsafe fn cmd_next(&self, offset: Cmd) -> Cmd {
        unsafe { Cmd(self.script.get::<Instruction>(offset.addr()).as_ref().next) }
    }

    /// # Panics
    /// if `T` needs drop
    pub fn emit<T>(&mut self, imm: T, command: CommandCast<T>) -> (Cmd, Imm<T>) {
        self.push_command(imm, unsafe { cast_command(command) })
    }

    pub unsafe fn place<T>(&mut self, command: CommandCast<T>) -> (Cmd, Imm<T>) {
        unsafe { self.place_cmd(cast_command(command)) }
    }

    /// # Panics
    /// if `T` needs drop
    pub fn push_command<T>(&mut self, imm: T, command: Command) -> (Cmd, Imm<T>) {
        assert!(!needs_drop::<T>());
        let imm_layout = Layout::new::<T>();
        unsafe {
            let cmd = |init| Instruction::new(init, command, imm_layout);
            let cmd = self.script.push_map(cmd);
            (Cmd::new(cmd), Imm::new(self.script.push(imm)))
        }
    }

    pub unsafe fn place_cmd<T>(&mut self, command: Command) -> (Cmd, Imm<T>) {
        let imm_layout = Layout::new::<T>();
        unsafe {
            let cmd = |init| Instruction::new(init, command, imm_layout);
            let cmd = self.script.push_map(cmd);
            (Cmd::new(cmd), Imm::new(self.script.alloc(imm_layout)))
        }
    }

    pub fn run(&mut self) -> ScriptResult {
        let mut jump = None;
        let mut offset = 0;
        while offset < self.script.len() {
            unsafe {
                let next = self.step(&mut jump, offset)?;
                offset = jump.take().unwrap_or(next);
            }
        }
        Ok(())
    }

    unsafe fn step(
        &mut self,
        jump: &mut Option<usize>,
        offset: usize,
    ) -> Result<usize, ScriptError> {
        unsafe {
            let cmd = self.get::<Instruction>(offset).as_ref();
            let imm = self.script.ptr_mut(cmd.data as usize);
            let script = &mut self.script;
            (cmd.func)(imm, Context { jump, script })?;
            Ok(cmd.next as usize)
        }
    }
}

pub struct Context<'a> {
    jump: &'a mut Option<usize>,
    script: &'a mut MemoryBuilder,
}

impl Context<'_> {
    pub fn jump(&mut self, addr: Cmd) {
        let _ = self.jump.insert(addr.0 as usize);
    }

    pub fn cond_jump(&mut self, cond: bool, addr: Cmd) {
        if cond {
            let _ = self.jump.insert(addr.0 as usize);
        }
    }

    #[inline]
    pub fn get_ref<'a, T>(&self, imm: Imm<T>) -> &'a T {
        unsafe { self.script.get::<T>(imm.addr()).as_ref() }
    }

    #[inline]
    pub fn get_mut<'a, T>(&mut self, imm: Imm<T>) -> &'a mut T {
        unsafe { self.script.get::<T>(imm.addr()).as_mut() }
    }

    #[inline]
    pub fn write<T>(&mut self, imm: Imm<T>, value: T) {
        unsafe { self.script.get::<T>(imm.addr()).write(value) }
    }

    /// # Safety
    /// no
    pub fn drop_in_place<T>(&mut self, imm: Imm<T>) {
        if needs_drop::<T>() {
            unsafe { self.script.get::<T>(imm.addr()).drop_in_place() }
        }
    }
}

#[test]
fn script() {
    let mut machine = Machine::new();

    let (cmd0, _) = machine.emit(77_usize, |&mut imm, mut ctx| {
        ctx.cond_jump(imm == 77, Cmd(77));
        Ok(())
    });

    let (cmd1, _) = machine.emit(88_usize, |&mut imm, mut ctx| {
        ctx.cond_jump(imm == 88, Cmd(88));
        Ok(())
    });

    let mut jump = None;

    let next = unsafe { machine.step(&mut jump, cmd0.0 as usize).unwrap() };
    assert_eq!(next, cmd1.0 as usize);
    assert_eq!(jump.take(), Some(77));

    let mut jump = None;
    let next = unsafe { machine.step(&mut jump, cmd1.0 as usize).unwrap() };
    assert!(next >= machine.script.len());
    assert_eq!(jump.take(), Some(88));
}
