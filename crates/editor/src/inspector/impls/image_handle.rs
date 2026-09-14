use crate::reflect_editor::{Arg, FieldEditor, ReflectEditor};
use bevy::asset::{Assets, Handle};
use bevy::render::{
    render_resource::{Extent3d, TextureDimension, TextureFormat},
    texture::{Image, TextureFormatPixelInfo},
};
use bevy_egui::EguiUserTextures;
use egui::load::SizedTexture;
use image::{DynamicImage, ImageBuffer};
use once_cell::sync::Lazy;
use std::collections::{hash_map::Entry, HashMap, HashSet};
use std::sync::Mutex;

pub struct ImageHandleEditor;

impl FieldEditor for ImageHandleEditor {
    type Target = Handle<Image>;

    fn edit_ref(env: ReflectEditor, Arg { ui, .. }: Arg, value: &Self::Target) {
        let egui_user_textures = unsafe { env.get_resource_mut::<bevy_egui::EguiUserTextures>() };
        let images = unsafe { env.get_resource_mut::<Assets<Image>>() };

        let mut egui_user_textures = match egui_user_textures {
            Ok(egui_user_textures) => egui_user_textures,
            Err(error) => return error.show_typed::<bevy_egui::EguiContext>(ui),
        };

        let mut images = match images {
            Ok(images) => images,
            Err(error) => return error.show_typed::<Assets<Image>>(ui),
        };

        let mut scaled_down_textures = SCALED_DOWN_TEXTURES.lock().unwrap();

        // todo: read asset events to re-rescale images of they changed
        let rescaled = rescaled_image(
            value,
            &mut scaled_down_textures,
            &mut images,
            &mut egui_user_textures,
        );
        let Some((rescaled_handle, texture_id)) = rescaled else {
            ui.label("<texture>");
            return;
        };

        let rescaled_image = images.get(&rescaled_handle).unwrap();
        show_image(rescaled_image, texture_id, ui);
    }

    fn edit_mut(env: ReflectEditor, arg: Arg, value: &mut Self::Target) -> bool {
        Self::edit_ref(env, arg, value);
        false
    }
}

static SCALED_DOWN_TEXTURES: Lazy<Mutex<ScaledDownTextures>> = Lazy::new(Default::default);

fn show_image(
    image: &Image,
    texture_id: egui::TextureId,
    ui: &mut egui::Ui,
) -> Option<egui::Response> {
    let size = image.texture_descriptor.size;
    let size = egui::Vec2::new(size.width as f32, size.height as f32);

    let id = texture_id;
    let source = SizedTexture { id, size };

    if size.max_elem() >= 128.0 {
        let response = egui::CollapsingHeader::new("Texture").show(ui, |ui| ui.image(source));
        response.body_response
    } else {
        Some(ui.image(source))
    }
}

#[derive(Default)]
struct ScaledDownTextures {
    textures: HashMap<Handle<Image>, Handle<Image>>,
    rescaled_textures: HashSet<Handle<Image>>,
}

const RESCALE_TO_FIT: (u32, u32) = (100, 100);

fn rescaled_image(
    handle: &Handle<Image>,
    scaled_down_textures: &mut ScaledDownTextures,
    textures: &mut Assets<Image>,
    egui_usere_textures: &mut EguiUserTextures,
) -> Option<(Handle<Image>, egui::TextureId)> {
    let (texture, texture_id) = match scaled_down_textures.textures.entry(handle.clone()) {
        Entry::Occupied(handle) => {
            let handle: Handle<Image> = handle.get().clone();
            (handle.clone(), egui_usere_textures.add_image(handle))
        }
        Entry::Vacant(entry) => {
            if scaled_down_textures.rescaled_textures.contains(handle) {
                return None;
            }

            let original = textures.get(handle)?;

            let (image, is_srgb) = try_into_dynamic(original)?;
            let filter = image::imageops::FilterType::Triangle;
            let resized = image.resize(RESCALE_TO_FIT.0, RESCALE_TO_FIT.1, filter);
            let resized = from_dynamic(resized, is_srgb);

            let resized_handle = textures.add(resized);
            let weak = resized_handle.clone_weak();
            let texture_id = egui_usere_textures.add_image(resized_handle.clone());
            entry.insert(resized_handle);
            scaled_down_textures.rescaled_textures.insert(weak.clone());

            (weak, texture_id)
        }
    };

    Some((texture, texture_id))
}

