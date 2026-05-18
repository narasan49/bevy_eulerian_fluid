use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::{
        diagnostic::RecordDiagnostics,
        render_graph::{self, RenderLabel},
        render_resource::{ComputePassDescriptor, PipelineCache},
    },
};

use crate::{
    core::{
        advect_levelset::{AdvectLevelSetBindGroup, AdvectLevelSetPipeline},
        advect_velocity::{AdvectVelocityBindGroup, AdvectVelocityPipeline},
        apply_forces::{ApplyForcesBindGroup, ApplyForcesPipeline},
        divergence::{DivergenceBindGroup, DivergencePipeline},
        extrapolate_velocity::{ExtrapolateVelocityBindGroups, ExtrapolateVelocityPipeline},
        fluid_source::update_fluid_source::{
            UpdateFluidSourceBindGroupsQuery, UpdateFluidSourcePipeline,
        },
        fluid_uniform::FluidUniformBindGroup,
        initialize_resources::{InitializeResourcesBindGroup, InitializeResourcesPipeline},
        projection::{
            self, gauss_seidel::GaussSeidelPipeline, multigrid::MultiGridPipelines,
            ProjectionBindGroupsQuery, ProjectionMethod,
        },
        reinitialize_levelset::{self, ReinitializeLevelSetBindGroupQuery, ReinitializeMethod},
        solid_body::{
            update_area_fraction_solid::{UpdateAreaFractionBindGroup, UpdateAreaFractionPipeline},
            update_solid::{UpdateSolidBindGroup, UpdateSolidPipeline},
        },
        solve_u::{SolveUBindGroup, SolveUPipeline},
        solve_v::{SolveVBindGroup, SolveVPipeline},
        solve_w::{SolveWBindGroup, SolveWPipeline},
        workgroup::WorkgroupShape,
    },
    fluid_status::FluidStatus,
    resource::EulerFluid3d,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct FluidLabel;

#[derive(Debug)]
enum State {
    Loading,
    Update,
}

#[derive(QueryData)]
struct FluidBindGroupsQueryData {
    initialize_resources_bind_group: &'static InitializeResourcesBindGroup,
    update_solid_bind_group: &'static UpdateSolidBindGroup,
    update_area_fraction_bind_group: &'static UpdateAreaFractionBindGroup,
    advect_velocity_bind_group: &'static AdvectVelocityBindGroup,
    apply_forces_bind_group: &'static ApplyForcesBindGroup,
    divergence_bind_group: &'static DivergenceBindGroup,
    solve_u_bind_group: &'static SolveUBindGroup,
    solve_v_bind_group: &'static SolveVBindGroup,
    solve_w_bind_group: &'static SolveWBindGroup,
    extrapolate_velocity_bind_groups: &'static ExtrapolateVelocityBindGroups,
    advect_levelset_bind_group: &'static AdvectLevelSetBindGroup,
    update_fluid_source_bind_groups: UpdateFluidSourceBindGroupsQuery,
    reinit_levelset_bind_groups: ReinitializeLevelSetBindGroupQuery,
    // fluid_to_solid_bind_groups: &'static FluidToSolidForcesBindGroups,
    fluid_uniform: &'static FluidUniformBindGroup,
    // levelset_gradient_bind_group: &'static LevelSetGradientBindGroup,
    projection_bind_groups: ProjectionBindGroupsQuery,
}

pub(crate) struct EulerFluidNode {
    state: State,
    // Query BindGroups components
    // Reference: bevy\crates\bevy_ui\src\render\render_pass.rs
    fluid_query: QueryState<(
        FluidBindGroupsQueryData,
        &'static EulerFluid3d,
        &'static ProjectionMethod,
        &'static ReinitializeMethod,
        &'static FluidStatus,
    )>,
    fluid_status_query: QueryState<
        &'static mut FluidStatus,
        (With<EulerFluid3d>, With<InitializeResourcesBindGroup>),
    >,
}

impl FromWorld for EulerFluidNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            state: State::Loading,
            fluid_query: world.query_filtered(),
            fluid_status_query: world.query_filtered(),
        }
    }
}

