use bevy::{
    asset::RenderAssetUsages,
    image::TextureFormatPixelInfo,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};

pub trait ImageForCS {
    fn new_texture_storage(size: (u32, u32), format: TextureFormat) -> Self;
}

impl ImageForCS for Image {
    fn new_texture_storage(size: (u32, u32), format: TextureFormat) -> Self {
        let pixel_size = format.pixel_size().unwrap();
        let mut zeros = Vec::new();
        zeros.resize(pixel_size, 0u8);

        let mut image = Image::new_fill(
            Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &zeros,
            format,
            RenderAssetUsages::RENDER_WORLD,
        );

        image.texture_descriptor.usage = TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING;
        image
    }
}

pub trait NewTexture {
    fn new_texture_storage(&mut self, size: (u32, u32), format: TextureFormat) -> Handle<Image>;
}

impl<'a> NewTexture for ResMut<'a, Assets<Image>> {
    fn new_texture_storage(&mut self, size: (u32, u32), format: TextureFormat) -> Handle<Image> {
        let u0 = Image::new_texture_storage(size, format);
        self.add(u0)
    }
}
