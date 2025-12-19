use bevy::{
    asset::{embedded_asset, embedded_path, AssetPath},
    mesh::MeshTag,
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderType},
        storage::ShaderStorageBuffer,
    },
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};

use crate::{
    settings::{FluidSettings, FluidTextures},
    velocity_overlay::{
        construct_map::ConstructVelocityArrowsResource, InitialOverlayVisibility, VelocityOverlay,
        VelocityOverlayGroup,
    },
};

const SHADER_PATH: &str = "shaders/vector_map.wgsl";

#[derive(ShaderType, Clone, Default)]
struct Arrow {
    position: Vec2,
    vector: Vec2,
    color: LinearRgba,
}

pub(super) struct DrawOverlayVelocityPlugin;

impl Plugin for DrawOverlayVelocityPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/vector_map.wgsl");
        app.add_plugins(Material2dPlugin::<VectorMapMaterial>::default())
            .add_systems(Update, spawn_arrows_on_fluid_spawn);
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct VectorMapMaterial {
    #[storage(0, read_only)]
    arrows: Handle<ShaderStorageBuffer>,
}

impl Material2d for VectorMapMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!(SHADER_PATH)).with_source("embedded"),
        )
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!(SHADER_PATH)).with_source("embedded"),
        )
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

fn spawn_arrows_on_fluid_spawn(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &FluidSettings,
            &FluidTextures,
            &VelocityOverlay,
            &InitialOverlayVisibility,
        ),
        Added<FluidTextures>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VectorMapMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (entity, fluid_settings, fluid_textures, overlay_settings, initial_visibility) in &query {
        info!("Setting up velocity vector map.");
        let arrow_dim =
            (fluid_settings.size / overlay_settings.bin_size).element_product() as usize;
        let buffer = buffers.add(ShaderStorageBuffer::from(vec![Arrow::default(); arrow_dim]));

        // Segment2d does not contain UV.
        // let mesh = meshes.add(Segment2d::default());
        let mesh = meshes.add(Rectangle::from_size(Vec2::new(1.0, 0.1)));
        let material = materials.add(VectorMapMaterial {
            arrows: buffer.clone(),
        });
        let construc_velocity_arrows_resource = ConstructVelocityArrowsResource {
            u0: fluid_textures.u.clone(),
            v0: fluid_textures.v.clone(),
            arrows: buffer.clone(),
        };

        commands
            .entity(entity)
            .insert(construc_velocity_arrows_resource);

        let group_entity = commands
            .spawn((VelocityOverlayGroup, initial_visibility.0, ChildOf(entity)))
            .id();
        // Spawn arrow instances as child entities.
        for idx in 0..arrow_dim {
            commands.spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(material.clone()),
                MeshTag(idx as u32),
                Transform::IDENTITY,
                ChildOf(group_entity),
            ));
        }
    }
}
