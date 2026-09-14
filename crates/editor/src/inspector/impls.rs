//! UI implementations for leaf types

use crate::reflect_editor::{
    iter_all_eq, Arg, EditorVTableBuilder, Options, ReflectEditor, ReflectOptions,
};
use bevy::reflect::{TypeData, TypeInfo, TypeRegistry, VariantInfo};

pub mod asset_handle;
pub mod color;
pub mod entity;
pub mod image_handle;
pub mod math;
pub mod math_quat;
pub mod mesh_handle;
pub mod render_layer;

#[doc(hidden)]
macro_rules! many_ui {
    ( $ty:ty => $name:ident $inner:path ) => {
        pub fn $name(
            env: ReflectEditor,
            args: Arg,
            values: &mut [&mut dyn bevy::reflect::Reflect],
            projector: &dyn Fn(&mut dyn bevy::reflect::Reflect) -> &mut dyn bevy::reflect::Reflect,
        ) -> bool {
            let iter = values.iter_mut();
            let iter = iter.map(|value| projector(*value).downcast_ref::<$ty>().unwrap());
            let same = iter_all_eq(iter);

            let mut temp = same.cloned().unwrap_or_default();
            let changed = $inner(env, args, &mut temp);
            if changed {
                for value in values.iter_mut() {
                    *(projector(*value).downcast_mut::<$ty>().unwrap()) = temp.clone();
                }
            }
            changed
        }
    };
}

many_ui!(bool => bool_many self::math::bool_mut);
many_ui!(std::borrow::Cow<str> => cow_str_many self::math::cow_str_mut);
many_ui!(String => string_many self::math::string_mut);
many_ui!(bevy::render::color::Color => color_many self::color::color_mut);
many_ui!(bevy::render::view::RenderLayers => render_layers_many self::render_layer::render_layers_mut);
many_ui!(bevy::math::Quat => quat_many self::math_quat::quat_mut);

/// Register [`EditorVTable`]s for [`bevy::math`]/`glam` types
/// Register [`EditorVTable`]s for primitive rust types as well as standard library types
/// Register [`EditorVTable`]s for `bevy` types
pub fn register_all(registry: &mut TypeRegistry) {
    use self::color::*;
    use self::entity::*;
    use self::image_handle::*;
    use self::math::*;
    use self::math_quat::*;
    use self::mesh_handle::*;
    use self::render_layer::*;

    use bevy::math::{DMat2, DMat3, DMat4, DVec2, DVec3, DVec4, Mat3A, Vec3A};
    use bevy::prelude::*;
    use bevy::render::view::RenderLayers;

    let mut builder = EditorVTableBuilder::from(registry);

    builder.add_num::<f32>();
    builder.add_num::<f64>();

    builder.add_num::<i8>();
    builder.add_num::<i16>();
    builder.add_num::<i32>();
    builder.add_num::<i64>();
    builder.add_num::<isize>();

    builder.add_num::<u8>();
    builder.add_num::<u16>();
    builder.add_num::<u32>();
    builder.add_num::<u64>();
    builder.add_num::<usize>();

    builder.add_many::<bool>(bool_ref, bool_mut, bool_many);
    builder.add_many::<String>(string_ref, string_mut, string_many);
    builder.add_many::<std::borrow::Cow<str>>(cow_str_ref, cow_str_mut, cow_str_many);

    builder.add::<std::time::Duration>(duration_ref, duration_mut);
    builder.add::<std::time::Instant>(instant_ref, instant_mut);

    builder.add_editor::<ImageHandleEditor>();
    builder.add_editor::<MeshHandleEditor>();

    builder.add::<Entity>(entity_ref, entity_mut);

    builder.add_many::<Color>(color_ref, color_mut, color_many);
    builder.add_many::<RenderLayers>(render_layers_ref, render_layers_mut, render_layers_many);

    builder.add_many::<Vec2>(vec2_ref, vec2_mut, vec2_many);
    builder.add_many::<Vec3>(vec3_ref, vec3_mut, vec3_many);
    builder.add_many::<Vec4>(vec4_ref, vec4_mut, vec4_many);
    builder.add_many::<Vec3A>(vec3a_ref, vec3a_mut, vec3a_many);
    builder.add_many::<UVec2>(uvec2_ref, uvec2_mut, uvec2_many);
    builder.add_many::<UVec3>(uvec3_ref, uvec3_mut, uvec3_many);
    builder.add_many::<UVec4>(uvec4_ref, uvec4_mut, uvec4_many);
    builder.add_many::<IVec2>(ivec2_ref, ivec2_mut, ivec2_many);
    builder.add_many::<IVec3>(ivec3_ref, ivec3_mut, ivec3_many);
    builder.add_many::<IVec4>(ivec4_ref, ivec4_mut, ivec4_many);
    builder.add_many::<DVec2>(dvec2_ref, dvec2_mut, dvec2_many);
    builder.add_many::<DVec3>(dvec3_ref, dvec3_mut, dvec3_many);
    builder.add_many::<DVec4>(dvec4_ref, dvec4_mut, dvec4_many);

    builder.add::<Mat2>(mat2_ref, mat2_mut);
    builder.add::<Mat3>(mat3_ref, mat3_mut);
    builder.add::<Mat4>(mat4_ref, mat4_mut);
    builder.add::<Mat3A>(mat3a_ref, mat3a_mut);
    builder.add::<BVec2>(bvec2_ref, bvec2_mut);
    builder.add::<BVec3>(bvec3_ref, bvec3_mut);
    builder.add::<BVec4>(bvec4_ref, bvec4_mut);
    builder.add::<DMat2>(dmat2_ref, dmat2_mut);
    builder.add::<DMat3>(dmat3_ref, dmat3_mut);
    builder.add::<DMat4>(dmat4_ref, dmat4_mut);

    builder.add_many::<Quat>(quat_ref, quat_mut, quat_many);
}

