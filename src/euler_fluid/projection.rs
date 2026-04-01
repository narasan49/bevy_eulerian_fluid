pub mod gauss_seidel;

use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{ComputePass, PipelineCache},
    },
};

use crate::{
    fluid_uniform::SimulationUniformBindGroup,
    pipeline::DispatchFluidPass,
    plugin::FluidComputePassPlugin,
    projection::gauss_seidel::{
        GaussSeidelBindGroup, GaussSeidelConfig, GaussSeidelPass, GaussSeidelPipeline,
    },
    solve_pressure::{SolvePressureBindGroups, SolvePressurePipeline},
};

pub(crate) struct PressureProjectionPlugin;

impl Plugin for PressureProjectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FluidComputePassPlugin::<GaussSeidelPass>::default(),
            ExtractComponentPlugin::<ProjectionMethod>::default(),
        ));
    }
}

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub enum ProjectionMethod {
    #[default]
    Jacobi,
    GaussSeidel(GaussSeidelConfig),
    MultiGrid,
}

#[derive(QueryData)]
pub(crate) struct ProjectionBindGroupsQuery {
    pub gauss_seidel_bind_group: Option<&'static GaussSeidelBindGroup>,
    pub jacobi_bind_groups: Option<&'static SolvePressureBindGroups>,
}

pub(crate) fn dispatch(
    world: &World,
    method: &ProjectionMethod,
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    projection_bind_groups: ProjectionBindGroupsQueryItem,
    uniform_bind_group: &SimulationUniformBindGroup,
    size: UVec2,
) {
    match method {
        ProjectionMethod::GaussSeidel(config) => {
            pass.push_debug_group("Projection (Gauss-Seidel)");

            let pipeline = world.resource::<GaussSeidelPipeline>();
            let num_workgroups = (size / 8).extend(1);
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
        ProjectionMethod::Jacobi => {
            pass.push_debug_group("Projection (Jacobi)");
            let pipeline = world.resource::<SolvePressurePipeline>();
            let jacobi_iteration_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.jacobi_iteration_pipeline)
                .unwrap();
            let jacobi_iteration_reverse_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.jacobi_iteration_reverse_pipeline)
                .unwrap();

            let bind_groups = projection_bind_groups.jacobi_bind_groups.unwrap();

            pass.set_bind_group(
                1,
                &uniform_bind_group.bind_group,
                &[uniform_bind_group.index],
            );
            for _ in 0..50 {
                pass.set_pipeline(&jacobi_iteration_pipeline);
                pass.set_bind_group(0, &bind_groups.jacobi_iteration_bind_group, &[]);
                pass.dispatch_center(size);

                pass.set_pipeline(&jacobi_iteration_reverse_pipeline);
                pass.set_bind_group(0, &bind_groups.jacobi_iteration_reverse_bind_group, &[]);
                pass.dispatch_center(size);
            }
            pass.pop_debug_group();
        }
        ProjectionMethod::MultiGrid => {
            unimplemented!();
        }
    }
}
