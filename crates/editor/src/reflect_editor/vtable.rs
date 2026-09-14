use super::{errors, AnyOptions, ReflectEditor};
use bevy::reflect::{Reflect, TypeRegistry};
use std::any::{Any, TypeId};

#[derive(bevy::prelude::Deref, bevy::prelude::DerefMut)]
pub struct Arg<'ui, 'opt> {
    #[deref]
    pub ui: &'ui mut egui::Ui,
    pub id: egui::Id,
    pub opt: AnyOptions<'opt>,
}

impl<'ui, 'opt> Arg<'ui, 'opt> {
    pub(crate) fn with_opt<'b>(self, opt: AnyOptions<'b>) -> Arg<'ui, 'b> {
        Arg { opt, ..self }
    }

    pub(crate) fn with_ui<'b>(self, ui: &'b mut egui::Ui) -> Arg<'b, 'opt> {
        Arg { ui, ..self }
    }

    pub(crate) fn id(id: egui::Id, ui: &'ui mut egui::Ui) -> Self {
        let opt = AnyOptions::EMPTY;
        Self { id, ui, opt }
    }

    pub(crate) fn null(ui: &'ui mut egui::Ui) -> Self {
        let id = egui::Id::null();
        let opt = AnyOptions::EMPTY;
        Self { id, ui, opt }
    }

    pub(crate) fn new_with<'a: 'ui + 'opt>(
        ui: &'ui mut egui::Ui,
        id: egui::Id,
        opt: AnyOptions<'opt>,
        child: impl std::hash::Hash,
    ) -> Self {
        Self {
            id: id.with(child),
            ui,
            opt,
        }
    }

    pub(crate) fn with<'a: 'ui + 'opt>(self, child: impl std::hash::Hash) -> Self {
        Self {
            id: self.id.with(child),
            ..self
        }
    }

    pub(crate) fn field<'a: 'ui + 'opt>(
        ui: &'ui mut egui::Ui,
        id: egui::Id,
        opt: AnyOptions<'opt>,
        field_index: usize,
    ) -> Self {
        Self {
            id: id.with(field_index),
            ui,
            opt: opt.field(field_index),
        }
    }

    pub(crate) fn variant_field<'a: 'ui + 'opt>(
        ui: &'ui mut egui::Ui,
        id: egui::Id,
        opt: AnyOptions<'opt>,
        variant_index: usize,
        field_index: usize,
    ) -> Self {
        Self {
            id: id.with((variant_index, field_index)),
            ui,
            opt: opt.variant_field(variant_index, field_index),
        }
    }

    pub fn ui_field_name<T>(&mut self, show: bool, text: impl FnOnce() -> T)
    where
        T: Into<egui::WidgetText>,
    {
        if show {
            self.ui.label(text());
        }
    }
}

pub type EditorFnRef = fn(ReflectEditor, Arg, &dyn Any);
pub type EditorFnMut = fn(ReflectEditor, Arg, &mut dyn Any) -> bool;

pub type EditorFnMany = fn(
    ReflectEditor,
    Arg,
    &mut [&mut dyn Reflect],
    &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
) -> bool;

pub trait FieldEditor {
    type Target: Any;

    fn edit_ref(editor: ReflectEditor, arg: Arg, value: &Self::Target);
    fn edit_mut(editor: ReflectEditor, arg: Arg, value: &mut Self::Target) -> bool;

    fn edit_many(
        _editor: ReflectEditor,
        args: Arg,
        _values: &mut [&mut dyn Reflect],
        _projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        let type_name = pretty_type_name::pretty_type_name::<Self::Target>();
        errors::no_multiedit(args.ui, &type_name);
        false
    }
}

/// Function pointers for displaying a concrete type, to be registered in the [`TypeRegistry`].
///
/// This can used for leaf types like `u8` or `String`, as well as people who want to completely customize the way
/// to display a certain type.
#[derive(Clone)]
pub struct EditorVTable {
    fn_ref: EditorFnRef,
    fn_mut: EditorFnMut,
    fn_many: EditorFnMany,
}

impl EditorVTable {
    /// Create a new [`EditorVTable`] from functions displaying a type
    fn new(fn_ref: EditorFnRef, fn_mut: EditorFnMut, fn_many: EditorFnMany) -> Self {
        Self {
            fn_ref,
            fn_mut,
            fn_many,
        }
    }

    pub(super) fn ui_ref(&self, env: ReflectEditor, args: Arg, value: &dyn Reflect) {
        (self.fn_ref)(env, args, value.as_any())
    }

    pub(super) fn ui_mut(&self, env: ReflectEditor, args: Arg, value: &mut dyn Reflect) -> bool {
        (self.fn_mut)(env, args, value.as_any_mut())
    }

    pub(super) fn ui_many(
        &self,
        env: ReflectEditor,
        args: Arg,
        values: &mut [&mut dyn Reflect],
        projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        (self.fn_many)(env, args, values, projector)
    }

