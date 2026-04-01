use bevy::{
    asset::{embedded_asset, embedded_path, AssetPath},
    mesh::MeshTag,
    prelude::*,
    render::{render_resource::AsBindGroup, storage::ShaderStorageBuffer},
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
};

use crate::{
    particle_levelset_two_layers::plugin::PLSResources,
    settings::{FluidSettings, FluidTextures},
};

pub(crate) struct DebugDrawLevelsetParticlesPlugin;

impl Plugin for DebugDrawLevelsetParticlesPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/debug_draw_particles.wgsl");
        app.add_plugins(Material2dPlugin::<LevelsetParticlesMaterial>::default())
            .add_systems(Update, spawn_particles);
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct LevelsetParticlesMaterial {
    #[storage(0, read_only)]
    levelset_particles: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only)]
    levelset_particles_count: Handle<ShaderStorageBuffer>,
    #[uniform(2)]
    fluid_size: Vec2,
    #[uniform(3)]
    color: LinearRgba,
}

impl Material2d for LevelsetParticlesMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("shaders/debug_draw_particles.wgsl"))
                .with_source("embedded"),
        )
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("shaders/debug_draw_particles.wgsl"))
                .with_source("embedded"),
        )
    }
}

fn spawn_particles(
    mut commands: Commands,
    query: Query<(Entity, &PLSResources, &FluidSettings), Added<FluidTextures>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LevelsetParticlesMaterial>>,
) {
    for (entity, pls_resources, settings) in &query {
        let mesh = meshes.add(Circle::new(1.0));
        let material = materials.add(LevelsetParticlesMaterial {
            levelset_particles: pls_resources.positive_particles.clone(),
            levelset_particles_count: pls_resources.positive_particles_count.clone(),
            fluid_size: settings.size.as_vec2(),
            color: LinearRgba::RED,
        });

        let negative_material = materials.add(LevelsetParticlesMaterial {
            levelset_particles: pls_resources.negative_particles.clone(),
            levelset_particles_count: pls_resources.negative_particles_count.clone(),
            fluid_size: settings.size.as_vec2(),
            color: LinearRgba::GREEN,
        });

        // ToDo: Correct number of meshes
        for i in 0..32000 {
            commands.spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(material.clone()),
                MeshTag(i),
                ChildOf(entity),
            ));
            commands.spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(negative_material.clone()),
                MeshTag(i),
                ChildOf(entity),
            ));
        }
    }
}
