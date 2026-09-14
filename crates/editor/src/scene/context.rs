use crate::scene::{ReflectEntity, ReflectScene};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::reflect::{
    FromReflect, GetField, GetTupleField, GetTupleStructField, Reflect, ReflectMut, ReflectRef,
    TypeRegistryArc,
};

#[derive(Default, Resource)]
pub struct CurrentScene(pub Handle<ReflectScene>);

#[derive(SystemParam)]
pub struct SceneEditorContext<'w, 's> {
    pub current_scene: Res<'w, CurrentScene>,
    pub all_scenes: ResMut<'w, Assets<ReflectScene>>,

    pub types: Res<'w, AppTypeRegistry>,
    pub assets: ResMut<'w, AssetServer>,

    #[system_param(ignore)]
    marker: std::marker::PhantomData<&'s usize>,
}

impl<'w, 's> SceneEditorContext<'w, 's> {
    pub fn scene(&self) -> &ReflectScene {
        self.all_scenes.get(&self.current_scene.0).unwrap()
    }

    pub fn scene_mut(&mut self) -> &mut ReflectScene {
        self.all_scenes.get_mut(&self.current_scene.0).unwrap()
    }

    pub fn get(&mut self, index: usize) -> Option<EntityEditor> {
        self.all_scenes
            .get_mut(&self.current_scene.0)
            .unwrap()
            .entities
            .get_mut(index)
            .map(|entity| EntityEditor {
                index,
                entity,
                types: &self.types,
                assets: &mut self.assets,
            })
    }
}

#[derive(Deref, DerefMut)]
pub struct EntityEditor<'a> {
    pub index: usize,
    pub types: &'a TypeRegistryArc,
    #[deref]
    pub entity: &'a mut ReflectEntity,
    pub assets: &'a mut AssetServer,
}

impl<'a> EntityEditor<'a> {
    #[inline]
    pub fn has<T: Reflect>(&self) -> bool {
        let type_name = std::any::type_name::<T>();
        self.entity
            .components
            .iter()
            .any(|c| c.type_name() == type_name)
    }

    #[inline]
    pub fn without<T: Reflect>(&self) -> bool {
        let type_name = std::any::type_name::<T>();
        self.entity
            .components
            .iter()
            .all(|c| c.type_name() != type_name)
    }

    #[inline]
    pub fn children(&self) -> Option<&(dyn bevy::reflect::List + 'static)> {
        let type_name = std::any::type_name::<Children>();
        let children = self
            .entity
            .components
            .iter()
            .find(|c| c.type_name() == type_name)?;

        let reflect = children.reflect_ref();

        let reflect = if let ReflectRef::TupleStruct(reflect) = reflect {
            reflect
        } else {
            return None;
        };

        if let ReflectRef::List(list) = reflect.field(0)?.reflect_ref() {
            Some(list)
        } else {
            None
        }
    }
}

impl ReflectEntityGetters for ReflectEntity {
    fn component_ref<R: Reflect>(&self) -> Option<ReflectRef> {
        let type_name = std::any::type_name::<R>();
        self.components
            .iter()
            .find(|c| c.type_name() == type_name)
            .map(|r| r.reflect_ref())
    }

    fn component_mut<R: Reflect>(&mut self) -> Option<ReflectMut> {
        let type_name = std::any::type_name::<R>();
        self.components
            .iter_mut()
            .find(|c| c.type_name() == type_name)
            .map(|r| r.reflect_mut())
    }

    fn component_read<R: FromReflect + Sized>(&self) -> Option<R> {
        let type_name = std::any::type_name::<R>();
        self.components
            .iter()
            .find(|c| c.type_name() == type_name)
            .and_then(|r| R::from_reflect(r.as_ref()))
    }
}

pub trait ReflectEntityGetters {
    fn component_ref<R: Reflect>(&self) -> Option<ReflectRef>;
    fn component_mut<R: Reflect>(&mut self) -> Option<ReflectMut>;
    fn component_read<R: FromReflect>(&self) -> Option<R>;

    fn has<T: Reflect>(&self) -> bool {
        self.component_ref::<T>().is_some()
    }

    fn without<T: Reflect>(&self) -> bool {
        self.component_ref::<T>().is_none()
    }

    fn children(&self) -> Option<&(dyn bevy::reflect::List + 'static)> {
        let reflect = if let ReflectRef::TupleStruct(reflect) = self.component_ref::<Children>()? {
            reflect
        } else {
            return None;
        };
        if let ReflectRef::List(list) = reflect.field(0)?.reflect_ref() {
            Some(list)
        } else {
            None
        }
    }

    fn tuple_field_ref<R: Reflect, F: Reflect>(&self, index: usize) -> Option<&F> {
        match self.component_ref::<R>()? {
            ReflectRef::Tuple(s) => s.get_field(index),
            ReflectRef::TupleStruct(s) => s.get_field(index),
            _ => None,
        }
    }

    fn tuple_field_mut<R: Reflect, F: Reflect>(&mut self, index: usize) -> Option<&mut F> {
        match self.component_mut::<R>()? {
            ReflectMut::Tuple(s) => s.get_field_mut(index),
            ReflectMut::TupleStruct(s) => s.get_field_mut(index),
            _ => None,
        }
    }

    fn field_ref<R: Reflect, F: Reflect>(&self, name: &str) -> Option<&F> {
        match self.component_ref::<R>()? {
            ReflectRef::Struct(s) => s.get_field(name),
            _ => None,
        }
    }

    fn field_mut<R: Reflect, F: Reflect>(&mut self, name: &str) -> Option<&mut F> {
        match self.component_mut::<R>()? {
            ReflectMut::Struct(s) => s.get_field_mut(name),
            _ => None,
        }
    }
}

pub trait ReflectExt {
    fn field_ref<F: Reflect>(&self, name: &str) -> Option<&F>;
}

impl<'a> ReflectExt for ReflectRef<'a> {
    fn field_ref<F: Reflect>(&self, name: &str) -> Option<&F> {
        match self {
            ReflectRef::Struct(s) => s.get_field(name),
            _ => None,
        }
    }
}
