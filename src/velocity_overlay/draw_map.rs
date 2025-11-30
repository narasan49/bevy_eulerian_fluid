use bevy::{
    asset::{embedded_asset, embedded_path, AssetPath},
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderType},
        storage::ShaderStorageBuffer,
    },
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
};

use crate::{
    settings::{FluidSettings, FluidTextures},
    velocity_overlay::{construct_map::ConstructVelocityArrowsResource, VelocityOverlay},
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
}

fn spawn_arrows_on_fluid_spawn(
    mut commands: Commands,
    query: Query<(Entity, &FluidSettings, &FluidTextures, &VelocityOverlay), Added<FluidTextures>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VectorMapMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (entity, fluid_settings, fluid_texture, overlay_settings) in &query {
        info!("Setting up velocity vector map.");
        let arrow_dim =
            (fluid_settings.size / overlay_settings.bin_size).element_product() as usize;
        let buffer = buffers.add(ShaderStorageBuffer::from(vec![Arrow::default(); arrow_dim]));

        let mesh = meshes.add(Segment2d::default());
        let material = materials.add(VectorMapMaterial {
            arrows: buffer.clone(),
        });
        let construc_velocity_arrows_resource = ConstructVelocityArrowsResource {
            u0: fluid_texture.u.clone(),
            v0: fluid_texture.v.clone(),
            arrows: buffer,
        };

        commands
            .entity(entity)
            .insert(construc_velocity_arrows_resource);

        // Spawn arrow instances as child entities.
        for _ in 0..arrow_dim {
            let arrow_entity = commands
                .spawn((
                    Mesh2d(mesh.clone()),
                    MeshMaterial2d(material.clone()),
                    Transform::IDENTITY,
                ))
                .id();
            commands.entity(entity).add_child(arrow_entity);
        }
    }
}
