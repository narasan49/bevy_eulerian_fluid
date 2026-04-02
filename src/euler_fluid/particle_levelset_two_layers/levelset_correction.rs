pub mod accumulate_phi_correction;
pub mod accumulate_phi_correction_second;
pub mod correct_levelset;
pub mod correct_levelset_second;
pub mod mark_escaped_particles;
pub mod mark_escaped_particles_second;
pub mod reset_levelset_correction;
pub mod reset_levelset_correction_second;
pub mod update_particle_radii;

use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::render_resource::{ComputePass, PipelineCache},
};

use crate::{
    particle_levelset_two_layers::{
        levelset_correction::{
            accumulate_phi_correction::{
                AccumulateLevelSetCorrectionMinusBindGroup, AccumulateLevelSetCorrectionMinusPass,
                AccumulateLevelSetCorrectionPipeline, AccumulateLevelSetCorrectionPlusBindGroup,
                AccumulateLevelSetCorrectionPlusPass,
            },
            accumulate_phi_correction_second::{
                AccumulateLevelSetCorrectionMinusSecondBindGroup,
                AccumulateLevelSetCorrectionMinusSecondPass,
                AccumulateLevelSetCorrectionPlusSecondBindGroup,
                AccumulateLevelSetCorrectionPlusSecondPass,
            },
            correct_levelset::{
                CorrectLevelSetBindGroup, CorrectLevelSetPass, CorrectLevelSetPipeline,
            },
            correct_levelset_second::{CorrectLevelSetSecondBindGroup, CorrectLevelSetSecondPass},
            mark_escaped_particles::{
                MarkEscapedParticlesBindGroup, MarkEscapedParticlesPass,
                MarkEscapedParticlesPipeline,
            },
            mark_escaped_particles_second::{
                MarkEscapedParticlesSecondBindGroup, MarkEscapedParticlesSecondPass,
            },
            reset_levelset_correction::{
                ResetLevelSetCorrectionBindGroup, ResetLevelSetCorrectionPass,
                ResetLevelSetCorrectionPipeline,
            },
            reset_levelset_correction_second::{
                ResetLevelSetCorrectionSecondBindGroup, ResetLevelSetCorrectionSecondPass,
            },
            update_particle_radii::{
                UpdateNegativeParticleRadiiBindGroup, UpdateNegativeParticleRadiiPass,
                UpdateParticleRadiiPipeline, UpdatePositiveParticleRadiiBindGroup,
                UpdatePositiveParticleRadiiPass,
            },
        },
        particle::MAX_PARTICLES_PER_CELL,
        plugin::WORKGROUP_SIZE_PARTICLE,
    },
    pipeline::WORKGROUP_SIZE,
    plugin::FluidComputePassPlugin,
};

pub(crate) struct LevelsetCorrectionPlugin;

impl Plugin for LevelsetCorrectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FluidComputePassPlugin::<MarkEscapedParticlesPass>::default(),
            FluidComputePassPlugin::<ResetLevelSetCorrectionPass>::default(),
            FluidComputePassPlugin::<AccumulateLevelSetCorrectionPlusPass>::default(),
            FluidComputePassPlugin::<AccumulateLevelSetCorrectionMinusPass>::default(),
            FluidComputePassPlugin::<CorrectLevelSetPass>::default(),
            FluidComputePassPlugin::<MarkEscapedParticlesSecondPass>::default(),
            FluidComputePassPlugin::<ResetLevelSetCorrectionSecondPass>::default(),
            FluidComputePassPlugin::<AccumulateLevelSetCorrectionPlusSecondPass>::default(),
            FluidComputePassPlugin::<AccumulateLevelSetCorrectionMinusSecondPass>::default(),
            FluidComputePassPlugin::<CorrectLevelSetSecondPass>::default(),
            FluidComputePassPlugin::<UpdatePositiveParticleRadiiPass>::default(),
            FluidComputePassPlugin::<UpdateNegativeParticleRadiiPass>::default(),
        ));
    }
}

#[derive(QueryData)]
pub(crate) struct PLSLevelsetCorrectionQuery {
    pub mark_escaped_particles_bind_group: &'static MarkEscapedParticlesBindGroup,
    pub reset_levelset_correction_bind_group: &'static ResetLevelSetCorrectionBindGroup,
    pub accumulate_levelset_correction_plus_bind_group:
        &'static AccumulateLevelSetCorrectionPlusBindGroup,
    pub accumulate_levelset_correction_minus_bind_group:
        &'static AccumulateLevelSetCorrectionMinusBindGroup,
    pub correct_levelset_bind_group: &'static CorrectLevelSetBindGroup,
}

