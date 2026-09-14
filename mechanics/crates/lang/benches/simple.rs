use bevy::reflect::Reflect;
use bevy_lang::vm::{Machine, ScriptAccess, ScriptError};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut script = Machine::new();

    #[derive(Reflect, Clone, Copy)]
    struct Something {
        value: i32,
    }

    {
        let (_, s_var) = script.init_var(Something { value: 5 });

        let (_, lhs) = script.place_var::<i32>();
        let (_, rhs) = script.init_var(7_i32);

        let access = ScriptAccess::field("value");
        script.reflect_access::<Something, i32>(s_var, access, lhs);

        let (_, output) = script.place_var::<i32>();

        script.emit((lhs, rhs, output), |&mut (lhs, rhs, output), mut ctx| {
            let lhs = ctx.get_ref(lhs);
            let rhs = ctx.get_ref(rhs);
            ctx.write(output, std::ops::Add::add(*lhs, *rhs));
            Ok(())
        });

        // script.debug_var::<i32>(output);
    }

    #[inline(never)]
    fn native(src: Something) -> Result<i32, ScriptError> {
        let access = ScriptAccess::field("value");

        let Some(next) = access.access(&src)? else {
            return Err(ScriptError::NotFound(access.clone()));
        };

        let Some(next) = next.try_downcast_ref::<i32>() else {
            return Err(ScriptError::TypeMismatch);
        };

        let lhs: i32 = *next;
        let rhs: i32 = 7_i32;

        Ok(lhs + rhs)
    }

    #[inline(never)]
    fn native_no_reflect(src: Something) -> Result<i32, ScriptError> {
        let lhs: i32 = src.value;
        let rhs: i32 = 7_i32;
        Ok(lhs + rhs)
    }

    c.bench_function("native", |b| {
        b.iter(|| {
            let input = black_box(Something { value: 5 });
            let result = native(input);
            let _ = black_box(result);
        })
    });

    c.bench_function("native_no_reflect", |b| {
        b.iter(|| {
            let input = black_box(Something { value: 5 });
            let result = native_no_reflect(input);
            let _ = black_box(result);
        })
    });

    c.bench_function("script", |b| {
        b.iter(|| {
            let _ = black_box(script.run());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