    fn no_many<T: Any>(
        _: ReflectEditor,
        args: Arg,
        _: &mut [&mut dyn Reflect],
        _: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        errors::no_multiedit(args.ui, &pretty_type_name::pretty_type_name::<T>());
        false
    }
}

pub struct EditorVTableBuilder<'a> {
    registry: &'a mut TypeRegistry,
}

impl<'a> From<&'a mut TypeRegistry> for EditorVTableBuilder<'a> {
    fn from(registry: &'a mut TypeRegistry) -> Self {
        Self { registry }
    }
}

impl EditorVTableBuilder<'_> {
    pub fn add<T: Any>(&mut self, fn_ref: EditorFnRef, fn_mut: EditorFnMut) {
        self.add_many::<T>(fn_ref, fn_mut, EditorVTable::no_many::<T>);
    }

    pub fn add_editor<T: FieldEditor>(&mut self) {
        pub fn vtable_for<E: FieldEditor>() -> EditorVTable {
            fn fn_ref<E: FieldEditor>(e: ReflectEditor, arg: Arg, value: &dyn Any) {
                E::edit_ref(e, arg, value.downcast_ref::<E::Target>().unwrap())
            }

            fn fn_mut<E: FieldEditor>(e: ReflectEditor, arg: Arg, value: &mut dyn Any) -> bool {
                E::edit_mut(e, arg, value.downcast_mut::<E::Target>().unwrap())
            }

            EditorVTable {
                fn_ref: fn_ref::<E>,
                fn_mut: fn_mut::<E>,
                fn_many: E::edit_many,
            }
        }

        let ty = self.registry.get_mut(TypeId::of::<T::Target>());
        let ty = ty.unwrap_or_else(|| panic!("{} not registered", std::any::type_name::<T>()));
        ty.insert(vtable_for::<T>());
    }

    pub fn add_many<T: Any>(
        &mut self,
        fn_ref: EditorFnRef,
        fn_mut: EditorFnMut,
        fn_many: EditorFnMany,
    ) {
        let ty = self.registry.get_mut(TypeId::of::<T>());
        let ty = ty.unwrap_or_else(|| panic!("{} not registered", std::any::type_name::<T>()));
        ty.insert(EditorVTable::new(fn_ref, fn_mut, fn_many));
    }
}

/// Function which will be executed for every field recursively, which can be used to skip regular traversal, `_readonly` variant
///
/// This can be used to recognize `Handle<T>` types and display them as their actual value instead.
/// Returning `None` means that no short circuiting is required, and `Some(changed)` means that the value was short-circuited
/// and changed if the boolean is true.
pub type ShortCircuitFnRef = fn(ReflectEditor, Arg, value: &dyn Reflect) -> Option<()>;

/// Function which will be executed for every field recursively, which can be used to skip regular traversal.
///
/// This can be used to recognize `Handle<T>` types and display them as their actual value instead.
/// Returning `None` means that no short circuiting is required, and `Some(changed)` means that the value was short-circuited
/// and changed if the boolean is true.
pub type ShortCircuitFnMut = fn(ReflectEditor, Arg, value: &mut dyn Reflect) -> Option<bool>;

/// Function which will be executed for every field recursively, which can be used to skip regular traversal, `_many` variant
///
/// This can be used to recognize `Handle<T>` types and display them as their actual value instead.
/// Returning `None` means that no short circuiting is required, and `Some(changed)` means that the value was short-circuited
/// and changed if the boolean is true.
pub type ShortCircuitFnMany = fn(
    ReflectEditor,
    Arg,
    type_id: TypeId,
    type_name: &str,
    values: &mut [&mut dyn Reflect],
    projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
) -> Option<bool>;

#[derive(Clone, Copy)]
pub struct ShortCircuit {
    /// Same as [`short_circuit`](InspectorUi::short_circuit), but for read only usage.
    pub fn_ref: ShortCircuitFnRef,
    /// Function which will be executed for every field recursively, which can be used to skip regular traversal.
    /// This can be used to recognize `Handle<T>` types and display them as their actual value instead.
    pub fn_mut: ShortCircuitFnMut,
    pub fn_many: ShortCircuitFnMany,
}

impl Default for ShortCircuit {
    fn default() -> Self {
        Self {
            fn_ref: |_, _, _| None,
            fn_mut: |_, _, _| None,
            fn_many: |_, _, _, _, _, _| None,
        }
    }
}

impl ShortCircuit {
    pub fn new_opt(
        fn_ref: Option<ShortCircuitFnRef>,
        fn_mut: Option<ShortCircuitFnMut>,
        fn_many: Option<ShortCircuitFnMany>,
    ) -> Self {
        Self {
            fn_ref: fn_ref.unwrap_or(|_, _, _| None),
            fn_mut: fn_mut.unwrap_or(|_, _, _| None),
            fn_many: fn_many.unwrap_or(|_, _, _, _, _, _| None),
        }
    }
}