#[derive(QueryData)]
pub(crate) struct PLSLevelsetCorrectionSecondQuery {
    pub mark_escaped_particles_bind_group: &'static MarkEscapedParticlesSecondBindGroup,
    pub reset_levelset_correction_bind_group: &'static ResetLevelSetCorrectionSecondBindGroup,
    pub accumulate_levelset_correction_plus_bind_group:
        &'static AccumulateLevelSetCorrectionPlusSecondBindGroup,
    pub accumulate_levelset_correction_minus_bind_group:
        &'static AccumulateLevelSetCorrectionMinusSecondBindGroup,
    pub correct_levelset_bind_group: &'static CorrectLevelSetSecondBindGroup,
    pub update_positive_particle_radii_bind_group: &'static UpdatePositiveParticleRadiiBindGroup,
    pub update_negative_particle_radii_bind_group: &'static UpdateNegativeParticleRadiiBindGroup,
}

pub(crate) fn dispatch(
    world: &World,
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    bind_groups: PLSLevelsetCorrectionQueryItem,
    grid_size: UVec2,
) {
    let num_workgroups_grid = (grid_size / WORKGROUP_SIZE).extend(1);
    let num_workgroups_particle = UVec3::new(
        grid_size.element_product() * MAX_PARTICLES_PER_CELL as u32 / WORKGROUP_SIZE_PARTICLE,
        1,
        1,
    );

    pass.push_debug_group("Level set correction");
    let mark_escaped_particles_pipeline = world.resource::<MarkEscapedParticlesPipeline>();
    mark_escaped_particles_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups.mark_escaped_particles_bind_group.bind_group,
        num_workgroups_particle,
    );

    let reset_levelset_correction_pipeline = world.resource::<ResetLevelSetCorrectionPipeline>();
    reset_levelset_correction_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups.reset_levelset_correction_bind_group.bind_group,
        num_workgroups_grid,
    );

    let accumulate_levelset_correction_pipeline =
        world.resource::<AccumulateLevelSetCorrectionPipeline>();
    accumulate_levelset_correction_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups
            .accumulate_levelset_correction_plus_bind_group
            .bind_group,
        num_workgroups_particle,
    );

    accumulate_levelset_correction_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups
            .accumulate_levelset_correction_minus_bind_group
            .bind_group,
        num_workgroups_particle,
    );

    let correct_levelset_pipeline = world.resource::<CorrectLevelSetPipeline>();
    correct_levelset_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups.correct_levelset_bind_group.bind_group,
        num_workgroups_grid,
    );

    pass.pop_debug_group();
}

pub(crate) fn dispatch_second(
    world: &World,
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    bind_groups: PLSLevelsetCorrectionSecondQueryItem,
    grid_size: UVec2,
) {
    let num_workgroups_grid = (grid_size / WORKGROUP_SIZE).extend(1);
    let num_workgroups_particle = UVec3::new(
        grid_size.element_product() * MAX_PARTICLES_PER_CELL as u32 / WORKGROUP_SIZE_PARTICLE,
        1,
        1,
    );

    pass.push_debug_group("Level set correction (second)");
    let mark_escaped_particles_pipeline = world.resource::<MarkEscapedParticlesPipeline>();
    mark_escaped_particles_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups.mark_escaped_particles_bind_group.bind_group,
        num_workgroups_particle,
    );

    let reset_levelset_correction_pipeline = world.resource::<ResetLevelSetCorrectionPipeline>();
    reset_levelset_correction_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups.reset_levelset_correction_bind_group.bind_group,
        num_workgroups_grid,
    );

    let accumulate_levelset_correction_pipeline =
        world.resource::<AccumulateLevelSetCorrectionPipeline>();
    accumulate_levelset_correction_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups
            .accumulate_levelset_correction_plus_bind_group
            .bind_group,
        num_workgroups_particle,
    );

    accumulate_levelset_correction_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups
            .accumulate_levelset_correction_minus_bind_group
            .bind_group,
        num_workgroups_particle,
    );

    let correct_levelset_pipeline = world.resource::<CorrectLevelSetPipeline>();
    correct_levelset_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups.correct_levelset_bind_group.bind_group,
        num_workgroups_grid,
    );

    let update_particle_radii_pipeline = world.resource::<UpdateParticleRadiiPipeline>();
    update_particle_radii_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups
            .update_positive_particle_radii_bind_group
            .bind_group,
        num_workgroups_particle,
    );

    let update_particle_radii_pipeline = world.resource::<UpdateParticleRadiiPipeline>();
    update_particle_radii_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &bind_groups
            .update_negative_particle_radii_bind_group
            .bind_group,
        num_workgroups_particle,
    );

    pass.pop_debug_group();
}