impl render_graph::Node for EulerFluidNode {
    fn update(&mut self, world: &mut World) {
        self.fluid_query.update_archetypes(world);
        self.fluid_status_query.update_archetypes(world);
        let pipeline_cache = world.resource::<PipelineCache>();
        match self.state {
            State::Loading => {
                info_once!("[once] loading fluid node");
                let initialize_resources_pipeline = world.resource::<InitializeResourcesPipeline>();

                let update_solid_pipeline = world.resource::<UpdateSolidPipeline>();
                let update_area_fraction_pipeline = world.resource::<UpdateAreaFractionPipeline>();
                let advect_velocity_pipeline = world.resource::<AdvectVelocityPipeline>();
                let divergence_pipeline = world.resource::<DivergencePipeline>();
                let apply_forces_pipeline = world.resource::<ApplyForcesPipeline>();
                let gauss_seidel_pipeline = world.resource::<GaussSeidelPipeline>();
                let multi_grid_pipeline = world.resource::<MultiGridPipelines>();
                let solve_u_pipeline = world.resource::<SolveUPipeline>();
                let solve_v_pipeline = world.resource::<SolveVPipeline>();
                let solve_w_pipeline = world.resource::<SolveWPipeline>();
                let extrapolate_velocity_pipeline = world.resource::<ExtrapolateVelocityPipeline>();
                let advect_levelset_pipeline = world.resource::<AdvectLevelSetPipeline>();
                // let fluid_to_solid_forces_pipeline = world.resource::<FluidToSolidForcesPipeline>();
                let update_fluid_source_pipeline = world.resource::<UpdateFluidSourcePipeline>();

                if initialize_resources_pipeline
                    .is_ready(pipeline_cache)
                    && update_solid_pipeline.is_ready(pipeline_cache)
                    && update_area_fraction_pipeline.is_ready(pipeline_cache)
                    && advect_velocity_pipeline.is_ready(pipeline_cache)
                    && apply_forces_pipeline.is_ready(pipeline_cache)
                    && divergence_pipeline.is_ready(pipeline_cache)
                    && gauss_seidel_pipeline.is_ready(pipeline_cache)
                    && multi_grid_pipeline.is_ready(pipeline_cache)
                    && solve_u_pipeline.is_ready(pipeline_cache)
                    && solve_v_pipeline.is_ready(pipeline_cache)
                    && solve_w_pipeline.is_ready(pipeline_cache)
                    && extrapolate_velocity_pipeline.is_ready(pipeline_cache)
                    && advect_levelset_pipeline.is_ready(pipeline_cache)
                    && reinitialize_levelset::is_pipeline_ready(world, pipeline_cache)
                    // && fluid_to_solid_forces_pipeline.is_pipeline_state_ready(pipeline_cache)
                    && update_fluid_source_pipeline.is_ready(pipeline_cache)
                {
                    self.state = State::Update;
                }
            }
            State::Update => {
                // let current_step = world.resource::<CurrentPhysicsStepNumberRenderWorld>();
                // let physics_step_numper = world.resource::<PhysicsFrameInfo>().step_number;
                // if current_step.0 == physics_step_numper {
                //     self.state = State::Idle;
                // } else {
                //     let mut current_step =
                //         world.resource_mut::<CurrentPhysicsStepNumberRenderWorld>();
                //     current_step.0 = physics_step_numper;
                //     self.state = State::Update;
                // }

                for mut status in self.fluid_status_query.iter_mut(world) {
                    match *status {
                        FluidStatus::Initialize => {
                            *status = FluidStatus::Update;
                        }
                        FluidStatus::Update => {}
                        FluidStatus::Start => {
                            *status = FluidStatus::Initialize;
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
        let workgroup_shape = world.resource::<WorkgroupShape>();

        for (bind_groups, fluid3d, projection_method, reinitialize_method, fluid_status) in
            self.fluid_query.iter_manual(world)
        {
            match self.state {
                State::Loading => {}
                State::Update => {
                    match fluid_status {
                        FluidStatus::Initialize => {
                            info_once!("[once] initializing fluid");
                            let mut pass = render_context.command_encoder().begin_compute_pass(
                                &ComputePassDescriptor {
                                    label: Some("initialize_fluid3d"),
                                    ..default()
                                },
                            );

                            let initialize_center_pipeline =
                                world.resource::<InitializeResourcesPipeline>();
                            initialize_center_pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                &bind_groups.initialize_resources_bind_group,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            let update_fluid_source_pipeline =
                                world.resource::<UpdateFluidSourcePipeline>();
                            update_fluid_source_pipeline.dispatch_init(
                                pipeline_cache,
                                &mut pass,
                                &bind_groups.update_fluid_source_bind_groups,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            reinitialize_levelset::dispatch(
                                world,
                                reinitialize_method,
                                pipeline_cache,
                                &mut pass,
                                bind_groups.reinit_levelset_bind_groups,
                                workgroup_shape,
                                fluid3d.resolution,
                            );
                        }
                        FluidStatus::Update => {
                            info_once!("[once] running fluid node");
                            let diagnostics = render_context.diagnostic_recorder();
                            let mut pass = render_context.command_encoder().begin_compute_pass(
                                &ComputePassDescriptor {
                                    label: Some("eulerian_fluid3d"),
                                    ..default()
                                },
                            );
                            let pass_span = diagnostics.pass_span(&mut pass, "eulerian_fluid");

                            let update_solid_pipeline = world.resource::<UpdateSolidPipeline>();
                            // let obstacles_bind_groups = world.resource::<SolidObstaclesBindGroups>();
                            update_solid_pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.update_solid_bind_group,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            let update_area_fraction_pipeline =
                                world.resource::<UpdateAreaFractionPipeline>();
                            update_area_fraction_pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.update_area_fraction_bind_group,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            let advect_velocity_pipeline =
                                world.resource::<AdvectVelocityPipeline>();
                            advect_velocity_pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.advect_velocity_bind_group,
                                bind_groups.fluid_uniform,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            let apply_forces_pipeline = world.resource::<ApplyForcesPipeline>();
                            apply_forces_pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.apply_forces_bind_group,
                                bind_groups.fluid_uniform,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            let divergence_pipeline = world.resource::<DivergencePipeline>();
                            divergence_pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                &bind_groups.divergence_bind_group,
                                bind_groups.fluid_uniform,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            projection::dispatch(
                                world,
                                projection_method,
                                pipeline_cache,
                                &mut pass,
                                bind_groups.projection_bind_groups,
                                bind_groups.fluid_uniform,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            let solve_u_pipeline = world.resource::<SolveUPipeline>();
                            solve_u_pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.solve_u_bind_group,
                                bind_groups.fluid_uniform,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            let solve_v_pipeline = world.resource::<SolveVPipeline>();
                            solve_v_pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.solve_v_bind_group,
                                bind_groups.fluid_uniform,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            let solve_w_pipeline = world.resource::<SolveWPipeline>();
                            solve_w_pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.solve_w_bind_group,
                                bind_groups.fluid_uniform,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            let extrapolate_velocity_pipeline =
                                world.resource::<ExtrapolateVelocityPipeline>();
                            extrapolate_velocity_pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                bind_groups.extrapolate_velocity_bind_groups,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            let advect_levelset_pipeline =
                                world.resource::<AdvectLevelSetPipeline>();
                            advect_levelset_pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                &bind_groups.advect_levelset_bind_group,
                                &bind_groups.fluid_uniform,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            let update_fluid_source_pipeline =
                                world.resource::<UpdateFluidSourcePipeline>();
                            update_fluid_source_pipeline.dispatch(
                                pipeline_cache,
                                &mut pass,
                                &bind_groups.update_fluid_source_bind_groups,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            reinitialize_levelset::dispatch(
                                world,
                                reinitialize_method,
                                pipeline_cache,
                                &mut pass,
                                bind_groups.reinit_levelset_bind_groups,
                                workgroup_shape,
                                fluid3d.resolution,
                            );

                            // let fluid_to_solid_forces_pipeline =
                            //     world.resource::<FluidToSolidForcesPipeline>();
                            // fluid_to_solid_forces(
                            //     pipeline_cache,
                            //     &mut pass,
                            //     bind_groups.fluid_to_solid_bind_groups,
                            //     obstacles_bind_groups,
                            //     bind_groups.fluid_uniform,
                            //     fluid_to_solid_forces_pipeline,
                            //     fluid3d.size,
                            // );

                            pass_span.end(&mut pass);
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}

// fn fluid_to_solid_forces(
//     pipeline_cache: &PipelineCache,
//     pass: &mut ComputePass,
//     bind_groups: &FluidToSolidForcesBindGroups,
//     obstacles_bind_groups: &SolidObstaclesBindGroups,
//     uniform_bind_group: &SimulationUniformBindGroup,
//     pipeline: &FluidToSolidForcesPipeline,
//     size: UVec2,
// ) {
//     pass.push_debug_group("Fluid to solid forces");
//     let sample_forces_pipeline = pipeline_cache
//         .get_compute_pipeline(pipeline.sample_forces_pipeline)
//         .unwrap();
//     let accumulate_forces_pipeline = pipeline_cache
//         .get_compute_pipeline(pipeline.accumulate_forces_pipeline)
//         .unwrap();

//     pass.set_pipeline(&sample_forces_pipeline);
//     pass.set_bind_group(0, &bind_groups.sample_forces_bind_group, &[]);
//     pass.set_bind_group(1, &obstacles_bind_groups.solid_obstacles_bind_group, &[]);
//     pass.set_bind_group(
//         2,
//         &uniform_bind_group.bind_group,
//         &[uniform_bind_group.index],
//     );
//     pass.dispatch_center(size);

//     pass.set_pipeline(&accumulate_forces_pipeline);
//     pass.set_bind_group(0, &bind_groups.accumulate_forces_bind_group, &[]);
//     pass.dispatch_workgroups(MAX_SOLIDS as u32, 1, 1);
//     pass.pop_debug_group();
// }
