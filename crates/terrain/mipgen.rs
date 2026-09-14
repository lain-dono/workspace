use bevy::{
    platform::collections::HashSet,
    prelude::*,
    render::render_resource::{TextureDimension, TextureFormat},
};
use image::{DynamicImage, ImageBuffer, imageops::FilterType};

pub fn plugin(app: &mut App) {
    app.init_resource::<ImageWaiter>()
        .add_systems(Update, gen_mipmap);
}

#[derive(Resource, Default)]
pub struct ImageWaiter {
    handles: HashSet<Handle<Image>>,
}

impl ImageWaiter {
    pub fn add(&mut self, handle: Handle<Image>) {
        self.handles.insert(handle);
    }
}

pub fn gen_mipmap(
    mut to_remove: Local<Vec<Handle<Image>>>,
    mut waiters: ResMut<ImageWaiter>,

    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
) {
    for event in events.read() {
        for id in &waiters.handles {
            if event.is_loaded_with_dependencies(id) {
                let mut source = images.get_mut(id).unwrap();
                let filter = image::imageops::FilterType::CatmullRom;
                crate::mipgen::generate_mipmap(&mut source, filter);
                to_remove.push(id.clone());
            }
        }
    }

    for id in to_remove.drain(..) {
        waiters.handles.remove(&id);
    }
}

/// `added_cache_size` is for tracking the amount of data that was cached by this call.
/// Compressed BCn data is cached on disk if cache_compressed_image_data is enabled.
pub fn generate_mipmap(source: &mut Image, filter: FilterType) {
    let size = source.texture_descriptor.size;
    let format = source.texture_descriptor.format;
    let dimension = source.texture_descriptor.dimension;

    assert!(!source.is_compressed());
    assert_eq!(dimension, TextureDimension::D2);
    assert_eq!(size.depth_or_array_layers, 1);

    let Some(buf) = source.data.clone() else {
        panic!("Conversion into dynamic image not supported for GPU storage texture")
    };

    let image = match format {
        TextureFormat::R8Unorm | TextureFormat::R8Snorm | TextureFormat::R8Uint => {
            ImageBuffer::from_raw(size.width, size.height, buf).map(DynamicImage::ImageLuma8)
        }
        TextureFormat::R16Unorm | TextureFormat::R16Snorm | TextureFormat::R16Uint => {
            let (buf, _) = buf.as_chunks::<2>();
            let buf = buf.iter().map(|&bytes| u16::from_le_bytes(bytes)).collect();
            ImageBuffer::from_raw(size.width, size.height, buf).map(DynamicImage::ImageLuma16)
        }
        TextureFormat::Rg8Unorm => {
            ImageBuffer::from_raw(size.width, size.height, buf).map(DynamicImage::ImageLumaA8)
        }
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => {
            ImageBuffer::from_raw(size.width, size.height, buf).map(DynamicImage::ImageRgba8)
        }
        _ => panic!("Conversion into dynamic image not supported for {format:?}."),
    };

    let mut image = image.unwrap();
    let mut data = image.as_bytes().to_vec();

    let mip_level_count = size.max_mips(dimension);

    for level in 1..mip_level_count {
        let size = size.mip_level_size(level, dimension).physical_size(format);
        image = image.resize_exact(size.width, size.height, filter);
        data.extend_from_slice(image.as_bytes());
    }

    source.texture_descriptor.mip_level_count = mip_level_count;
    source.data = Some(data);
    source.copy_on_resize = true;
}
