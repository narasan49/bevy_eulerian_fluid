use bevy::{
    asset::{load_internal_asset, uuid_handle},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
};

const RENDER_VELOCITY_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("1b76721b-57c8-4cc6-a777-81d87a544fcf");

const RENDER_VELOCITY_2D_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("cd089033-7def-4b5b-95db-3084756cc270");

pub struct FluidMaterialPlugin;

impl Plugin for FluidMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VelocityMaterial>::default())
            .add_plugins(Material2dPlugin::<VelocityMaterial>::default());

        load_internal_asset!(
            app,
            RENDER_VELOCITY_SHADER_HANDLE,
            "material/shaders/render_velocity.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            RENDER_VELOCITY_2D_SHADER_HANDLE,
            "material/shaders/render_velocity_2d.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(Asset, Clone, AsBindGroup, TypePath, Debug)]
pub struct VelocityMaterial {
    #[uniform(0)]
    pub u_range: Vec2,
    #[uniform(1)]
    pub v_range: Vec2,
    #[texture(2)]
    #[sampler(3)]
    pub u: Handle<Image>,
    #[texture(4)]
    #[sampler(5)]
    pub v: Handle<Image>,
}

impl Material for VelocityMaterial {
    fn fragment_shader() -> ShaderRef {
        RENDER_VELOCITY_SHADER_HANDLE.into()
    }
}

impl Material2d for VelocityMaterial {
    fn fragment_shader() -> ShaderRef {
        RENDER_VELOCITY_2D_SHADER_HANDLE.into()
    }
}
