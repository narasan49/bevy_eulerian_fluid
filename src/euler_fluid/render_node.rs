use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::{
        render_graph::{self, RenderLabel},
        render_resource::{ComputePass, ComputePassDescriptor, PipelineCache},
    },
};

use crate::{
    advect_levelset::{AdvectLevelSetBindGroups, AdvectLevelSetPipeline},
    advection::{self, AdvectionBindGroup, AdvectionPipeline},
    apply_forces::{ApplyForcesBindGroups, ApplyForcesPipeline},
    divergence::{DivergenceBindGroup, DivergencePipeline},
    extrapolate_velocity::{ExtrapolateVelocityBindGroups, ExtrapolateVelocityPipeline},
    fluid_status::FluidStatus,
    fluid_to_solid::{
        FluidToSolidForcesBindGroups, FluidToSolidForcesPipeline, SolidObstaclesBindGroups,
        MAX_SOLIDS,
    },
    fluid_uniform::SimulationUniformBindGroup,
    initialize::{
        InitializeGridCenterBindGroup, InitializeGridCenterPipeline, InitializeGridEdgeBindGroup,
        InitializeGridEdgePipeline,
    },
    levelset_gradient::{LevelSetGradientBindGroup, LevelSetGradientPipeline},
    particle_levelset_two_layers::{
        self,
        levelset_correction::{PLSLevelsetCorrectionQuery, PLSLevelsetCorrectionSecondQuery},
        plugin::{
            are_pls_pipelines_ready, PLSAdvectionBindGroupsQuery, PLSInitializeBindGroupsQuery,
        },
        reseed::PLSReseedBindGroupsQuery,
    },
    physics_time::{CurrentPhysicsStepNumberRenderWorld, PhysicsFrameInfo},
    pipeline::{DispatchFluidPass, Pipeline, WORKGROUP_SIZE},
    projection::{
        self, gauss_seidel::GaussSeidelPipeline, ProjectionBindGroupsQuery, ProjectionMethod,
    },
    reinitialize_levelset::{self, ReinitializeLevelSetBindGroupQuery, ReinitializeMethod},
    settings::FluidSettings,
    solve_pressure::SolvePressurePipeline,
    solve_velocity::{SolveVelocityBindGroups, SolveVelocityPipeline},
    update_area_fraction::{UpdateAreaFractionBindGroup, UpdateAreaFractionPipeline},
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
    initialize_center_bind_group: &'static InitializeGridCenterBindGroup,
    initialize_edge_bind_group: &'static InitializeGridEdgeBindGroup,
    update_solid_bind_groups: &'static UpdateSolidBindGroups,
    update_area_fraction_bind_group: &'static UpdateAreaFractionBindGroup,
    advection_bind_groups: &'static AdvectionBindGroup,
    apply_forces_bind_groups: &'static ApplyForcesBindGroups,
    divergence_bind_groups: &'static DivergenceBindGroup,
    solve_velocity_bind_groups: &'static SolveVelocityBindGroups,
    extrapolate_velocity_bind_groups: &'static ExtrapolateVelocityBindGroups,
    advect_levelset_bind_groups: &'static AdvectLevelSetBindGroups,
    reinit_levelset_bind_groups: ReinitializeLevelSetBindGroupQuery,
    fluid_to_solid_bind_groups: &'static FluidToSolidForcesBindGroups,
    simulation_uniform: &'static SimulationUniformBindGroup,
    levelset_gradient_bind_group: &'static LevelSetGradientBindGroup,
    projection_bind_groups: ProjectionBindGroupsQuery,
}

