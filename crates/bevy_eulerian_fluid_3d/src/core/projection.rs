use std::fmt::Display;

use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{ComputePass, PipelineCache},
    },
};
use bevy_eulerian_fluid_common::fluid_compute_pass::FluidComputePassPlugin;

use crate::core::{
    fluid_uniform::FluidUniformBindGroup,
    projection::{
        gauss_seidel::{
            GaussSeidelBindGroup, GaussSeidelConfig, GaussSeidelPass, GaussSeidelPipeline,
        },
        multigrid::{
            MultiGridBindGroups, MultiGridConfig, MultiGridNumLevels, MultiGridPassPlugin,
            MultiGridPipelines,
        },
    },
    workgroup::{workgroup_size_center, WorkgroupShape},
};

pub mod gauss_seidel;
pub mod multigrid;

#[derive(QueryData)]
pub(crate) struct ProjectionBindGroupsQuery {
    pub gauss_seidel_bind_group: Option<&'static GaussSeidelBindGroup>,
    pub multi_grid_bind_groups: Option<(&'static MultiGridBindGroups, &'static MultiGridNumLevels)>,
}
pub(crate) struct ProjectionPassPlugin;

impl Plugin for ProjectionPassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FluidComputePassPlugin::<GaussSeidelPass>::default(),
            ExtractComponentPlugin::<ProjectionMethod>::default(),
            MultiGridPassPlugin,
        ));
    }
}

/// Method for iterative pressure solver.
#[derive(Component, ExtractComponent, Clone, Debug)]
pub enum ProjectionMethod {
    /// SOR-weighted Red Black Gauss-Seidel solver. It converges faster than Jacobi iteration.
    GaussSeidel(GaussSeidelConfig),
    /// Multigrid solver. Most performant.
    MultiGrid(MultiGridConfig),
}

impl Default for ProjectionMethod {
    fn default() -> Self {
        ProjectionMethod::GaussSeidel(GaussSeidelConfig::default())
    }
}

impl Display for ProjectionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectionMethod::GaussSeidel(gauss_seidel_config) => {
                write!(f, "Gauss-Seidel({})", gauss_seidel_config.num_iterations)
            }
            ProjectionMethod::MultiGrid(multi_grid_config) => {
                write!(
                    f,
                    "Multigrid({}-{}-{})",
                    multi_grid_config.pre_smooth_config.num_iterations,
                    multi_grid_config.coarsest_config.num_iterations,
                    multi_grid_config.post_smooth_config.num_iterations
                )
            }
        }
    }
}

pub(crate) fn dispatch(
    world: &World,
    method: &ProjectionMethod,
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    projection_bind_groups: ProjectionBindGroupsQueryItem,
    uniform_bind_group: &FluidUniformBindGroup,
    workgroup_shape: &WorkgroupShape,
    size: UVec3,
) {
    match method {
        ProjectionMethod::GaussSeidel(config) => {
            pass.push_debug_group("projection (gauss-seidel)");

            let pipeline = world.resource::<GaussSeidelPipeline>();
            let num_workgroups = workgroup_size_center(size, workgroup_shape);
            pipeline.dispatch(
                pipeline_cache,
                pass,
                &projection_bind_groups
                    .gauss_seidel_bind_group
                    .unwrap()
                    .bind_group,
                uniform_bind_group,
                num_workgroups,
                config,
            );

            pass.pop_debug_group();
        }
        ProjectionMethod::MultiGrid(config) => {
            let (bind_groups, num_levels) = projection_bind_groups.multi_grid_bind_groups.expect(
                "MultiGridBindGroups or MultiGridNumLevels components are missing in RenderWorld.",
            );
            pass.push_debug_group("projection (multigrid)");
            let pipelines = world.resource::<MultiGridPipelines>();
            pipelines.dispatch(
                pipeline_cache,
                pass,
                bind_groups,
                uniform_bind_group,
                workgroup_shape,
                size,
                config,
                num_levels,
            );
            pass.pop_debug_group();
        }
    }
}
