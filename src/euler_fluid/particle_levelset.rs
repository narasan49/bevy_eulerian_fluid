pub mod advect_particles;
pub mod debug_draw_particles;
pub mod initialize_interface_indices;
pub mod initialize_particles;

use bevy::{
    asset::embedded_asset,
    prelude::*,
    render::{extract_component::ExtractComponentPlugin, Render, RenderApp, RenderSystems},
};

use crate::particle_levelset::debug_draw_particles::DebugDrawLevelsetParticlesPlugin;

pub(crate) struct ParticleLevelsetPlugin;

impl Plugin for ParticleLevelsetPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(
            app,
            "particle_levelset/shaders/initialize_interface_indices.wgsl"
        );
        embedded_asset!(app, "particle_levelset/shaders/initialize_particles.wgsl");
        embedded_asset!(app, "particle_levelset/shaders/advect_particles.wgsl");
        
        app.add_plugins((
            ExtractComponentPlugin::<
                initialize_interface_indices::InitializeInterfaceIndicesResource,
            >::default(),
            ExtractComponentPlugin::<initialize_particles::InitializeParticlesResource>::default(),
            ExtractComponentPlugin::<advect_particles::AdvectParticlesResource>::default(),
            DebugDrawLevelsetParticlesPlugin,
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            (
                initialize_interface_indices::prepare_bind_groups,
                initialize_particles::prepare_bind_groups,
                advect_particles::prepare_bind_groups,
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
    }
}