pub(crate) struct EulerFluidNode {
    state: State,
    // Query BindGroups components
    // Reference: bevy\crates\bevy_ui\src\render\render_pass.rs
    fluid_query: QueryState<(
        FluidBindGroupsQueryData,
        Option<PLSInitializeBindGroupsQuery>,
        Option<PLSAdvectionBindGroupsQuery>,
        Option<PLSLevelsetCorrectionQuery>,
        Option<PLSLevelsetCorrectionSecondQuery>,
        Option<PLSReseedBindGroupsQuery>,
        &'static FluidStatus,
        &'static FluidSettings,
        &'static ProjectionMethod,
        &'static ReinitializeMethod,
    )>,
    query_fluid_status: QueryState<(Entity, Option<&'static mut FluidStatus>), With<FluidSettings>>,
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
                let initialize_center_pipeline = world.resource::<InitializeGridCenterPipeline>();
                let initialize_edge_pipeline = world.resource::<InitializeGridEdgePipeline>();

                let update_solid_pipeline = world.resource::<UpdateSolidPipeline>();
                let update_area_fraction_pipeline = world.resource::<UpdateAreaFractionPipeline>();
                let advection_pipeline = world.resource::<AdvectionPipeline>();
                let divergence_pipeline = world.resource::<DivergencePipeline>();
                let apply_forces_pipeline = world.resource::<ApplyForcesPipeline>();
                let solve_pressure_pipeline = world.resource::<SolvePressurePipeline>();
                let gauss_seidel_pipeline = world.resource::<GaussSeidelPipeline>();
                let solve_velocity_pipeline = world.resource::<SolveVelocityPipeline>();
                let extrapolate_velocity_pipeline = world.resource::<ExtrapolateVelocityPipeline>();
                let advect_levelset_pipeline = world.resource::<AdvectLevelSetPipeline>();
                let fluid_to_solid_forces_pipeline = world.resource::<FluidToSolidForcesPipeline>();

                if initialize_center_pipeline.pipeline.is_ready(pipeline_cache)
                    && initialize_edge_pipeline.pipeline.is_ready(pipeline_cache)
                    && update_solid_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && update_area_fraction_pipeline
                        .pipeline
                        .is_ready(pipeline_cache)
                    && advection_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && apply_forces_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && divergence_pipeline.pipeline.is_ready(pipeline_cache)
                    && solve_pressure_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && gauss_seidel_pipeline.is_ready(pipeline_cache)
                    && solve_velocity_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && extrapolate_velocity_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && advect_levelset_pipeline.pipeline.is_ready(pipeline_cache)
                    && reinitialize_levelset::is_pipeline_ready(world, pipeline_cache)
                    && fluid_to_solid_forces_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && are_pls_pipelines_ready(world, pipeline_cache)
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
                for (
                    bind_groups,
                    pls_init_bind_groups,
                    pls_update_bind_groups,
                    pls_correct_levelset_bind_groups,
                    pls_correct_levelset_second_bind_groups,
                    pls_reseed_bind_groups,
                    fluid_status,
                    fluid_settings,
                    projection_method,
                    reinitialize_method,
                ) in self.fluid_query.iter_manual(world)
                {
                    match fluid_status {
                        FluidStatus::Uninitialized => {
                            let mut pass = render_context.command_encoder().begin_compute_pass(
                                &ComputePassDescriptor {
                                    label: Some("Initialize fluid"),
                                    ..default()
                                },
                            );
                            let num_workgroups_grid =
                                (fluid_settings.size / WORKGROUP_SIZE).extend(1);
                            let num_workgroups_x_edge = ((fluid_settings.size + UVec2::X)
                                / UVec2::new(1, WORKGROUP_SIZE * WORKGROUP_SIZE))
                            .extend(1);

                            let initialize_center_pipeline =
                                world.resource::<InitializeGridCenterPipeline>();
                            initialize_center_pipeline.pipeline.dispatch_with_uniform(
                                pipeline_cache,
                                &mut pass,
                                &bind_groups.initialize_center_bind_group.bind_group,
                                bind_groups.simulation_uniform,
                                num_workgroups_grid,
                            );

                            let initialize_edge_pipeline =
                                world.resource::<InitializeGridEdgePipeline>();
                            initialize_edge_pipeline.pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                &bind_groups.initialize_edge_bind_group.bind_group,
                                num_workgroups_x_edge,
                            );

                            if let Some(pls_init_bind_groups) = pls_init_bind_groups {
                                particle_levelset_two_layers::plugin::dispatch_initialize(
                                    world,
                                    pipeline_cache,
                                    &mut pass,
                                    pls_init_bind_groups,
                                    fluid_settings.size,
                                );
                            }
                        }
                        FluidStatus::Initialized => {
                            let mut pass = render_context.command_encoder().begin_compute_pass(
                                &ComputePassDescriptor {
                                    label: Some("Eulerian fluid"),
                                    ..default()
                                },
                            );
                            let num_workgroups_grid =
                                (fluid_settings.size / WORKGROUP_SIZE).extend(1);

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

                            let update_area_fraction_pipeline =
                                world.resource::<UpdateAreaFractionPipeline>();
                            update_area_fraction_pipeline.pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                &bind_groups.update_area_fraction_bind_group.bind_group,
                                num_workgroups_grid,
                            );

                            let advection_pipeline = world.resource::<AdvectionPipeline>();
                            advection::dispatch(
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
                            divergence_pipeline.pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                &bind_groups.divergence_bind_groups.bind_group,
                                num_workgroups_grid,
                            );

                            projection::dispatch(
                                world,
                                projection_method,
                                pipeline_cache,
                                &mut pass,
                                bind_groups.projection_bind_groups,
                                bind_groups.simulation_uniform,
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

                            let advect_levelset_pipeline =
                                world.resource::<AdvectLevelSetPipeline>();
                            advect_levelset_pipeline.pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                &bind_groups.advect_levelset_bind_groups.bind_group,
                                num_workgroups_grid,
                            );

                            if let Some(pls_update_bind_groups) = pls_update_bind_groups {
                                particle_levelset_two_layers::plugin::dispatch_update(
                                    world,
                                    pipeline_cache,
                                    &mut pass,
                                    pls_update_bind_groups,
                                    bind_groups.simulation_uniform,
                                    fluid_settings.size,
                                );
                            }

                            if let Some(correct_levelset_bind_groups) =
                                pls_correct_levelset_bind_groups
                            {
                                particle_levelset_two_layers::levelset_correction::dispatch(
                                    world,
                                    pipeline_cache,
                                    &mut pass,
                                    correct_levelset_bind_groups,
                                    fluid_settings.size,
                                );
                            }

                            reinitialize_levelset::dispatch(
                                world,
                                reinitialize_method,
                                pipeline_cache,
                                &mut pass,
                                bind_groups.reinit_levelset_bind_groups,
                                fluid_settings.size,
                            );

                            if let Some(correct_levelset_second_bind_groups) =
                                pls_correct_levelset_second_bind_groups
                            {
                                particle_levelset_two_layers::levelset_correction::dispatch_second(
                                    world,
                                    pipeline_cache,
                                    &mut pass,
                                    correct_levelset_second_bind_groups,
                                    fluid_settings.size,
                                );
                            }

                            if let Some(reseed_bind_groups) = pls_reseed_bind_groups {
                                particle_levelset_two_layers::reseed::dispatch(
                                    world,
                                    pipeline_cache,
                                    &mut pass,
                                    reseed_bind_groups,
                                    bind_groups.simulation_uniform,
                                    fluid_settings.size,
                                );
                            }

                            let levelset_gradient_pipeline =
                                world.resource::<LevelSetGradientPipeline>();
                            levelset_gradient_pipeline.pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                &bind_groups.levelset_gradient_bind_group.bind_group,
                                num_workgroups_grid,
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
    let initialize_u_valid_pipeline = pipeline_cache
        .get_compute_pipeline(extrapolate_velocity_pipeline.initialize_u_valid_pipeline)
        .unwrap();
    let initialize_v_valid_pipeline = pipeline_cache
        .get_compute_pipeline(extrapolate_velocity_pipeline.initialize_v_valid_pipeline)
        .unwrap();
    let extrapolate_u_pipeline = pipeline_cache
        .get_compute_pipeline(extrapolate_velocity_pipeline.extrapolate_u_pipeline)
        .unwrap();
    let extrapolate_v_pipeline = pipeline_cache
        .get_compute_pipeline(extrapolate_velocity_pipeline.extrapolate_v_pipeline)
        .unwrap();

    pass.set_pipeline(&initialize_u_valid_pipeline);
    pass.set_bind_group(
        0,
        &extrapolate_velocity_bind_groups.initialize_u_valid_bind_group,
        &[],
    );
    pass.dispatch_x_major(size);

    pass.set_pipeline(&initialize_v_valid_pipeline);
    pass.set_bind_group(
        0,
        &extrapolate_velocity_bind_groups.initialize_v_valid_bind_group,
        &[],
    );
    pass.dispatch_y_major(size);

    for _ in 0..(10 / 2) {
        pass.set_pipeline(&extrapolate_u_pipeline);
        pass.set_bind_group(
            0,
            &extrapolate_velocity_bind_groups.extrapolate_u_bind_group,
            &[],
        );
        pass.dispatch_x_major(size);
        pass.set_bind_group(
            0,
            &extrapolate_velocity_bind_groups.extrapolate_u_reverse_bind_group,
            &[],
        );
        pass.dispatch_x_major(size);

        pass.set_pipeline(&extrapolate_v_pipeline);
        pass.set_bind_group(
            0,
            &extrapolate_velocity_bind_groups.extrapolate_v_bind_group,
            &[],
        );
        pass.dispatch_y_major(size);
        pass.set_bind_group(
            0,
            &extrapolate_velocity_bind_groups.extrapolate_v_reverse_bind_group,
            &[],
        );
        pass.dispatch_y_major(size);
    }

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

    pass.set_pipeline(&update_solid_pipeline);
    pass.set_bind_group(0, &bind_groups.update_solid_bind_group, &[]);
    pass.set_bind_group(1, &obstacles_bind_groups.solid_obstacles_bind_group, &[]);
    pass.set_bind_group(
        2,
        &uniform_bind_group.bind_group,
        &[uniform_bind_group.index],
    );
    pass.dispatch_center(size);

    pass.pop_debug_group();
}
