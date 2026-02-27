pub mod advect_particles;
pub mod debug_draw_particles;
pub mod distribute_particles_to_grid;
pub mod initialize_interface_indices;
pub mod initialize_particles;

use bevy::{
    asset::embedded_asset,
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin, render_resource::ShaderType, Render, RenderApp,
        RenderSystems,
    },
    shader::load_shader_library,
};

use crate::particle_levelset::debug_draw_particles::DebugDrawLevelsetParticlesPlugin;

#[derive(ShaderType, Clone, Copy, Default)]
pub(crate) struct Particle {
    pub position: Vec2,
    pub level: f32,
}

pub(crate) struct ParticleLevelsetPlugin;

impl Plugin for ParticleLevelsetPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(
            app,
            "particle_levelset/shaders/initialize_interface_indices.wgsl"
        );
        embedded_asset!(app, "particle_levelset/shaders/initialize_particles.wgsl");
        embedded_asset!(app, "particle_levelset/shaders/advect_particles.wgsl");
        embedded_asset!(
            app,
            "particle_levelset/shaders/distribute/count_particles_in_cell.wgsl"
        );
        embedded_asset!(
            app,
            "particle_levelset/shaders/distribute/prefix_sum_particle_counts.wgsl"
        );
        embedded_asset!(
            app,
            "particle_levelset/shaders/distribute/sort_particles.wgsl"
        );
        embedded_asset!(
            app,
            "particle_levelset/shaders/distribute/distribute_particles.wgsl"
        );
        embedded_asset!(
            app,
            "particle_levelset/shaders/distribute/correct_levelset.wgsl"
        );

        load_shader_library!(app, "particle_levelset/shaders/particle.wgsl");

        app.add_plugins((
            ExtractComponentPlugin::<
                initialize_interface_indices::InitializeInterfaceIndicesResource,
            >::default(),
            ExtractComponentPlugin::<initialize_particles::InitializeParticlesResource>::default(),
            ExtractComponentPlugin::<advect_particles::AdvectParticlesResource>::default(),
            ExtractComponentPlugin::<distribute_particles_to_grid::CountParticlesInCellResource>::default(),
            ExtractComponentPlugin::<distribute_particles_to_grid::PrefixSumParticleCountsResource>::default(),
            ExtractComponentPlugin::<distribute_particles_to_grid::SortParticlesResource>::default(),
            ExtractComponentPlugin::<distribute_particles_to_grid::DistributeParticlesResource>::default(),
            ExtractComponentPlugin::<distribute_particles_to_grid::CorrectLevelsetResource>::default(),
            DebugDrawLevelsetParticlesPlugin,
        ));
        app.add_systems(Update, distribute_particles_to_grid::reset_buffers);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            (
                initialize_interface_indices::prepare_bind_groups,
                initialize_particles::prepare_bind_groups,
                advect_particles::prepare_bind_groups,
                distribute_particles_to_grid::prepare_bind_groups,
            )
                .in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<initialize_interface_indices::InitializeInterfaceIndicesPipeline>();
        render_app.init_resource::<initialize_particles::InitializeParticlesPipeline>();
        render_app.init_resource::<advect_particles::AdvectParticlesPipeline>();
        render_app
            .init_resource::<distribute_particles_to_grid::DistributeParticlesToGridPipelines>();
    }
}
