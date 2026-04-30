use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::{
        render_graph::{self, RenderLabel},
        render_resource::{ComputePassDescriptor, PipelineCache},
    },
};

use crate::diagnostics::{
    calculate_volume::{CalculateVolumeBindGroup, CalculateVolumePipeline},
    component::GridSize,
    max_velocity::{MaxVelocityBindGroup, MaxVelocityPipeline},
    min_velocity::{MinVelocityBindGroup, MinVelocityPipeline},
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct DiagnosticsLabel;

enum State {
    Loading,
    Update,
}

#[derive(QueryData)]
struct FluidVolumeQueryData {
    grid_size: &'static GridSize,
    calculate_volume_bind_group: &'static CalculateVolumeBindGroup,
}

#[derive(QueryData)]
struct MinVelocityQueryData {
    grid_size: &'static GridSize,
    min_velocity_bind_group: &'static MinVelocityBindGroup,
}

#[derive(QueryData)]
struct MaxVelocityQueryData {
    grid_size: &'static GridSize,
    max_velocity_bind_group: &'static MaxVelocityBindGroup,
}

pub(crate) struct DiagnosticsNode {
    state: State,
    query: QueryState<FluidVolumeQueryData>,
    q_min_velocity: QueryState<MinVelocityQueryData>,
    q_max_velocity: QueryState<MaxVelocityQueryData>,
}

impl DiagnosticsNode {
    pub fn new(world: &mut World) -> Self {
        Self {
            state: State::Loading,
            query: world.query_filtered(),
            q_min_velocity: world.query_filtered(),
            q_max_velocity: world.query_filtered(),
        }
    }
}

impl render_graph::Node for DiagnosticsNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
        self.q_min_velocity.update_archetypes(world);
        self.q_max_velocity.update_archetypes(world);
        let pipeline_cache = world.resource::<PipelineCache>();
        match self.state {
            State::Loading => {
                let volume_pipeline = world.resource::<CalculateVolumePipeline>();
                let min_velocity_pipeline = world.resource::<MinVelocityPipeline>();
                let max_velocity_pipeline = world.resource::<MaxVelocityPipeline>();
                if volume_pipeline.is_ready(pipeline_cache)
                    && min_velocity_pipeline.is_ready(pipeline_cache)
                    && max_velocity_pipeline.is_ready(pipeline_cache)
                {
                    self.state = State::Update;
                }
            }
            State::Update => {}
        }
    }

    fn run<'w>(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> std::result::Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        match self.state {
            State::Loading => {}
            State::Update => {
                for q in self.query.iter_manual(world) {
                    let mut pass = render_context.command_encoder().begin_compute_pass(
                        &ComputePassDescriptor {
                            label: Some("Fluid diagnostics - Volume"),
                            ..default()
                        },
                    );
                    let grid_size = q.grid_size.0;
                    let calculate_volume_pipeline = world.resource::<CalculateVolumePipeline>();

                    calculate_volume_pipeline.dispatch(
                        pipeline_cache,
                        &mut pass,
                        q.calculate_volume_bind_group,
                        grid_size,
                    );
                }

                for q in self.q_min_velocity.iter_manual(world) {
                    let mut pass = render_context.command_encoder().begin_compute_pass(
                        &ComputePassDescriptor {
                            label: Some("Fluid diagnostics - Min velocity"),
                            ..default()
                        },
                    );
                    let grid_size = q.grid_size.0;
                    let min_velocity_pipeline = world.resource::<MinVelocityPipeline>();
                    min_velocity_pipeline.dispatch(
                        pipeline_cache,
                        &mut pass,
                        q.min_velocity_bind_group,
                        grid_size,
                    );
                }

                for q in self.q_max_velocity.iter_manual(world) {
                    let mut pass = render_context.command_encoder().begin_compute_pass(
                        &ComputePassDescriptor {
                            label: Some("Fluid diagnostics - Max velocity"),
                            ..default()
                        },
                    );
                    let grid_size = q.grid_size.0;
                    let max_velocity_pipeline = world.resource::<MaxVelocityPipeline>();
                    max_velocity_pipeline.dispatch(
                        pipeline_cache,
                        &mut pass,
                        q.max_velocity_bind_group,
                        grid_size,
                    );
                }
            }
        }
        Ok(())
    }
}
