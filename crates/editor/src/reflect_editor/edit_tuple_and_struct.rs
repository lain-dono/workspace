use super::Arg;
use bevy::reflect::{
    {Reflect, ReflectMut}, {Struct, StructInfo}, {Tuple, TupleInfo}, {TupleStruct, TupleStructInfo},
};

impl Arg<'_, '_> {
    pub fn ui_tuple(self, len: usize, mut f: impl FnMut(Arg, usize, bool) -> bool) -> bool {
        let Self { id, ui, opt } = self;
        let response = match len {
            0 => return false,
            1 => ui.vertical_centered_justified(|ui| f(Arg::field(ui, id, opt, 0), 0, false)),
            _ => {
                let grid = egui::Grid::new(self.id).num_columns(2);
                grid.show(ui, |ui| {
                    let mut changed = false;
                    for index in 0..len {
                        changed |= f(Arg::field(ui, id, opt, index), index, true);
                        ui.end_row();
                    }
                    changed
                })
            }
        };
        response.inner
    }

    pub fn ui_struct(self, len: usize, mut show: impl FnMut(Arg, usize, bool) -> bool) -> bool {
        let mut changed = false;
        let grid = egui::Grid::new(self.id).num_columns(2);
        grid.show(self.ui, |ui| {
            for index in 0..len {
                changed |= show(Arg::field(ui, self.id, self.opt, index), index, true);
                ui.end_row();
            }
        });
        changed
    }
}

impl super::ReflectEditor<'_, '_, '_> {
    pub(crate) fn tuple_struct_ref(&mut self, arg: Arg, value: &dyn TupleStruct) {
        arg.ui_tuple(value.field_len(), |mut arg, index, show_label| {
            arg.ui_field_name(show_label, || index.to_string());
            self.reflect_ref(arg, value.field(index).unwrap());
            false
        });
    }

    pub(crate) fn tuple_struct_mut(&mut self, arg: Arg, value: &mut dyn TupleStruct) -> bool {
        arg.ui_tuple(value.field_len(), |mut arg, index, show_label| {
            arg.ui_field_name(show_label, || index.to_string());
            self.reflect_mut(arg, value.field_mut(index).unwrap())
        })
    }

    pub(crate) fn tuple_struct_many(
        &mut self,
        arg: Arg,
        info: &TupleStructInfo,
        values: &mut [&mut dyn Reflect],
        projector: impl Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        arg.ui_tuple(info.field_len(), |mut arg, index, show_label| {
            let field = info.field_at(index).unwrap();
            arg.ui_field_name(show_label, || index.to_string());
            self.reflect_many(field.type_id(), field.type_path(), arg, values, &|r| {
                tuple_struct_field_at(projector(r), index)
            })
        })
    }

    pub(crate) fn struct_ref(&mut self, arg: Arg, value: &dyn Struct) {
        arg.ui_struct(value.field_len(), |mut arg, index, show_label| {
            arg.ui_field_name(show_label, || value.name_at(index).unwrap());
            self.reflect_ref(arg, value.field_at(index).unwrap());
            false
        });
    }

    pub(crate) fn struct_mut(&mut self, arg: Arg, value: &mut dyn Struct) -> bool {
        arg.ui_struct(value.field_len(), |mut arg, index, show_label| {
            arg.ui_field_name(show_label, || value.name_at(index).unwrap());
            self.reflect_mut(arg, value.field_at_mut(index).unwrap())
        })
    }

    pub(crate) fn struct_many(
        &mut self,
        arg: Arg,
        info: &StructInfo,
        values: &mut [&mut dyn Reflect],
        projector: impl Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        arg.ui_struct(info.field_len(), |mut arg, index, show_label| {
            let field = info.field_at(index).unwrap();
            arg.ui_field_name(show_label, || field.name());
            self.reflect_many(field.type_id(), field.type_path(), arg, values, &|r| {
                struct_field_at(projector(r), index)
            })
        })
    }

    pub(crate) fn tuple_ref(&mut self, arg: Arg, value: &dyn Tuple) {
        arg.ui_tuple(value.field_len(), |mut arg, index, show_label| {
            arg.ui_field_name(show_label, || index.to_string());
            self.reflect_ref(arg, value.field(index).unwrap());
            false
        });
    }

    pub(crate) fn tuple_mut(&mut self, arg: Arg, value: &mut dyn Tuple) -> bool {
        arg.ui_tuple(value.field_len(), |mut arg, index, show_label| {
            arg.ui_field_name(show_label, || index.to_string());
            self.reflect_mut(arg, value.field_mut(index).unwrap())
        })
    }

    pub(crate) fn tuple_many(
        &mut self,
        arg: Arg,
        info: &TupleInfo,
        values: &mut [&mut dyn Reflect],
        projector: impl Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        arg.ui_tuple(info.field_len(), |mut arg, index, show_label| {
            let field = info.field_at(index).unwrap();
            arg.ui_field_name(show_label, || index.to_string());
            self.reflect_many(field.type_id(), field.type_path(), arg, values, &|r| {
                tuple_field_at(projector(r), index)
            })
        })
    }
}

fn tuple_field_at(reflect: &mut dyn Reflect, index: usize) -> &mut dyn Reflect {
    match reflect.reflect_mut() {
        ReflectMut::Tuple(value) => value.field_mut(index).unwrap(),
        _ => unreachable!(),
    }
}

fn struct_field_at(reflect: &mut dyn Reflect, index: usize) -> &mut dyn Reflect {
    match reflect.reflect_mut() {
        ReflectMut::Struct(value) => value.field_at_mut(index).unwrap(),
        _ => unreachable!(),
    }
}

fn tuple_struct_field_at(reflect: &mut dyn Reflect, index: usize) -> &mut dyn Reflect {
    match reflect.reflect_mut() {
        ReflectMut::TupleStruct(value) => value.field_mut(index).unwrap(),
        _ => unreachable!(),
    }
}
