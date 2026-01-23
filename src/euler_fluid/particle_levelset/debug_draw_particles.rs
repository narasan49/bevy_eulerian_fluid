use bevy::{
    asset::{embedded_asset, embedded_path, AssetPath},
    mesh::MeshTag,
    prelude::*,
    render::{render_resource::AsBindGroup, storage::ShaderStorageBuffer},
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
};

use crate::{
    particle_levelset::initialize_particles::InitializeParticlesResource,
    settings::{FluidSettings, FluidTextures},
};

pub(super) struct DebugDrawLevelsetParticlesPlugin;

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
    #[uniform(1)]
    fluid_size: Vec2,
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
    query: Query<
        (
            Entity,
            &FluidTextures,
            &InitializeParticlesResource,
            &FluidSettings,
        ),
        Added<FluidTextures>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LevelsetParticlesMaterial>>,
    buffers: Res<Assets<ShaderStorageBuffer>>,
) {
    for (entity, fluid_textures, initialize_particles, settings) in &query {
        let particle_count = buffers.get(&initialize_particles.count).unwrap();
        let count: &u32 = bytemuck::from_bytes(particle_count.data.as_ref().unwrap());
        info!(
            "Spawning debug markers for levelset particles. Particle count {:?}",
            *count
        );

        let mesh = meshes.add(Circle::new(1.0));
        let material = materials.add(LevelsetParticlesMaterial {
            levelset_particles: fluid_textures.levelset_particles.clone(),
            fluid_size: settings.size.as_vec2(),
        });

        // ToDo: Correct number of meshes
        for i in 0..4000 {
            commands.spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(material.clone()),
                MeshTag(i),
                ChildOf(entity),
            ));
        }
    }
}
