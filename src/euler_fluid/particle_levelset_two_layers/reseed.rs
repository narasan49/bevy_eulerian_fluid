pub mod add_particles;
pub mod count_particles_in_cell;
pub mod delete_particles;
pub mod prefix_sum_alive_particles;
pub mod prefix_sum_particle_counts;
pub mod reseed_particles;
pub mod sort_particles;
pub mod update_particles_count;

use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::render_resource::{ComputePass, PipelineCache},
};

use crate::{
    common_pass::prefix_sum::PrefixSumPipeline,
    fluid_uniform::SimulationUniformBindGroup,
    particle_levelset_two_layers::{
        particle::MAX_PARTICLES_PER_CELL,
        plugin::WORKGROUP_SIZE_PARTICLE,
        reseed::{
            add_particles::{
                AddNegativeParticlesBindGroup, AddNegativeParticlesPass, AddParticlesPipeline,
                AddPositiveParticlesBindGroup, AddPositiveParticlesPass,
            },
            count_particles_in_cell::{
                CountNegativeParticlesInCellBindGroup, CountNegativeParticlesInCellPass,
                CountParticlesInCellPipeline, CountPositiveParticlesInCellBindGroup,
                CountPositiveParticlesInCellPass,
            },
            delete_particles::{
                DeleteNegativeParticlesBindGroup, DeleteNegativeParticlesPass,
                DeleteParticlesPipeline, DeletePositiveParticlesBindGroup,
                DeletePositiveParticlesPass,
            },
            prefix_sum_alive_particles::{
                PrefixSumAliveNegativeParticlesBindGroup, PrefixSumAliveNegativeParticlesPass,
                PrefixSumAlivePositiveParticlesBindGroup, PrefixSumAlivePositiveParticlesPass,
            },
            prefix_sum_particle_counts::{
                PrefixSumNegativeParticlesCountBindGroup, PrefixSumNegativeParticlesCountPass,
                PrefixSumPositiveParticlesCountBindGroup, PrefixSumPositiveParticlesCountPass,
            },
            reseed_particles::{
                ReseedNegativeParticlesBindGroup, ReseedNegativeParticlesPass,
                ReseedParticlesPipeline, ReseedPositiveParticlesBindGroup,
                ReseedPositiveParticlesPass,
            },
            sort_particles::{
                SortNegativeParticlesBindGroup, SortNegativeParticlesPass, SortParticlesPipeline,
                SortPositiveParticlesBindGroup, SortPositiveParticlesPass,
            },
            update_particles_count::{
                UpdateNegativeParticlesCountBindGroup, UpdateNegativeParticlesCountPass,
                UpdateParticlesCountPipeline, UpdatePositiveParticlesCountBindGroup,
                UpdatePositiveParticlesCountPass,
            },
        },
    },
    pipeline::WORKGROUP_SIZE,
    plugin::FluidComputePassPlugin,
};

pub(crate) struct ReseedPlugin;

impl Plugin for ReseedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // sort negative/positive particles
            // - count particles
            // - prefix sum particle coounts
            // - sort particles
            FluidComputePassPlugin::<CountPositiveParticlesInCellPass>::default(),
            FluidComputePassPlugin::<CountNegativeParticlesInCellPass>::default(),
            FluidComputePassPlugin::<PrefixSumPositiveParticlesCountPass>::default(),
            FluidComputePassPlugin::<PrefixSumNegativeParticlesCountPass>::default(),
            FluidComputePassPlugin::<SortPositiveParticlesPass>::default(),
            FluidComputePassPlugin::<SortNegativeParticlesPass>::default(),
        ))
        .add_plugins((
            // Reseed
            FluidComputePassPlugin::<ReseedPositiveParticlesPass>::default(),
            FluidComputePassPlugin::<ReseedNegativeParticlesPass>::default(),
            FluidComputePassPlugin::<PrefixSumAlivePositiveParticlesPass>::default(),
            FluidComputePassPlugin::<PrefixSumAliveNegativeParticlesPass>::default(),
        ))
        .add_plugins((
            // delete particles
            // update particles count
            // add particles
            FluidComputePassPlugin::<DeletePositiveParticlesPass>::default(),
            FluidComputePassPlugin::<DeleteNegativeParticlesPass>::default(),
            FluidComputePassPlugin::<UpdatePositiveParticlesCountPass>::default(),
            FluidComputePassPlugin::<UpdateNegativeParticlesCountPass>::default(),
            FluidComputePassPlugin::<AddPositiveParticlesPass>::default(),
            FluidComputePassPlugin::<AddNegativeParticlesPass>::default(),
        ));
    }
}

