#![allow(dead_code, unused_variables)]

pub enum Instruction {
    SetMotiveChange {
        per_hr_owner: u8,
        stop_value_owner: u8,
        target_motive: u8,
        clear_all: u8,
        per_hr_data: u16,
        stop_value_data: u16,
    },
}

pub struct Script {
    instructions: Vec<Instruction>,
}

pub struct Machine {
    current: usize,
}

pub struct Context {
    caller: bevy::ecs::entity::Entity,
}

impl Context {
    fn variable_i(&self, variable: Variable) -> i16 {
        todo!()
    }

    fn variable_u(&self, variable: Variable) -> i16 {
        todo!()
    }

    fn set_variable_i(&self, variable: Variable, value: i16) {
        todo!()
    }

    fn set_variable_u(&self, variable: Variable, value: u16) {
        todo!()
    }

    fn caller<T>(&self) -> &T {
        todo!()
    }

    fn caller_mut<T>(&mut self) -> &mut T {
        todo!()
    }

    fn caller_motive_mut(&mut self, motive: MotiveId) -> &mut Motive {
        self.caller_mut::<Avatar>().motive_mut(motive)
    }
}

pub enum Code {
    GotoTrue = 0,
    GotoFalse = 1,

    GotoTrueNextTick = 2,
    GotoFalseNextTick = 3,

    ReturnTrue = 4,
    ReturnFalse = 5,

    Error = 6,
    ContinueNextTick = 7,

    Continue = 8, //used for primitives which change the control flow, don't quite return, more or idle yet.
    Interrupt = 9, //instantly ends this queue item. Used by Idle for Input with allow push: when any interactions are queued it exits out like this.
    ContinueFutureTick = 10, //special schedule mode used by idle and idle for input. removes processing for this object for multiple frames.
}

impl From<()> for Code {
    fn from(value: ()) -> Self {
        Self::GotoTrue
    }
}

impl From<bool> for Code {
    fn from(value: bool) -> Self {
        if value {
            Self::GotoTrue
        } else {
            Self::GotoFalse
        }
    }
}

fn calc(context: &mut Context, operand: [u8; 8]) -> Code {
    #[derive(Clone, Copy, bytemuck::AnyBitPattern)]
    struct Operand {
        op: u16,
        lhs: Variable,
        rhs: Variable,
    }

    let operand: Operand = bytemuck::cast(operand);

    let ls = context.variable_i(operand.lhs);
    let rs = context.variable_i(operand.rhs);
    let lu = ls as u16;
    let ru = rs as u16;

    #[allow(clippy::unit_arg)]
    match operand.op {
        0x00 => Code::from(ls < rs),
        0x01 => Code::from(lu < ru),
        0x02 => Code::from(ls <= rs),
        0x03 => Code::from(lu <= ru),
        0x04 => Code::from(ls > rs),
        0x05 => Code::from(lu > ru),
        0x06 => Code::from(ls >= rs),
        0x07 => Code::from(lu >= ru),
        0x08 => Code::from(ls == rs),
        0x09 => Code::from(ls != rs),

        0x0A => Code::from(context.set_variable_i(operand.lhs, ls + rs)),
        0x0B => Code::from(context.set_variable_i(operand.lhs, ls - rs)),

        0x10 => Code::from(context.set_variable_i(operand.lhs, ls * rs)),
        0x11 => Code::from(context.set_variable_u(operand.lhs, lu * ru)),
        0x12 => Code::from(context.set_variable_i(operand.lhs, ls / rs)),
        0x13 => Code::from(context.set_variable_u(operand.lhs, lu / ru)),
        0x14 => Code::from(context.set_variable_i(operand.lhs, ls % rs)),
        0x15 => Code::from(context.set_variable_u(operand.lhs, lu % ru)),

        0x20 => Code::from((lu & ru) != 0),
        0x21 => Code::from((lu & ru) == 0),

        0x22 => Code::from(context.set_variable_u(operand.lhs, lu | ru)),
        0x23 => Code::from(context.set_variable_u(operand.lhs, lu & ru)),

        0x24 => Code::from(context.set_variable_u(operand.lhs, lu | (1 << (ru & 0xF)))),
        0x25 => Code::from(context.set_variable_u(operand.lhs, lu & (1 << (ru & 0xF)))),

        _ => Code::GotoFalse,
    }
}

fn set_motive_change(context: &mut Context, operand: [u8; 8]) -> Code {
    #[derive(Clone, Copy, bytemuck::AnyBitPattern)]
    struct Operand {
        command: u8,
        motive: MotiveId,
        delta: Variable,
        limit: Variable,
    }

    let operand: Operand = bytemuck::cast(operand);
    let motive = operand.motive;

    match operand.command {
        0 => context.caller_mut::<Avatar>().clear_motive_changes(),
        1 => {
            let delta = context.variable_i(operand.delta);
            let limit = context.variable_i(operand.limit);
            context.caller_motive_mut(motive).tick_by(delta, limit);
        }
        2 => {
            let delta = context.variable_i(operand.delta);
            let limit = context.variable_i(operand.limit);
            context.caller_motive_mut(motive).set_change(delta, limit);
        }
        _ => (),
    }

    Code::GotoTrue
}

struct MotiveChange {
    delta: f32,
    limit: f32,
}

struct Motive {
    value: i16,
    min: i16,
    max: i16,
    change_delta: i16,
    change_limit: i16,
}

impl Motive {
    fn set(&mut self, value: i16) {
        self.value = value.clamp(self.min, self.max);
    }

    fn tick(&mut self) {
        self.tick_by(self.change_delta, self.change_limit);
    }

    fn tick_by(&mut self, delta: i16, limit: i16) {
        if delta > 0 && self.value > limit || delta < 0 && self.value < limit {
            return;
        }

        self.value += delta;

        if delta > 0 && self.value > limit || delta < 0 && self.value < limit {
            self.value = limit;
        }
    }

    fn set_change(&mut self, delta: i16, limit: i16) {
        self.change_delta = delta;
        self.change_limit = limit;
    }

    fn clear_change_rate(&mut self) {
        self.change_delta = 0;
        self.change_limit = i16::MAX;
    }
}

struct Avatar {
    motives: [Motive; 16],
}

impl Avatar {
    pub fn motive(&self, motive: MotiveId) -> &Motive {
        &self.motives[motive.index()]
    }

    pub fn motive_mut(&mut self, motive: MotiveId) -> &mut Motive {
        &mut self.motives[motive.index()]
    }

    pub fn has_motive_change(&mut self, motive: MotiveId) -> bool {
        self.motives[motive.index()].change_delta != 0
    }

    pub fn clear_motive_changes(&mut self) {
        for motive in &mut self.motives {
            motive.clear_change_rate();
        }
    }
}

#[derive(Clone, Copy, bytemuck::AnyBitPattern)]
#[repr(transparent)]
struct MotiveId(u8);

impl MotiveId {
    fn index(self) -> usize {
        (self.0 & 0x0F) as usize
    }
}

#[derive(Clone, Copy, bytemuck::AnyBitPattern)]
#[repr(transparent)]
struct Scope(u8);

#[derive(Clone, Copy, bytemuck::AnyBitPattern)]
#[repr(transparent)]
struct Variable([u8; 3]);

impl Variable {
    const fn new(index: u8, value: i16) -> Self {
        let [a, b] = value.to_le_bytes();
        Self([index, a, b])
    }

    const fn index(self) -> u8 {
        self.0[0]
    }

    const fn value(self) -> i16 {
        i16::from_le_bytes([self.0[1], self.0[2]])
    }
}