pub fn register_default_options(registry: &mut TypeRegistry) {
    use self::math::NumberOptions;

    insert_options_enum::<bevy::render::color::Color>(
        registry,
        &[
            ("Rgba", "red", &NumberOptions::<f32>::normalized()),
            ("Rgba", "green", &NumberOptions::<f32>::normalized()),
            ("Rgba", "blue", &NumberOptions::<f32>::normalized()),
            ("Rgba", "alpha", &NumberOptions::<f32>::normalized()),
            ("RgbaLinear", "red", &NumberOptions::<f32>::normalized()),
            ("RgbaLinear", "green", &NumberOptions::<f32>::normalized()),
            ("RgbaLinear", "blue", &NumberOptions::<f32>::normalized()),
            ("RgbaLinear", "alpha", &NumberOptions::<f32>::normalized()),
            ("Hsla", "hue", &NumberOptions::<f32>::between(0.0, 360.0)),
            ("Hsla", "saturation", &NumberOptions::<f32>::normalized()),
            ("Hsla", "lightness", &NumberOptions::<f32>::normalized()),
            ("Hsla", "alpha", &NumberOptions::<f32>::normalized()),
        ],
    );

    let grading = NumberOptions::<f32>::positive().with_speed(0.01);
    insert_options_struct::<bevy::render::view::ColorGrading>(
        registry,
        &[
            ("exposure", &grading),
            ("gamma", &grading),
            ("pre_saturation", &grading),
            ("post_saturation", &grading),
        ],
    );

    //XXX #[cfg(feature = "bevy_pbr")]
    {
        insert_options_struct::<bevy::pbr::AmbientLight>(
            registry,
            &[("brightness", &NumberOptions::<f32>::normalized())],
        );
        insert_options_struct::<bevy::pbr::PointLight>(
            registry,
            &[
                ("intensity", &NumberOptions::<f32>::positive()),
                ("range", &NumberOptions::<f32>::positive()),
                ("radius", &NumberOptions::<f32>::positive()),
            ],
        );
        insert_options_struct::<bevy::pbr::DirectionalLight>(
            registry,
            &[("illuminance", &NumberOptions::<f32>::positive())],
        );
        insert_options_struct::<bevy::pbr::StandardMaterial>(
            registry,
            &[
                (
                    "perceptual_roughness",
                    &NumberOptions::<f32>::between(0.089, 1.0),
                ),
                ("metallic", &NumberOptions::<f32>::normalized()),
                ("reflectance", &NumberOptions::<f32>::normalized()),
                ("depth_bias", &NumberOptions::<f32>::positive()),
            ],
        );
        insert_options_enum::<bevy::pbr::ClusterConfig>(
            registry,
            &[
                ("FixedZ", "z_slices", &NumberOptions::<u32>::at_least(1)),
                (
                    "XYZ",
                    "dimensions",
                    &NumberOptions::<bevy::math::UVec3>::at_least(bevy::math::UVec3::ONE),
                ),
            ],
        );
    }

    insert_options_enum::<bevy::core_pipeline::core_3d::Camera3dDepthLoadOp>(
        registry,
        &[("Clear", "0", &NumberOptions::<f32>::normalized())],
    );

    insert_options_struct::<bevy::time::Virtual>(
        registry,
        &[
            ("relative_speed", &NumberOptions::<f32>::positive()),
            ("effective_speed", &NumberOptions::<f32>::positive()),
        ],
    );
}

fn insert_options_struct<T: 'static>(
    registry: &mut TypeRegistry,
    fields: &[(&'static str, &dyn TypeData)],
) {
    let Some(registration) = registry.get_mut(std::any::TypeId::of::<T>()) else {
        let name = std::any::type_name::<T>();
        return bevy::log::warn!("Attempting to set default inspector options for {name}, but it wasn't registered in the type registry.");
    };
    if registration.data::<ReflectOptions>().is_none() {
        let mut options = Options::default();
        for (field, data) in fields {
            let TypeInfo::Struct(info) = registration.type_info() else {
                unreachable!()
            };
            let field_index = info.index_of(field).unwrap();
            options.field(field_index, TypeData::clone_type_data(*data));
        }
        registration.insert(ReflectOptions(options));
    }
}

fn insert_options_enum<T: 'static>(
    registry: &mut TypeRegistry,
    fields: &[(&'static str, &'static str, &dyn TypeData)],
) {
    let Some(registration) = registry.get_mut(std::any::TypeId::of::<T>()) else {
        bevy::log::warn!("Attempting to set default inspector options for {}, but it wasn't registered in the type registry.", std::any::type_name::<T>());
        return;
    };

    if registration.data::<ReflectOptions>().is_some() {
        return;
    }

    let mut options = Options::default();
    for &(variant, field, data) in fields {
        let TypeInfo::Enum(info) = registration.type_info() else {
            unreachable!()
        };
        let variant_index = info.index_of(variant).unwrap();
        let field_index = match info.variant_at(variant_index).unwrap() {
            VariantInfo::Struct(info) => info.index_of(field).unwrap(),
            VariantInfo::Tuple(_) => field.parse().unwrap(),
            VariantInfo::Unit(_) => unreachable!(),
        };
        options.variant_field(variant_index, field_index, TypeData::clone_type_data(data));
    }
    registration.insert(ReflectOptions(options));
}
