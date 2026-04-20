use bevy::{
    asset::{embedded_asset, embedded_path, AssetPath},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
};

pub struct ExampleMaterialsPlugin;

impl Plugin for ExampleMaterialsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/background.wgsl");
        embedded_asset!(app, "shaders/draw_levelset.wgsl");
        embedded_asset!(app, "shaders/vorticity.wgsl");

        app.add_plugins((
            Material2dPlugin::<BackgroundMaterial>::default(),
            Material2dPlugin::<LevelsetMaterial>::default(),
            Material2dPlugin::<VorticityMaterial>::default(),
        ));
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BackgroundMaterial {}

impl Material2d for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("shaders/background.wgsl"))
                .with_source("embedded"),
        )
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LevelsetMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub levelset: Handle<Image>,
    #[uniform(2)]
    pub base_color: Vec3,
}

impl Material2d for LevelsetMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("shaders/draw_levelset.wgsl"))
                .with_source("embedded"),
        )
    }

    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Blend
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VorticityMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub u: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    pub v: Handle<Image>,
}

impl Material2d for VorticityMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("shaders/vorticity.wgsl"))
                .with_source("embedded"),
        )
    }
}
