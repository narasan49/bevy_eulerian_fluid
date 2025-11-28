use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::{
        render_graph::{self, RenderLabel},
        render_resource::{ComputePass, ComputePassDescriptor, PipelineCache},
    },
};

use crate::{
    advect_scalar::{AdvectScalarBindGroups, AdvectScalarPipeline},
    advection::{AdvectionBindGroups, AdvectionPipeline},
    apply_forces::{ApplyForcesBindGroups, ApplyForcesPipeline},
    divergence::{DivergenceBindGroups, DivergencePipeline},
    extrapolate_velocity::{ExtrapolateVelocityBindGroups, ExtrapolateVelocityPipeline},
    fluid_status::FluidStatus,
    fluid_to_solid::{
        FluidToSolidForcesBindGroups, FluidToSolidForcesPipeline, SolidObstaclesBindGroups,
        MAX_SOLIDS,
    },
    fluid_uniform::SimulationUniformBindGroup,
    initialize::{InitializeBindGroups, InitializePipeline},
    physics_time::{CurrentPhysicsStepNumberRenderWorld, PhysicsFrameInfo},
    pipeline::{DispatchFluidPass, Pipeline},
    reinitialize_levelset::{ReinitLevelsetBindGroups, ReinitLevelsetPipeline},
    settings::FluidSettings,
    solve_pressure::{SolvePressureBindGroups, SolvePressurePipeline},
    solve_velocity::{SolveVelocityBindGroups, SolveVelocityPipeline},
    update_solid::{UpdateSolidBindGroups, UpdateSolidPipeline},
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct FluidLabel;

#[derive(Debug)]
enum State {
    Loading,
    Init,
    Update,
    Idle,
}

#[derive(QueryData)]
struct FluidBindGroupsQueryData {
    initialize_bind_groups: &'static InitializeBindGroups,
    update_solid_bind_groups: &'static UpdateSolidBindGroups,
    advection_bind_groups: &'static AdvectionBindGroups,
    apply_forces_bind_groups: &'static ApplyForcesBindGroups,
    divergence_bind_groups: &'static DivergenceBindGroups,
    solve_pressure_bind_groups: &'static SolvePressureBindGroups,
    solve_velocity_bind_groups: &'static SolveVelocityBindGroups,
    extrapolate_velocity_bind_groups: &'static ExtrapolateVelocityBindGroups,
    advect_scalar_bind_groups: &'static AdvectScalarBindGroups,
    reinit_levelset_bind_groups: &'static ReinitLevelsetBindGroups,
    fluid_to_solid_bind_groups: &'static FluidToSolidForcesBindGroups,
    simulation_uniform: &'static SimulationUniformBindGroup,
}

pub(crate) struct EulerFluidNode {
    state: State,
    // Query BindGroups components
    // Reference: bevy\crates\bevy_ui\src\render\render_pass.rs
    fluid_query: QueryState<(
        FluidBindGroupsQueryData,
        &'static FluidStatus,
        &'static FluidSettings,
    )>,
    query_fluid_status: QueryState<
        (Entity, Option<&'static mut FluidStatus>),
        (With<FluidSettings>, With<InitializeBindGroups>),
    >,
}

impl EulerFluidNode {
    pub fn new(world: &mut World) -> Self {
        Self {
            state: State::Loading,
            fluid_query: world.query_filtered(),
            query_fluid_status: world.query_filtered(),
        }
    }
}

impl render_graph::Node for EulerFluidNode {
    fn update(&mut self, world: &mut World) {
        self.fluid_query.update_archetypes(world);
        let pipeline_cache = world.resource::<PipelineCache>();
        match self.state {
            State::Loading => {
                let initialize_pipeline = world.resource::<InitializePipeline>();
                let update_solid_pipeline = world.resource::<UpdateSolidPipeline>();
                let advection_pipeline = world.resource::<AdvectionPipeline>();
                let divergence_pipeline = world.resource::<DivergencePipeline>();
                let apply_forcces_pipeline = world.resource::<ApplyForcesPipeline>();
                let solve_pressure_pipeline = world.resource::<SolvePressurePipeline>();
                let solve_velocity_pipeline = world.resource::<SolveVelocityPipeline>();
                let extrapolate_velocity_pipeline = world.resource::<ExtrapolateVelocityPipeline>();
                let advect_scalar_pipeline = world.resource::<AdvectScalarPipeline>();
                let reinit_levelset_pipeline = world.resource::<ReinitLevelsetPipeline>();
                let fluid_to_solid_forces_pipeline = world.resource::<FluidToSolidForcesPipeline>();

                if initialize_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && update_solid_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && advection_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && apply_forcces_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && divergence_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && solve_pressure_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && solve_velocity_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && extrapolate_velocity_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && advect_scalar_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && reinit_levelset_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && fluid_to_solid_forces_pipeline.is_pipeline_state_ready(pipeline_cache)
                {
                    self.state = State::Init;
                }
            }
            State::Init => {
                self.state = State::Update;
            }
            State::Update | State::Idle => {
                let current_step = world.resource::<CurrentPhysicsStepNumberRenderWorld>();
                let physics_step_numper = world.resource::<PhysicsFrameInfo>().step_number;
                if current_step.0 == physics_step_numper {
                    self.state = State::Idle;
                } else {
                    let mut current_step =
                        world.resource_mut::<CurrentPhysicsStepNumberRenderWorld>();
                    current_step.0 = physics_step_numper;
                    self.state = State::Update;
                }

                for (_entity, fluid_status) in self.query_fluid_status.iter_mut(world) {
                    if let Some(mut fluid_status) = fluid_status {
                        match *fluid_status {
                            FluidStatus::Uninitialized => {
                                *fluid_status = FluidStatus::Initialized;
                            }
                            FluidStatus::Initialized => {}
                            FluidStatus::Reset => {
                                *fluid_status = FluidStatus::Uninitialized;
                            }
                        }
                    }
                }
            }
        }
    }
    fn run<'w>(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();

        match self.state {
            State::Loading => {}
            State::Init => {}
            State::Update => {
                for (bind_groups, fluid_status, fluid_settings) in
                    self.fluid_query.iter_manual(world)
                {
                    match fluid_status {
                        FluidStatus::Uninitialized => {
                            let mut pass = render_context.command_encoder().begin_compute_pass(
                                &ComputePassDescriptor {
                                    label: Some("Initialize fluid"),
                                    ..default()
                                },
                            );

                            let initialize_pipeline = world.resource::<InitializePipeline>();
                            initialize(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.initialize_bind_groups,
                                bind_groups.simulation_uniform,
                                initialize_pipeline,
                                fluid_settings.size,
                            );
                        }
                        FluidStatus::Initialized => {
                            let mut pass = render_context.command_encoder().begin_compute_pass(
                                &ComputePassDescriptor {
                                    label: Some("Eulerian fluid"),
                                    ..default()
                                },
                            );
                            let update_solid_pipeline = world.resource::<UpdateSolidPipeline>();
                            let obstacles_bind_groups =
                                world.resource::<SolidObstaclesBindGroups>();
                            update_solid(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.update_solid_bind_groups,
                                obstacles_bind_groups,
                                bind_groups.simulation_uniform,
                                update_solid_pipeline,
                                fluid_settings.size,
                            );

                            let advection_pipeline = world.resource::<AdvectionPipeline>();
                            advection(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.advection_bind_groups,
                                bind_groups.simulation_uniform,
                                advection_pipeline,
                                fluid_settings.size,
                            );

                            let apply_forces_pipeline = world.resource::<ApplyForcesPipeline>();
                            apply_forces(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.apply_forces_bind_groups,
                                bind_groups.simulation_uniform,
                                apply_forces_pipeline,
                                fluid_settings.size,
                            );

                            let divergence_pipeline = world.resource::<DivergencePipeline>();
                            divergence(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.divergence_bind_groups,
                                divergence_pipeline,
                                fluid_settings.size,
                            );

                            let solve_pressure_pipeline = world.resource::<SolvePressurePipeline>();
                            solve_pressure(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.solve_pressure_bind_groups,
                                bind_groups.simulation_uniform,
                                solve_pressure_pipeline,
                                fluid_settings.size,
                            );

                            let solve_velocity_pipeline = world.resource::<SolveVelocityPipeline>();
                            solve_velocity(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.solve_velocity_bind_groups,
                                bind_groups.simulation_uniform,
                                solve_velocity_pipeline,
                                fluid_settings.size,
                            );

                            let extrapolate_velocity_pipeline =
                                world.resource::<ExtrapolateVelocityPipeline>();
                            extrapolate_velocity(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.extrapolate_velocity_bind_groups,
                                extrapolate_velocity_pipeline,
                                fluid_settings.size,
                            );

                            let advect_scalar_pipeline = world.resource::<AdvectScalarPipeline>();
                            advect_scalar(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.advect_scalar_bind_groups,
                                bind_groups.simulation_uniform,
                                advect_scalar_pipeline,
                                fluid_settings.size,
                            );

                            let reinit_levelset_pipeline =
                                world.resource::<ReinitLevelsetPipeline>();
                            reinitialize_levelset(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.reinit_levelset_bind_groups,
                                reinit_levelset_pipeline,
                                fluid_settings.size,
                            );

                            let fluid_to_solid_forces_pipeline =
                                world.resource::<FluidToSolidForcesPipeline>();
                            fluid_to_solid_forces(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.fluid_to_solid_bind_groups,
                                obstacles_bind_groups,
                                bind_groups.simulation_uniform,
                                fluid_to_solid_forces_pipeline,
                                fluid_settings.size,
                            );
                        }
                        _ => {}
                    }
                }
            }
            State::Idle => {}
        }

        Ok(())
    }
}

fn initialize(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    initialize_bind_groups: &InitializeBindGroups,
    uniform_bind_group: &SimulationUniformBindGroup,
    pipeline: &InitializePipeline,
    size: UVec2,
) {
    pass.push_debug_group("Initialize simulation");
    let initialize_velocity_pipeline = pipeline_cache
        .get_compute_pipeline(pipeline.init_velocity_pipeline)
        .unwrap();
    let initialize_grid_center_pipeline = pipeline_cache
        .get_compute_pipeline(pipeline.init_grid_center_pipeline)
        .unwrap();

    pass.set_pipeline(&initialize_velocity_pipeline);
    pass.set_bind_group(0, &initialize_bind_groups.init_velocity_bind_group, &[]);
    pass.dispatch_x_major(size);

    pass.set_pipeline(&initialize_grid_center_pipeline);
    pass.set_bind_group(0, &initialize_bind_groups.init_grid_center_bind_group, &[]);
    pass.set_bind_group(
        1,
        &uniform_bind_group.bind_group,
        &[uniform_bind_group.index],
    );
    pass.dispatch_center(size);
    pass.pop_debug_group();
}

fn advection(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    advection_bind_groups: &AdvectionBindGroups,
    uniform_bind_group: &SimulationUniformBindGroup,
    advection_pipeline: &AdvectionPipeline,
    size: UVec2,
) {
    pass.push_debug_group("Advect velocity");
    let advect_u_pipeline = pipeline_cache
        .get_compute_pipeline(advection_pipeline.advect_u_pipeline)
        .unwrap();
    let advect_v_pipeline = pipeline_cache
        .get_compute_pipeline(advection_pipeline.advect_v_pipeline)
        .unwrap();

    pass.set_pipeline(&advect_u_pipeline);
    pass.set_bind_group(0, &advection_bind_groups.advection_bind_group, &[]);
    pass.set_bind_group(
        1,
        &uniform_bind_group.bind_group,
        &[uniform_bind_group.index],
    );
    pass.dispatch_x_major(size);

    pass.set_pipeline(&advect_v_pipeline);
    pass.dispatch_y_major(size);
    pass.pop_debug_group();
}

fn apply_forces(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    apply_forces_bind_groups: &ApplyForcesBindGroups,
    uniform_bind_group: &SimulationUniformBindGroup,
    apply_forces_pipeline: &ApplyForcesPipeline,
    size: UVec2,
) {
    pass.push_debug_group("Apply forces to fluid");
    let apply_forces_u_pipeline = pipeline_cache
        .get_compute_pipeline(apply_forces_pipeline.apply_forces_u_pipeline)
        .unwrap();
    let apply_forces_v_pipeline = pipeline_cache
        .get_compute_pipeline(apply_forces_pipeline.apply_forces_v_pipeline)
        .unwrap();

    pass.set_bind_group(0, &apply_forces_bind_groups.apply_forces_bind_group, &[]);
    pass.set_bind_group(
        1,
        &uniform_bind_group.bind_group,
        &[uniform_bind_group.index],
    );

    pass.set_pipeline(&apply_forces_u_pipeline);
    pass.dispatch_x_major(size);

    pass.set_pipeline(&apply_forces_v_pipeline);
    pass.dispatch_y_major(size);
    pass.pop_debug_group();
}

fn divergence(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    divergence_bind_groups: &DivergenceBindGroups,
    divergence_pipeline: &DivergencePipeline,
    size: UVec2,
) {
    pass.push_debug_group("Divergence");
    let divergence_pipeline = pipeline_cache
        .get_compute_pipeline(divergence_pipeline.divergence_pipeline)
        .unwrap();

    pass.set_pipeline(&divergence_pipeline);
    pass.set_bind_group(0, &divergence_bind_groups.divergence_bind_group, &[]);
    pass.dispatch_center(size);
    pass.pop_debug_group();
}

fn solve_pressure(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    solve_pressure_bind_groups: &SolvePressureBindGroups,
    uniform_bind_group: &SimulationUniformBindGroup,
    solve_pressure_pipeline: &SolvePressurePipeline,
    size: UVec2,
) {
    pass.push_debug_group("Solve pressure");
    let jacobi_iteration_pipeline = pipeline_cache
        .get_compute_pipeline(solve_pressure_pipeline.jacobi_iteration_pipeline)
        .unwrap();
    let jacobi_iteration_reverse_pipeline = pipeline_cache
        .get_compute_pipeline(solve_pressure_pipeline.jacobi_iteration_reverse_pipeline)
        .unwrap();

    pass.set_bind_group(
        1,
        &uniform_bind_group.bind_group,
        &[uniform_bind_group.index],
    );
    for _ in 0..50 {
        pass.set_pipeline(&jacobi_iteration_pipeline);
        pass.set_bind_group(
            0,
            &solve_pressure_bind_groups.jacobi_iteration_bind_group,
            &[],
        );
        pass.dispatch_center(size);

        pass.set_pipeline(&jacobi_iteration_reverse_pipeline);
        pass.set_bind_group(
            0,
            &solve_pressure_bind_groups.jacobi_iteration_reverse_bind_group,
            &[],
        );
        pass.dispatch_center(size);
    }
    pass.pop_debug_group();
}

fn solve_velocity(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    solve_velocity_bind_groups: &SolveVelocityBindGroups,
    uniform_bind_group: &SimulationUniformBindGroup,
    solve_velocity_pipeline: &SolveVelocityPipeline,
    size: UVec2,
) {
    pass.push_debug_group("Solve velocity");
    let solve_velocity_u_pipeline = pipeline_cache
        .get_compute_pipeline(solve_velocity_pipeline.solve_u_pipeline)
        .unwrap();
    let solve_velocity_v_pipeline = pipeline_cache
        .get_compute_pipeline(solve_velocity_pipeline.solve_v_pipeline)
        .unwrap();

    pass.set_pipeline(&solve_velocity_u_pipeline);
    pass.set_bind_group(0, &solve_velocity_bind_groups.solve_u_bind_group, &[]);
    pass.set_bind_group(
        1,
        &uniform_bind_group.bind_group,
        &[uniform_bind_group.index],
    );
    pass.dispatch_x_major(size);

    pass.set_pipeline(&solve_velocity_v_pipeline);
    pass.set_bind_group(0, &solve_velocity_bind_groups.solve_v_bind_group, &[]);
    pass.dispatch_y_major(size);
    pass.pop_debug_group();
}

fn extrapolate_velocity(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    extrapolate_velocity_bind_groups: &ExtrapolateVelocityBindGroups,
    extrapolate_velocity_pipeline: &ExtrapolateVelocityPipeline,
    size: UVec2,
) {
    pass.push_debug_group("Extrapolate velocity");
    let extrapolate_u_pipeline = pipeline_cache
        .get_compute_pipeline(extrapolate_velocity_pipeline.extrapolate_u_pipeline)
        .unwrap();
    let extrapolate_v_pipeline = pipeline_cache
        .get_compute_pipeline(extrapolate_velocity_pipeline.extrapolate_v_pipeline)
        .unwrap();

    pass.set_bind_group(
        0,
        &extrapolate_velocity_bind_groups.extrapolate_velocity_bind_group,
        &[],
    );

    pass.set_pipeline(&extrapolate_u_pipeline);
    pass.dispatch_x_major(size);
    pass.set_pipeline(&extrapolate_v_pipeline);
    pass.dispatch_y_major(size);
    pass.pop_debug_group();
}

fn advect_scalar(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    advect_scalar_bind_groups: &AdvectScalarBindGroups,
    uniform_bind_group: &SimulationUniformBindGroup,
    advect_scalar_pipeline: &AdvectScalarPipeline,
    size: UVec2,
) {
    pass.push_debug_group("Advect scalar");
    let advect_levelset_pipeline = pipeline_cache
        .get_compute_pipeline(advect_scalar_pipeline.advect_levelset_pipeline)
        .unwrap();

    pass.set_bind_group(
        0,
        &advect_scalar_bind_groups.advect_levelset_bind_group,
        &[],
    );
    pass.set_bind_group(
        1,
        &uniform_bind_group.bind_group,
        &[uniform_bind_group.index],
    );

    pass.set_pipeline(&advect_levelset_pipeline);
    pass.dispatch_center(size);
    pass.pop_debug_group();
}

fn reinitialize_levelset(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    bind_groups: &ReinitLevelsetBindGroups,
    pipeline: &ReinitLevelsetPipeline,
    size: UVec2,
) {
    pass.push_debug_group("Reinitialize levelset");
    let init_seeds_pipeline = pipeline_cache
        .get_compute_pipeline(pipeline.init_seeds_pipeline)
        .unwrap();
    let iterate_pipeline = pipeline_cache
        .get_compute_pipeline(pipeline.iterate_pipeline)
        .unwrap();
    let sdf_pipeline = pipeline_cache
        .get_compute_pipeline(pipeline.sdf_pipeline)
        .unwrap();

    pass.set_pipeline(init_seeds_pipeline);
    pass.set_bind_group(0, &bind_groups.init_seeds_bind_group, &[]);
    pass.dispatch_center(size);

    pass.set_pipeline(&iterate_pipeline);
    pass.set_bind_group(0, &bind_groups.iterate_bind_group, &[]);
    for bind_group in &bind_groups.jump_flooding_step_bind_groups {
        pass.set_bind_group(1, bind_group, &[]);
        pass.dispatch_center(size);
    }

    pass.set_pipeline(&sdf_pipeline);
    pass.set_bind_group(0, &bind_groups.sdf_bind_group, &[]);
    pass.dispatch_center(size);
    pass.pop_debug_group();
}

fn fluid_to_solid_forces(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    bind_groups: &FluidToSolidForcesBindGroups,
    obstacles_bind_groups: &SolidObstaclesBindGroups,
    uniform_bind_group: &SimulationUniformBindGroup,
    pipeline: &FluidToSolidForcesPipeline,
    size: UVec2,
) {
    pass.push_debug_group("Fluid to solid forces");
    let sample_forces_pipeline = pipeline_cache
        .get_compute_pipeline(pipeline.sample_forces_pipeline)
        .unwrap();
    let accumulate_forces_pipeline = pipeline_cache
        .get_compute_pipeline(pipeline.accumulate_forces_pipeline)
        .unwrap();

    pass.set_pipeline(&sample_forces_pipeline);
    pass.set_bind_group(0, &bind_groups.sample_forces_bind_group, &[]);
    pass.set_bind_group(1, &obstacles_bind_groups.solid_obstacles_bind_group, &[]);
    pass.set_bind_group(
        2,
        &uniform_bind_group.bind_group,
        &[uniform_bind_group.index],
    );
    pass.dispatch_center(size);

    pass.set_pipeline(&accumulate_forces_pipeline);
    pass.set_bind_group(0, &bind_groups.accumulate_forces_bind_group, &[]);
    pass.dispatch_workgroups(MAX_SOLIDS as u32, 1, 1);
    pass.pop_debug_group();
}

fn update_solid(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    bind_groups: &UpdateSolidBindGroups,
    obstacles_bind_groups: &SolidObstaclesBindGroups,
    uniform_bind_group: &SimulationUniformBindGroup,
    pipeline: &UpdateSolidPipeline,
    size: UVec2,
) {
    pass.push_debug_group("Update solid boundary");
    let update_solid_pipeline = pipeline_cache
        .get_compute_pipeline(pipeline.update_solid_pipeline)
        .unwrap();
    let update_solid_pressure_pipeline = pipeline_cache
        .get_compute_pipeline(pipeline.update_solid_pressure_pipeline)
        .unwrap();

    pass.set_pipeline(&update_solid_pipeline);
    pass.set_bind_group(0, &bind_groups.update_solid_bind_group, &[]);
    pass.set_bind_group(1, &obstacles_bind_groups.solid_obstacles_bind_group, &[]);
    pass.set_bind_group(
        2,
        &uniform_bind_group.bind_group,
        &[uniform_bind_group.index],
    );
    pass.dispatch_center(size);

    pass.set_pipeline(&update_solid_pressure_pipeline);
    pass.set_bind_group(0, &bind_groups.update_solid_pressure_bind_group, &[]);
    pass.dispatch_center(size);
    pass.pop_debug_group();
}
