use crate::stat::{Stat, StatAsset};
use bevy::asset::{AssetId, Assets};
use bevy::ecs::{
    component::{ComponentId, Components},
    system::System,
    world::{FilteredResourcesMut, World},
};
use bevy_def::DefIndex;
use bevy_lang::{
    arena::Handle,
    semantic::Block,
    vm::{Imm, Machine, ScriptAccess},
};

pub mod asm;
pub mod def;
pub mod ir;

fn resource_ref(
    machine: &mut Machine,
    id: ComponentId,
    res: Imm<FilteredResourcesMut<'static, 'static>>,
) -> Imm<*const u8> {
    use bevy_lang::vm;

    struct Data {
        id: ComponentId,
        res: vm::Imm<FilteredResourcesMut<'static, 'static>>,
        pointer: *const u8,
    }

    unsafe fn command(imm: &mut Data, ctx: vm::Context) -> vm::ScriptResult {
        let resources = ctx.get_ref(imm.res);
        imm.pointer = resources.get_by_id(imm.id).unwrap().as_ptr();
        Ok(())
    }

    let pointer = std::ptr::null();
    let imm = Data { id, res, pointer };

    let (_, imm) = machine.emit(imm, command);
    unsafe { imm.field(std::mem::offset_of!((ComponentId, u32, u32), 2)) }
}

fn resource_mut(
    machine: &mut Machine,
    id: ComponentId,
    res: Imm<FilteredResourcesMut<'static, 'static>>,
) -> Imm<*const u8> {
    use bevy_lang::vm;

    struct Data {
        id: ComponentId,
        res: vm::Imm<FilteredResourcesMut<'static, 'static>>,
        pointer: *const u8,
    }

    unsafe fn command(imm: &mut Data, ctx: vm::Context) -> vm::ScriptResult {
        let resources = ctx.get_ref(imm.res);
        imm.pointer = resources.get_by_id(imm.id).unwrap().as_ptr();
        Ok(())
    }

    let pointer = std::ptr::null();
    let imm = Data { id, res, pointer };

    let (_, imm) = machine.emit(imm, command);
    unsafe { imm.field(std::mem::offset_of!((ComponentId, u32, u32), 2)) }
}

pub fn print_system(
    world: &mut World,
    asset_id: AssetId<StatAsset>,
    component_id: ComponentId,
) -> impl System<In = (), Out = ()> {
    let (system, module) = new_print_system(world.components(), component_id);
    asm::build_system(world, &module, system)
}

fn new_print_system(
    components: &Components,
    component_id: ComponentId,
) -> (Handle<ir::SystemDecl>, ir::Module) {
    let mut b = ir::Module::default();

    let f32 = b.ty(ir::Type::Primitive(ir::Primitive::F32));
    // let usize = b.ty(ir::Type::Primitive(ir::Primitive::Usize));
    // let string = b.ty(ir::Type::Struct(vec![usize, usize, usize]));

    assert_eq!(size_of::<String>(), size_of::<usize>() * 3);

    let stat_component = b.component(component_id, ir::Type::Struct(vec![f32]));
    // let _stat_asset = b.asset("StatAsset", ir::Type::Struct(vec![string, f32, f32, f32]));

    let mut system = b.system_builder();

    let system = {
        let assets_stat_asset_id = components.resource_id::<Assets<StatAsset>>().unwrap();
        let def_index_stat_id = components.resource_id::<DefIndex<Stat>>().unwrap();

        let arg_query = system.arg_query_ref("query", [stat_component]);
        let _arg_asset = system.arg_resource_opaque("asset", assets_stat_asset_id);
        let _arg_index = system.arg_resource_opaque("index", def_index_stat_id);

        let init = system.label();
        let exit = system.label();

        let (item, _) = system.var_ty(ir::Type::FilteredEntityRef);

        system.mark(init);
        system.next_ref(exit, item, arg_query);

        let value = system.ref_component_var(item, stat_component);
        system.debug(value);

        system.drop(item);
        system.jump(init);
        system.mark(exit);
        system.build("print_system")
    };

    (b.system(system), b)
}

/*
let [res_index_id, res_asset_id]: [ComponentId; 2] = [
    generator.resources[&arg_index.ty],
    generator.resources[&arg_asset.ty],
];

let mut semantic = Semantic::default();

let (imm_res, imm_iter) = semantic.root(|mut block| {
    let res = block.variable::<FilteredResourcesMut>("resources");
    let iter = block.variable::<QueryIter<FilteredEntityRef, ()>>("iter");

    block.foreach("entity", iter, |mut block, entity| {
        let asset = def::read_asset::<StatAsset>(block.reborrow(), res, res_asset_id, asset_id);
        let value = def::read_def_component::<Stat>(block.reborrow(), entity, component_id);

        block.debug_variable(value);
        block.debug_variable(asset);
    });

    sample(&mut block);

    (res, iter)
});

let machine = semantic.build();
*/

pub fn sample(block: &mut Block) {
    #[derive(bevy::reflect::Reflect, Clone, Copy)]
    struct Something {
        value: i32,
    }

    let s_var = block.variable_init("some", Something { value: 5 });

    let lhs = block.variable::<i32>("lhs");
    let rhs = block.variable_init("rhs", 7_i32);

    block.reflect_access::<Something, i32>(s_var, ScriptAccess::field("value"), lhs);

    let output = block.variable::<i32>("rhs");
    block.add::<i32, i32, i32>(lhs, rhs, output);
    block.debug_variable::<i32>(output);

    // script.loop_range(0..3, |script| {
    //     script.print("{[0]} ---");
    // });

    // script.read_stat("defs", "entity", "stat", asset_id);
    // script.print("{stat.defname}: {stat.current} [{stat.minimal} .. {stat.maximal}]");
    // let (_, leet) = script.init_var(1337_i32);
    // script.debug_var::<i32>(leet);
}

#[test]
fn layout_builder() {
    use std::alloc::{Layout, LayoutError};

    struct TyLayoutBuilder {
        layout: Layout,
    }

    impl TyLayoutBuilder {
        const fn new() -> Self {
            Self {
                layout: unsafe { Layout::from_size_align_unchecked(0, 1) },
            }
        }

        const fn field(&mut self, layout: Layout) -> Result<usize, LayoutError> {
            match self.layout.extend(layout) {
                Ok((new_layout, offset)) => {
                    self.layout = new_layout;
                    Ok(offset)
                }
                Err(err) => Err(err),
            }
        }

        const fn build(self) -> Layout {
            self.layout.pad_to_align()
        }
    }

    let (layout, foo, bar) = const {
        let mut layout = TyLayoutBuilder::new();
        let foo = layout.field(Layout::new::<u32>());
        let bar = layout.field(Layout::new::<u32>());
        (layout.build(), foo, bar)
    };

    assert_eq!(layout, Layout::new::<[u32; 2]>());
    assert_eq!(foo.unwrap(), 0);
    assert_eq!(bar.unwrap(), 4);
}