/// Converts a [`DynamicImage`] to an [`Image`].
pub fn from_dynamic(dyn_img: DynamicImage, is_srgb: bool) -> Image {
    use bevy::core::cast_slice;
    let width;
    let height;

    let data: Vec<u8>;
    let format: TextureFormat;

    match dyn_img {
        DynamicImage::ImageLuma8(i) => {
            let i = DynamicImage::ImageLuma8(i).into_rgba8();
            width = i.width();
            height = i.height();
            format = if is_srgb {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            };

            data = i.into_raw();
        }
        DynamicImage::ImageLumaA8(i) => {
            let i = DynamicImage::ImageLumaA8(i).into_rgba8();
            width = i.width();
            height = i.height();
            format = if is_srgb {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            };

            data = i.into_raw();
        }
        DynamicImage::ImageRgb8(i) => {
            let i = DynamicImage::ImageRgb8(i).into_rgba8();
            width = i.width();
            height = i.height();
            format = if is_srgb {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            };

            data = i.into_raw();
        }
        DynamicImage::ImageRgba8(i) => {
            width = i.width();
            height = i.height();
            format = if is_srgb {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            };

            data = i.into_raw();
        }
        DynamicImage::ImageLuma16(i) => {
            width = i.width();
            height = i.height();
            format = TextureFormat::R16Uint;

            let raw_data = i.into_raw();

            data = cast_slice(&raw_data).to_owned();
        }
        DynamicImage::ImageLumaA16(i) => {
            width = i.width();
            height = i.height();
            format = TextureFormat::Rg16Uint;

            let raw_data = i.into_raw();

            data = cast_slice(&raw_data).to_owned();
        }
        DynamicImage::ImageRgb16(image) => {
            width = image.width();
            height = image.height();
            format = TextureFormat::Rgba16Uint;

            let size = width as usize * height as usize;
            let mut local_data = Vec::with_capacity(size * format.pixel_size());

            for pixel in image.into_raw().chunks_exact(3) {
                let r = pixel[0].to_ne_bytes();
                let g = pixel[1].to_ne_bytes();
                let b = pixel[2].to_ne_bytes();
                let a = u16::MAX.to_ne_bytes();

                local_data.extend_from_slice(&r);
                local_data.extend_from_slice(&g);
                local_data.extend_from_slice(&b);
                local_data.extend_from_slice(&a);
            }

            data = local_data;
        }
        DynamicImage::ImageRgba16(i) => {
            width = i.width();
            height = i.height();
            format = TextureFormat::Rgba16Uint;

            let raw_data = i.into_raw();

            data = cast_slice(&raw_data).to_owned();
        }
        DynamicImage::ImageRgb32F(image) => {
            width = image.width();
            height = image.height();
            format = TextureFormat::Rgba32Float;

            let mut local_data =
                Vec::with_capacity(width as usize * height as usize * format.pixel_size());

            for pixel in image.into_raw().chunks_exact(3) {
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let a = u16::MAX;

                local_data.extend_from_slice(&r.to_ne_bytes());
                local_data.extend_from_slice(&g.to_ne_bytes());
                local_data.extend_from_slice(&b.to_ne_bytes());
                local_data.extend_from_slice(&a.to_ne_bytes());
            }

            data = local_data;
        }
        DynamicImage::ImageRgba32F(image) => {
            width = image.width();
            height = image.height();
            format = TextureFormat::Rgba32Float;

            let raw_data = image.into_raw();
            data = cast_slice(&raw_data).to_owned();
        }
        // DynamicImage is now non exhaustive, catch future variants and convert them
        _ => {
            let image = dyn_img.into_rgba8();
            width = image.width();
            height = image.height();
            format = TextureFormat::Rgba8UnormSrgb;

            data = image.into_raw();
        }
    }

    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    Image::new(size, TextureDimension::D2, data, format)
}

pub fn try_into_dynamic(image: &Image) -> Option<(DynamicImage, bool)> {
    let (image, is_srgb) = match image.texture_descriptor.format {
        TextureFormat::R8Unorm => (
            DynamicImage::ImageLuma8(ImageBuffer::from_raw(
                image.texture_descriptor.size.width,
                image.texture_descriptor.size.height,
                image.data.clone(),
            )?),
            false,
        ),
        TextureFormat::Rg8Unorm => (
            DynamicImage::ImageLumaA8(ImageBuffer::from_raw(
                image.texture_descriptor.size.width,
                image.texture_descriptor.size.height,
                image.data.clone(),
            )?),
            false,
        ),
        TextureFormat::Rgba8UnormSrgb => (
            DynamicImage::ImageRgba8(ImageBuffer::from_raw(
                image.texture_descriptor.size.width,
                image.texture_descriptor.size.height,
                image.data.clone(),
            )?),
            true,
        ),
        _ => return None,
    };
    Some((image, is_srgb))
}
