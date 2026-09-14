use bevy::asset::{Asset, AssetId, Assets};
use bevy::ecs::component::ComponentId;
use bevy::ecs::world::{FilteredEntityRef, FilteredResourcesMut};
use bevy_def::{DefComponent, DefIndex, DefMut, DefRef};
use bevy_lang::semantic::Block;
use bevy_lang::vm::{Context, Imm, ScriptResult};

pub fn def_ref<T: DefComponent>(
    mut block: Block<'_>,
    res: Imm<FilteredResourcesMut<'static, 'static>>,
    index: ComponentId,
    asset: ComponentId,

    entity: Imm<FilteredEntityRef<'static>>,
    asset_id: AssetId<T::Asset>,
) -> (Imm<&'static T>, Imm<&'static T::Asset>) {
    struct Data<T: DefComponent> {
        res: Imm<FilteredResourcesMut<'static, 'static>>,
        index: ComponentId,
        asset: ComponentId,

        entity: Imm<FilteredEntityRef<'static>>,
        def_ref: Imm<DefRef<'static, 'static, T>>,
        asset_id: AssetId<T::Asset>,
    }

    fn command<T: DefComponent>(data: &mut Data<T>, mut ctx: Context<'_>) -> ScriptResult {
        let res = ctx.get_ref(data.res);
        let asset: &Assets<T::Asset> = unsafe { res.get_by_id(data.asset).unwrap().deref() };
        let asset = asset.get(data.asset_id).unwrap();

        let index: &DefIndex<T> = unsafe { res.get_by_id(data.index).unwrap().deref() };
        let entity = ctx.get_ref(data.entity);
        let component_id = index.asset_to_id().get(&data.asset_id).copied().unwrap();
        let value = unsafe { entity.get_by_id(component_id).unwrap().deref() };
        ctx.write(data.def_ref, DefRef { value, asset });
        Ok(())
    }

    let def_ref = block.variable("def_ref");

    let imm = Data {
        res,
        index,
        asset,

        entity,
        def_ref,
        asset_id,
    };

    let _ = block.emit(imm, command);

    unsafe { access_def_ref(def_ref) }
}

pub unsafe fn access_def_ref<T: DefComponent>(
    base: Imm<DefRef<'static, 'static, T>>,
) -> (Imm<&'static T>, Imm<&'static T::Asset>) {
    (
        unsafe { base.field(std::mem::offset_of!(DefRef<'static, 'static, T>, value)) },
        unsafe { base.field(std::mem::offset_of!(DefRef<'static, 'static, T>, asset)) },
    )
}

pub unsafe fn access_def_mut<T: DefComponent>(
    base: Imm<DefMut<'static, 'static, T>>,
) -> (Imm<&'static mut T>, Imm<&'static T::Asset>) {
    (
        unsafe { base.field(std::mem::offset_of!(DefMut<'static, 'static, T>, value)) },
        unsafe { base.field(std::mem::offset_of!(DefMut<'static, 'static, T>, asset)) },
    )
}

pub fn read_def_ref<T: DefComponent>(
    mut block: Block<'_>,
    res: Imm<FilteredResourcesMut<'static, 'static>>,
    asset: ComponentId,
    entity: Imm<FilteredEntityRef<'static>>,
    asset_id: AssetId<T::Asset>,
    component_id: ComponentId,
) -> (Imm<&'static T>, Imm<&'static T::Asset>) {
    struct Data<T: DefComponent> {
        res: Imm<FilteredResourcesMut<'static, 'static>>,
        asset: ComponentId,
        entity: Imm<FilteredEntityRef<'static>>,
        def_ref: Imm<DefRef<'static, 'static, T>>,
        asset_id: AssetId<T::Asset>,
        component_id: ComponentId,
    }

    fn command<T: DefComponent>(data: &mut Data<T>, mut ctx: Context<'_>) -> ScriptResult {
        let res = ctx.get_ref(data.res);
        let asset: &Assets<T::Asset> = unsafe { res.get_by_id(data.asset).unwrap().deref() };
        let asset = asset.get(data.asset_id).unwrap();

        let entity = ctx.get_ref(data.entity);
        let value = unsafe { entity.get_by_id(data.component_id).unwrap().deref() };
        ctx.write(data.def_ref, DefRef { value, asset });
        Ok(())
    }

    let def_ref = block.variable("def_ref");

    let imm = Data {
        res,
        asset,
        entity,
        def_ref,
        asset_id,
        component_id,
    };

    let _ = block.emit(imm, command);

    unsafe { access_def_ref(def_ref) }
}

pub fn read_def_component<T: DefComponent>(
    mut block: Block<'_>,
    entity: Imm<FilteredEntityRef<'static>>,
    component_id: ComponentId,
) -> Imm<&'static T> {
    struct Data<T: DefComponent> {
        place: *const T,
        entity: Imm<FilteredEntityRef<'static>>,
        component_id: ComponentId,
    }

    fn command<T: DefComponent>(data: &mut Data<T>, ctx: Context<'_>) -> ScriptResult {
        let entity = ctx.get_ref(data.entity);
        data.place = unsafe { entity.get_by_id(data.component_id).unwrap().deref() };
        Ok(())
    }

    let imm = Data::<T> {
        place: core::ptr::null(),
        entity,
        component_id,
    };

    let (_, imm) = block.emit(imm, command);

    unsafe { imm.field(std::mem::offset_of!(Data<T>, place)) }
}

pub fn read_asset<T: Asset>(
    mut block: Block<'_>,
    res: Imm<FilteredResourcesMut<'static, 'static>>,
    asset: ComponentId,
    asset_id: AssetId<T>,
) -> Imm<&'static T> {
    struct Data<T: Asset> {
        place: *const T,
        res: Imm<FilteredResourcesMut<'static, 'static>>,
        asset: ComponentId,
        asset_id: AssetId<T>,
    }

    fn command<T: Asset>(data: &mut Data<T>, ctx: Context<'_>) -> ScriptResult {
        let res = ctx.get_ref(data.res);
        let asset: &Assets<T> = unsafe { res.get_by_id(data.asset).unwrap().deref() };
        data.place = asset.get(data.asset_id).unwrap();
        Ok(())
    }

    let imm = Data::<T> {
        place: core::ptr::null(),
        res,
        asset,
        asset_id,
    };

    let (_, imm) = block.emit(imm, command);

    unsafe { imm.field(std::mem::offset_of!(Data<T>, place)) }
}