#[derive(QueryData)]
pub(crate) struct PLSReseedBindGroupsQuery {
    pub count_positive_particles_in_cell: &'static CountPositiveParticlesInCellBindGroup,
    pub count_negative_particles_in_cell: &'static CountNegativeParticlesInCellBindGroup,
    pub prefix_sum_positive_particles_count_bind_group:
        &'static PrefixSumPositiveParticlesCountBindGroup,
    pub prefix_sum_negative_particles_count_bind_group:
        &'static PrefixSumNegativeParticlesCountBindGroup,
    pub sort_positive_particles_bind_group: &'static SortPositiveParticlesBindGroup,
    pub sort_negative_particles_bind_group: &'static SortNegativeParticlesBindGroup,
    pub reseed_positive_particles_bind_group: &'static ReseedPositiveParticlesBindGroup,
    pub reseed_negative_particles_bind_group: &'static ReseedNegativeParticlesBindGroup,
    pub prefix_sum_alive_positive_particles_bind_group:
        &'static PrefixSumAlivePositiveParticlesBindGroup,
    pub prefix_sum_alive_negative_particles_bind_group:
        &'static PrefixSumAliveNegativeParticlesBindGroup,
    pub delete_positive_particles_bind_group: &'static DeletePositiveParticlesBindGroup,
    pub delete_negative_particles_bind_group: &'static DeleteNegativeParticlesBindGroup,
    pub update_positive_particles_count_bind_group: &'static UpdatePositiveParticlesCountBindGroup,
    pub update_negative_particles_count_bind_group: &'static UpdateNegativeParticlesCountBindGroup,
    pub add_positive_particles_bind_group: &'static AddPositiveParticlesBindGroup,
    pub add_negative_particles_bind_group: &'static AddNegativeParticlesBindGroup,
}

pub(crate) fn dispatch(
    world: &World,
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    reseed_bind_groups: PLSReseedBindGroupsQueryItem,
    uniform_bind_group: &SimulationUniformBindGroup,
    grid_size: UVec2,
) {
    let num_workgroups_grid = (grid_size / WORKGROUP_SIZE).extend(1);
    let num_workgroups_particle = UVec3::new(
        grid_size.element_product() * MAX_PARTICLES_PER_CELL as u32 / WORKGROUP_SIZE_PARTICLE,
        1,
        1,
    );

    pass.push_debug_group("Reseed particles");
    pass.push_debug_group("Sort particles");

    let count_particles_pipeline = world.resource::<CountParticlesInCellPipeline>();
    count_particles_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .count_positive_particles_in_cell
            .bind_group,
        num_workgroups_particle,
    );
    count_particles_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .count_negative_particles_in_cell
            .bind_group,
        num_workgroups_particle,
    );

    let prefix_sum_pipeline = world.resource::<PrefixSumPipeline>();
    prefix_sum_pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .prefix_sum_positive_particles_count_bind_group
            .bind_group,
        grid_size,
    );
    prefix_sum_pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .prefix_sum_negative_particles_count_bind_group
            .bind_group,
        grid_size,
    );

    let sort_particles_pipeline = world.resource::<SortParticlesPipeline>();
    sort_particles_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .sort_positive_particles_bind_group
            .bind_group,
        num_workgroups_particle,
    );
    sort_particles_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .sort_negative_particles_bind_group
            .bind_group,
        num_workgroups_particle,
    );

    pass.pop_debug_group();

    let reseed_particles_pipeline = world.resource::<ReseedParticlesPipeline>();
    pass.push_debug_group("Reseed positive particles");
    reseed_particles_pipeline.pipeline.dispatch_with_uniform(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .reseed_positive_particles_bind_group
            .bind_group,
        uniform_bind_group,
        num_workgroups_grid,
    );
    prefix_sum_pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .prefix_sum_alive_positive_particles_bind_group
            .bind_group,
        grid_size,
    );

    let delete_particles_pipeline = world.resource::<DeleteParticlesPipeline>();
    delete_particles_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .delete_positive_particles_bind_group
            .bind_group,
        num_workgroups_particle,
    );

    let update_particles_count_pipeline = world.resource::<UpdateParticlesCountPipeline>();
    update_particles_count_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .update_positive_particles_count_bind_group
            .bind_group,
        UVec3::ONE,
    );

    let add_particles_pipeline = world.resource::<AddParticlesPipeline>();
    add_particles_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .add_positive_particles_bind_group
            .bind_group,
        num_workgroups_grid,
    );

    pass.pop_debug_group();

    pass.push_debug_group("Reseed negative particles");

    reseed_particles_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .reseed_negative_particles_bind_group
            .bind_group,
        num_workgroups_grid,
    );
    prefix_sum_pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .prefix_sum_alive_negative_particles_bind_group
            .bind_group,
        grid_size,
    );
    delete_particles_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .delete_negative_particles_bind_group
            .bind_group,
        num_workgroups_particle,
    );
    update_particles_count_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .update_negative_particles_count_bind_group
            .bind_group,
        UVec3::ONE,
    );
    add_particles_pipeline.pipeline.dispatch(
        pipeline_cache,
        pass,
        &reseed_bind_groups
            .add_negative_particles_bind_group
            .bind_group,
        num_workgroups_grid,
    );
    pass.pop_debug_group();

    pass.pop_debug_group();
}
