use bevy::{
    asset::RenderAssetUsages,
    image::TextureFormatPixelInfo,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};

pub fn new_texture_storage_3d(
    images: &mut Assets<Image>,
    size: UVec3,
    format: TextureFormat,
) -> Handle<Image> {
    let pixel_size = format.pixel_size().unwrap();
    let mut zeros = Vec::new();
    zeros.resize(pixel_size, 0u8);

    let mut image = Image::new_fill(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: size.z,
        },
        TextureDimension::D3,
        &zeros,
        format,
        RenderAssetUsages::RENDER_WORLD,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    images.add(image)
}
