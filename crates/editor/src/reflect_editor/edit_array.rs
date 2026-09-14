use super::{errors, Arg};
use bevy::reflect::{Array, ArrayInfo, Reflect};

impl super::ReflectEditor<'_, '_, '_> {
    pub(crate) fn array_ref(&mut self, arg: Arg, array: &dyn Array) {
        arg.ui_array(array.len(), |arg, index| {
            self.reflect_ref(arg, array.get(index).unwrap());
            false
        });
    }

    pub(crate) fn array_mut(&mut self, arg: Arg, array: &mut dyn Array) -> bool {
        arg.ui_array(array.len(), |arg, index| {
            self.reflect_mut(arg, array.get_mut(index).unwrap())
        })
    }

    pub(crate) fn array_many(
        &mut self,
        arg: Arg,
        info: &ArrayInfo,
        _: &mut [&mut dyn Reflect],
        _: impl Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        let type_name = pretty_type_name::pretty_type_name_str(info.type_path());
        errors::no_multiedit(arg.ui, &type_name);
        false
    }
}

impl Arg<'_, '_> {
    fn ui_array(self, len: usize, mut f: impl FnMut(Arg, usize) -> bool) -> bool {
        let mut changed = false;
        self.ui.vertical_centered_justified(|ui| {
            for index in 0..len {
                ui.horizontal(|ui| {
                    changed |= f(Arg::new_with(ui, self.id, self.opt, index), index)
                });
                if index != len - 1 {
                    ui.separator();
                }
            }
        });
        changed
    }
}
