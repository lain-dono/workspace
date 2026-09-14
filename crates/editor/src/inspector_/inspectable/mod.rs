use super::InspectorUi;
use bevy::reflect::{Reflect, TypeRegistry};
use egui::{Id, Ui};

pub(crate) mod impl_bevy;
pub(crate) mod impl_glam;
pub(crate) mod impl_std;

pub type InspectRefFn = fn(&InspectorUi<'_>, &mut egui::Ui, egui::Id, &dyn Reflect) -> bool;
pub type InspectMutFn = fn(&InspectorUi<'_>, &mut egui::Ui, egui::Id, &mut dyn Reflect) -> bool;

#[derive(Clone)]
pub struct InspectableImpl {
    //fn_inspect_ref: InspectFn,
    pub fn_inspect_mut: InspectMutFn,
}

impl InspectableImpl {
    /*
    fn inspect_ref(&self, ctx: &InspectorUi, ui: &mut Ui, id: Id, r: &dyn Any) -> bool {
        (self.fn_inspect)(ctx, ui, id, r)
    }
    */

    pub fn inspect_mut(&self, ctx: &InspectorUi, ui: &mut Ui, id: Id, r: &mut dyn Reflect) -> bool {
        //dbg!("inspect_mut");
        (self.fn_inspect_mut)(ctx, ui, id, r)
    }
}

fn add<T: 'static>(registry: &mut TypeRegistry, fn_inspect_mut: InspectMutFn) {
    registry
        .get_mut(std::any::TypeId::of::<T>())
        .unwrap_or_else(|| panic!("{} not registered", std::any::type_name::<T>()))
        .insert(InspectableImpl { fn_inspect_mut });
}
