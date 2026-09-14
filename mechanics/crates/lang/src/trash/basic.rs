use super::{Context, ScriptAccess, ScriptError, ScriptResult};
use bevy::{ptr::PtrMut, reflect::PartialReflect};

// pub fn nop(_: Context, _imm: PtrMut) -> ScriptResult {
//     Ok(())
// }

// pub fn push<T: PartialReflect>(mut ctx: Context, imm: PtrMut) -> ScriptResult {
//     ctx.push(unsafe { imm.deref_mut::<T>() });
//     Ok(())
// }

// pub fn dup(mut ctx: Context, _imm: PtrMut) -> ScriptResult {
//     let last = ctx.peek()?;
//     ctx.push(last);
//     Ok(())
// }

// pub fn access(mut ctx: Context, imm: PtrMut) -> ScriptResult {
//     let access = unsafe { imm.deref_mut::<ScriptAccess>() };
//     let base = ctx.pop()?;
//     let next = access.access(base)?;
//     ctx.push(next.ok_or_else(|| ScriptError::NotFound(access.clone()))?);
//     Ok(())
// }

// pub fn branch_if_true(mut ctx: Context, imm: PtrMut) -> ScriptResult {
//     let &cond = ctx.pop()?.try_downcast_ref().unwrap();
//     if cond {
//         ctx.jump(unsafe { *imm.deref_mut::<usize>() });
//     }
//     Ok(())
// }

// pub fn branch_if_false(mut ctx: Context, imm: PtrMut) -> ScriptResult {
//     let &cond = ctx.pop()?.try_downcast_ref::<bool>().unwrap();
//     if !cond {
//         ctx.jump(unsafe { *imm.deref_mut::<usize>() });
//     }
//     Ok(())
// }
