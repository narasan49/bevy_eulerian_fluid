use std::borrow::Cow;

use bevy::asset::{embedded_asset, load_embedded_asset};
use bevy::ecs::query::QueryData;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::UniformBuffer;
use bevy::render::renderer::RenderQueue;
use bevy::render::storage::GpuShaderStorageBuffer;
use bevy::{
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, DynamicUniformIndex},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::uniform_buffer, AsBindGroup, BindGroup, BindGroupEntries,
            BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache, ShaderStages,
        },
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
    },
};

use crate::definition::{
    ForcesToSolid, SampleForcesResource, SolidCenterTextures, SolidForcesBins, SolidObstaclesBuffer,
};

use super::definition::{
    DivergenceTextures, FluidSettings, JumpFloodingSeedsTextures, JumpFloodingUniform,
    JumpFloodingUniformBuffer, LevelsetTextures, LocalForces, PressureTextures, SimulationUniform,
    SolidVelocityTextures, VelocityTextures, VelocityTexturesIntermediate, VelocityTexturesU,
    VelocityTexturesV,
};

pub(crate) struct FluidShaderResourcePlugin;

impl Plugin for FluidShaderResourcePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/initialize_velocity.wgsl");
        embedded_asset!(app, "shaders/initialize_grid_center.wgsl");
        embedded_asset!(app, "shaders/update_solid.wgsl");
        embedded_asset!(app, "shaders/update_solid_pressure.wgsl");
        embedded_asset!(app, "shaders/advect_velocity.wgsl");
        embedded_asset!(app, "shaders/apply_force.wgsl");
        embedded_asset!(app, "shaders/divergence.wgsl");
        embedded_asset!(app, "shaders/jacobi_iteration.wgsl");
        embedded_asset!(app, "shaders/solve_velocity_u.wgsl");
        embedded_asset!(app, "shaders/solve_velocity_v.wgsl");
        embedded_asset!(app, "shaders/extrapolate_velocity.wgsl");
        embedded_asset!(app, "shaders/recompute_levelset/initialize.wgsl");
        embedded_asset!(app, "shaders/recompute_levelset/iterate.wgsl");
        embedded_asset!(app, "shaders/recompute_levelset/calculate_sdf.wgsl");
        embedded_asset!(app, "shaders/advect_levelset.wgsl");
        embedded_asset!(app, "shaders/fluid_to_solid/sample_forces.wgsl");
        embedded_asset!(app, "shaders/fluid_to_solid/accumulate_forces.wgsl");
    }
}

#[derive(Resource)]
pub(crate) struct FluidPipelines {
    pub initialize_velocity_pipeline: CachedComputePipelineId,
    pub initialize_grid_center_pipeline: CachedComputePipelineId,
    pub update_solid_pipeline: CachedComputePipelineId,
    pub update_solid_pressure_pipeline: CachedComputePipelineId,
    pub advect_u_pipeline: CachedComputePipelineId,
    pub advect_v_pipeline: CachedComputePipelineId,
    pub apply_force_u_pipeline: CachedComputePipelineId,
    pub apply_force_v_pipeline: CachedComputePipelineId,
    pub divergence_pipeline: CachedComputePipelineId,
    pub jacobi_iteration_pipeline: CachedComputePipelineId,
    pub jacobi_iteration_reverse_pipeline: CachedComputePipelineId,
    pub solve_velocity_u_pipeline: CachedComputePipelineId,
    pub solve_velocity_v_pipeline: CachedComputePipelineId,
    pub extrapolate_u_pipeline: CachedComputePipelineId,
    pub extrapolate_v_pipeline: CachedComputePipelineId,
    pub recompute_levelset_initialization_pipeline: CachedComputePipelineId,
    pub recompute_levelset_iteration_pipeline: CachedComputePipelineId,
    pub recompute_levelset_solve_pipeline: CachedComputePipelineId,
    pub advect_levelset_pipeline: CachedComputePipelineId,
    pub sample_forces_pipeline: CachedComputePipelineId,
    pub accumulate_forces_pipeline: CachedComputePipelineId,
    velocity_bind_group_layout: BindGroupLayout,
    velocity_u_bind_group_layout: BindGroupLayout,
    velocity_v_bind_group_layout: BindGroupLayout,
    velocity_intermediate_bind_group_layout: BindGroupLayout,
    solid_velocity_bind_group_layout: BindGroupLayout,
    pressure_bind_group_layout: BindGroupLayout,
    divergence_bind_group_layout: BindGroupLayout,
    levelset_bind_group_layout: BindGroupLayout,
    local_forces_bind_group_layout: BindGroupLayout,
    uniform_bind_group_layout: BindGroupLayout,
    obstacles_bind_group_layout: BindGroupLayout,
    jump_flooding_seeds_bind_group_layout: BindGroupLayout,
    jump_flooding_uniform_bind_group_layout: BindGroupLayout,
    solid_forces_bins_bind_group_layout: BindGroupLayout,
    forces_to_solid_bind_group_layout: BindGroupLayout,
    solid_center_textures_bind_group_layout: BindGroupLayout,
    sample_forces_bind_group_layout: BindGroupLayout,
}

impl FromWorld for FluidPipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();
        info!("Creating FluidPipelines...");

        let uniform_bind_group_layout = render_device.create_bind_group_layout(
            Some("Create uniform bind group layout"),
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<SimulationUniform>(true),
            ),
        );
        let velocity_bind_group_layout = VelocityTextures::bind_group_layout(render_device);
        let velocity_u_bind_group_layout = VelocityTexturesU::bind_group_layout(render_device);
        let velocity_v_bind_group_layout = VelocityTexturesV::bind_group_layout(render_device);
        let velocity_intermediate_bind_group_layout =
            VelocityTexturesIntermediate::bind_group_layout(render_device);
        let solid_velocity_bind_group_layout =
            SolidVelocityTextures::bind_group_layout(render_device);
        let solid_center_textures_bind_group_layout =
            SolidCenterTextures::bind_group_layout(render_device);
        let local_forces_bind_group_layout = LocalForces::bind_group_layout(render_device);
        let pressure_bind_group_layout = PressureTextures::bind_group_layout(render_device);
        let divergence_bind_group_layout = DivergenceTextures::bind_group_layout(render_device);
        let levelset_bind_group_layout = LevelsetTextures::bind_group_layout(render_device);
        let obstacles_bind_group_layout = SolidObstaclesBuffer::bind_group_layout(render_device);
        let jump_flooding_seeds_bind_group_layout =
            JumpFloodingSeedsTextures::bind_group_layout(render_device);
        let jump_flooding_uniform_bind_group_layout = render_device.create_bind_group_layout(
            Some("Create JumpFloodingUniformBindGroupLayout"),
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<JumpFloodingUniform>(false),
            ),
        );
        let solid_forces_bins_bind_group_layout = SolidForcesBins::bind_group_layout(render_device);
        let forces_to_solid_bind_group_layout = ForcesToSolid::bind_group_layout(render_device);
        let sample_forces_bind_group_layout =
            SampleForcesResource::bind_group_layout(render_device);

        let initialize_velocity_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue InitializeVelocityPipeline")),
                layout: vec![velocity_bind_group_layout.clone()],
                shader: load_embedded_asset!(asset_server, "shaders/initialize_velocity.wgsl"),
                entry_point: Some(("initialize_velocity").into()),
                ..default()
            });

        let initialize_grid_center_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(("Queue InitializeGridCenterPipeline").into()),
                layout: vec![
                    levelset_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                shader: load_embedded_asset!(asset_server, "shaders/initialize_grid_center.wgsl"),
                entry_point: Some(("initialize_grid_center").into()),
                ..default()
            });

        let update_solid_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue UpdateSolidPipeline")),
                layout: vec![
                    solid_velocity_bind_group_layout.clone(),
                    solid_center_textures_bind_group_layout.clone(),
                    obstacles_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(asset_server, "shaders/update_solid.wgsl"),
                shader_defs: vec![],
                entry_point: Some(("update_solid").into()),
                zero_initialize_workgroup_memory: false,
            });

        let update_solid_pressure_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue UpdateSolidPressurePipeline")),
                layout: vec![
                    pressure_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(asset_server, "shaders/update_solid_pressure.wgsl"),
                shader_defs: vec![],
                entry_point: Some(("update_solid_pressure").into()),
                zero_initialize_workgroup_memory: false,
            });

        let advect_u_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("Queue AdvectionPipeline")),
            layout: vec![
                velocity_bind_group_layout.clone(),
                levelset_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: load_embedded_asset!(asset_server, "shaders/advect_velocity.wgsl"),
            shader_defs: vec![],
            entry_point: Some(("advect_u").into()),
            zero_initialize_workgroup_memory: false,
        });

        let advect_v_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("Queue AdvectionPipeline")),
            layout: vec![
                velocity_bind_group_layout.clone(),
                levelset_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: load_embedded_asset!(asset_server, "shaders/advect_velocity.wgsl"),
            shader_defs: vec![],
            entry_point: Some(("advect_v").into()),
            zero_initialize_workgroup_memory: false,
        });

        let apply_force_u_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue AddForcePipeline")),
                layout: vec![
                    velocity_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                    local_forces_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(asset_server, "shaders/apply_force.wgsl"),
                shader_defs: vec![],
                entry_point: Some(("apply_force_u").into()),
                zero_initialize_workgroup_memory: false,
            });

        let apply_force_v_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue AddForcePipeline")),
                layout: vec![
                    velocity_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                    local_forces_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(asset_server, "shaders/apply_force.wgsl"),
                shader_defs: vec![],
                entry_point: Some(("apply_force_v").into()),
                zero_initialize_workgroup_memory: false,
            });

        let divergence_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue DivergencePipeline")),
                layout: vec![
                    velocity_intermediate_bind_group_layout.clone(),
                    divergence_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                    solid_velocity_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(asset_server, "shaders/divergence.wgsl"),
                shader_defs: vec![],
                entry_point: Some(("divergence").into()),
                zero_initialize_workgroup_memory: false,
            });

        let jacobi_iteration_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue JacobiIterationPipeline")),
                layout: vec![
                    uniform_bind_group_layout.clone(),
                    pressure_bind_group_layout.clone(),
                    divergence_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(asset_server, "shaders/jacobi_iteration.wgsl"),
                shader_defs: vec![],
                entry_point: Some(("jacobi_iteration").into()),
                zero_initialize_workgroup_memory: false,
            });

        let jacobi_iteration_reverse_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue JacobiIterationReversePipeline")),
                layout: vec![
                    uniform_bind_group_layout.clone(),
                    pressure_bind_group_layout.clone(),
                    divergence_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(asset_server, "shaders/jacobi_iteration.wgsl"),
                shader_defs: vec![],
                entry_point: Some(("jacobi_iteration_reverse").into()),
                zero_initialize_workgroup_memory: false,
            });

        let solve_velocity_u_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue SolveVelocityUPipeline")),
                layout: vec![
                    velocity_v_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                    pressure_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(asset_server, "shaders/solve_velocity_u.wgsl"),
                shader_defs: vec![],
                entry_point: Some(("solve_velocity_u").into()),
                zero_initialize_workgroup_memory: false,
            });

        let solve_velocity_v_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue SolveVelocityVPipeline")),
                layout: vec![
                    velocity_v_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                    pressure_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(asset_server, "shaders/solve_velocity_v.wgsl"),
                shader_defs: vec![],
                entry_point: Some(("solve_velocity_v").into()),
                zero_initialize_workgroup_memory: false,
            });

        let extrapolate_u_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue ExtrapolateUPipeline")),
                layout: vec![
                    velocity_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(asset_server, "shaders/extrapolate_velocity.wgsl"),
                shader_defs: vec![],
                entry_point: Some(("extrapolate_u").into()),
                zero_initialize_workgroup_memory: false,
            });

        let extrapolate_v_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue ExtrapolateVPipeline")),
                layout: vec![
                    velocity_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(asset_server, "shaders/extrapolate_velocity.wgsl"),
                shader_defs: vec![],
                entry_point: Some(("extrapolate_v").into()),
                zero_initialize_workgroup_memory: false,
            });

        let recompute_levelset_initialization_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue RecomputeLevelsetInitializationPipeline")),
                layout: vec![
                    levelset_bind_group_layout.clone(),
                    jump_flooding_seeds_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(
                    asset_server,
                    "shaders/recompute_levelset/initialize.wgsl"
                ),
                shader_defs: vec![],
                entry_point: Some(("initialize").into()),
                zero_initialize_workgroup_memory: false,
            });

        let recompute_levelset_iteration_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue RecomputeLevelsetIteratePipeline")),
                layout: vec![
                    jump_flooding_seeds_bind_group_layout.clone(),
                    jump_flooding_uniform_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(
                    asset_server,
                    "shaders/recompute_levelset/iterate.wgsl"
                ),
                shader_defs: vec![],
                entry_point: Some(("iterate").into()),
                zero_initialize_workgroup_memory: false,
            });

        let recompute_levelset_solve_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue RecomputeLevelsetSolvePipeline")),
                layout: vec![
                    levelset_bind_group_layout.clone(),
                    jump_flooding_seeds_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(
                    asset_server,
                    "shaders/recompute_levelset/calculate_sdf.wgsl"
                ),
                shader_defs: vec![],
                entry_point: Some(("calculate_sdf").into()),
                zero_initialize_workgroup_memory: false,
            });

        let advect_levelset_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue AdvectLevelsetPipeline")),
                layout: vec![
                    velocity_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(asset_server, "shaders/advect_levelset.wgsl"),
                shader_defs: vec![],
                entry_point: Some(("advect_levelset").into()),
                zero_initialize_workgroup_memory: false,
            });

        let sample_forces_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue SampleForcesPipeline")),
                layout: vec![
                    sample_forces_bind_group_layout.clone(),
                    obstacles_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(
                    asset_server,
                    "shaders/fluid_to_solid/sample_forces.wgsl"
                ),
                shader_defs: vec![],
                entry_point: Some(("sample_forces_to_solid").into()),
                zero_initialize_workgroup_memory: false,
            });

        let accumulate_forces_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue AccumulateForcesPipeline")),
                layout: vec![
                    solid_forces_bins_bind_group_layout.clone(),
                    forces_to_solid_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: load_embedded_asset!(
                    asset_server,
                    "shaders/fluid_to_solid/accumulate_forces.wgsl"
                ),
                shader_defs: vec![],
                entry_point: Some(("accumulate_forces").into()),
                zero_initialize_workgroup_memory: false,
            });

        Self {
            initialize_velocity_pipeline,
            initialize_grid_center_pipeline,
            update_solid_pipeline,
            update_solid_pressure_pipeline,
            advect_u_pipeline,
            advect_v_pipeline,
            apply_force_u_pipeline,
            apply_force_v_pipeline,
            divergence_pipeline,
            jacobi_iteration_pipeline,
            jacobi_iteration_reverse_pipeline,
            solve_velocity_u_pipeline,
            solve_velocity_v_pipeline,
            extrapolate_u_pipeline,
            extrapolate_v_pipeline,
            recompute_levelset_initialization_pipeline,
            recompute_levelset_iteration_pipeline,
            recompute_levelset_solve_pipeline,
            advect_levelset_pipeline,
            accumulate_forces_pipeline,
            sample_forces_pipeline,
            velocity_bind_group_layout,
            velocity_u_bind_group_layout,
            velocity_v_bind_group_layout,
            velocity_intermediate_bind_group_layout,
            solid_velocity_bind_group_layout,
            pressure_bind_group_layout,
            divergence_bind_group_layout,
            levelset_bind_group_layout,
            local_forces_bind_group_layout,
            uniform_bind_group_layout,
            obstacles_bind_group_layout,
            jump_flooding_uniform_bind_group_layout,
            jump_flooding_seeds_bind_group_layout,
            solid_forces_bins_bind_group_layout,
            forces_to_solid_bind_group_layout,
            solid_center_textures_bind_group_layout,
            sample_forces_bind_group_layout,
        }
    }
}

#[derive(Component, Clone, ExtractComponent)]
pub(crate) struct FluidBindGroups {
    pub velocity_bind_group: BindGroup,
    pub velocity_u_bind_group: BindGroup,
    pub velocity_v_bind_group: BindGroup,
    pub velocity_intermediate_bind_group: BindGroup,
    pub solid_velocity_bind_group: BindGroup,
    pub solid_center_bind_group: BindGroup,
    pub pressure_bind_group: BindGroup,
    pub divergence_bind_group: BindGroup,
    pub local_forces_bind_group: BindGroup,
    pub levelset_bind_group: BindGroup,
    pub jump_flooding_seeds_bind_group: BindGroup,
    pub solid_forces_bins_bind_group: BindGroup,
    pub forces_to_solid_bind_group: BindGroup,
    pub sample_forces_bind_group: BindGroup,
    pub uniform_bind_group: BindGroup,
    pub uniform_index: u32,
}

#[derive(Resource)]
pub(crate) struct FluidBindGroupResources {
    pub obstacles_bind_group: BindGroup,
}

/// Different from [`FluidBindGroups`], [`DynamicUniformIndex`] will not be used.
/// Here, several bindings for jump flooding steps for each component.
/// However, only one index can be used per component on [`DynamicUniformIndex`].
/// Therefore, array of bind groups per component is adopted here.
#[derive(Component)]
pub(crate) struct JumpFloodingUniformBindGroups {
    pub jump_flooding_step_bind_groups: Box<[BindGroup]>,
    // pub uniform_index: u32,
}

pub(super) fn prepare_resource_recompute_levelset(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    query: Query<(Entity, &FluidSettings)>,
) {
    for (entity, settings) in &query {
        // steps for jump flooding algorithm: 1, 2, ..., 2^k, where: 2^k < max(size.0, size.1) <= 2^(k+1)
        let max_power =
            ((settings.size.0.max(settings.size.1) as f32).log2() - 1.0).floor() as usize;
        let mut step = 2_u32.pow((max_power + 1) as u32);
        let mut jump_flooding_buffer =
            Vec::<UniformBuffer<JumpFloodingUniform>>::with_capacity(max_power + 1);
        for _ in 0..max_power + 1 {
            step /= 2;
            jump_flooding_buffer.push(UniformBuffer::from(JumpFloodingUniform { step }));
        }
        for buffer in &mut jump_flooding_buffer {
            buffer.write_buffer(&render_device, &render_queue);
        }

        commands.entity(entity).insert(JumpFloodingUniformBuffer {
            buffer: jump_flooding_buffer,
        });
    }
}

#[derive(QueryData)]
pub(super) struct TextureQuery {
    entity: Entity,
    velocity_textures: Ref<'static, VelocityTextures>,
    velocity_textures_u: Ref<'static, VelocityTexturesU>,
    velocity_textures_v: Ref<'static, VelocityTexturesV>,
    velocity_textures_intermediate: Ref<'static, VelocityTexturesIntermediate>,
    solid_velocity_textures: Ref<'static, SolidVelocityTextures>,
    solid_center_textures: Ref<'static, SolidCenterTextures>,
    pressure_textures: Ref<'static, PressureTextures>,
    divergence_textures: Ref<'static, DivergenceTextures>,
    levelset_textures: Ref<'static, LevelsetTextures>,
    local_forces: Ref<'static, LocalForces>,
    simulation_uniform_index: Ref<'static, DynamicUniformIndex<SimulationUniform>>,
    jump_flooding_seeds_textures: Ref<'static, JumpFloodingSeedsTextures>,
    jump_flooding_uniform_buffer: Ref<'static, JumpFloodingUniformBuffer>,
    solid_forces_bins: Ref<'static, SolidForcesBins>,
    sample_forces: Ref<'static, SampleForcesResource>,
    forces_to_solid: Ref<'static, ForcesToSolid>,
}

pub(super) fn prepare_fluid_bind_groups(
    mut commands: Commands,
    pipelines: Res<FluidPipelines>,
    simulation_uniform: Res<ComponentUniforms<SimulationUniform>>,
    query: Query<TextureQuery>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let mut param = (gpu_images, fallback_image, buffers);
    for t in &query {
        let simulation_uniform = simulation_uniform.uniforms();
        let uniform_bind_group = render_device.create_bind_group(
            "Simulation Uniform BindGroup",
            &pipelines.uniform_bind_group_layout,
            &BindGroupEntries::single(simulation_uniform),
        );

        let velocity_bind_group = t
            .velocity_textures
            .as_bind_group(
                &pipelines.velocity_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let velocity_u_bind_group = t
            .velocity_textures_u
            .as_bind_group(
                &pipelines.velocity_u_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let velocity_v_bind_group = t
            .velocity_textures_v
            .as_bind_group(
                &pipelines.velocity_v_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let velocity_intermediate_bind_group = t
            .velocity_textures_intermediate
            .as_bind_group(
                &pipelines.velocity_intermediate_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let solid_velocity_bind_group = t
            .solid_velocity_textures
            .as_bind_group(
                &pipelines.solid_velocity_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let solid_center_bind_group = t
            .solid_center_textures
            .as_bind_group(
                &pipelines.solid_center_textures_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let pressure_bind_group = t
            .pressure_textures
            .as_bind_group(
                &pipelines.pressure_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let divergence_bind_group = t
            .divergence_textures
            .as_bind_group(
                &pipelines.divergence_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let local_forces_bind_group = t
            .local_forces
            .as_bind_group(
                &pipelines.local_forces_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let mut jump_flooding_step_bind_groups =
            Vec::with_capacity(t.jump_flooding_uniform_buffer.buffer.len());
        for buffer in &t.jump_flooding_uniform_buffer.buffer {
            jump_flooding_step_bind_groups.push(render_device.create_bind_group(
                Some("Create JumpFloodingStepBindGroup"),
                &pipelines.jump_flooding_uniform_bind_group_layout,
                &BindGroupEntries::single(buffer.binding().unwrap()),
            ));
        }

        let levelset_bind_group = t
            .levelset_textures
            .as_bind_group(
                &pipelines.levelset_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let jump_flooding_seeds_bind_group = t
            .jump_flooding_seeds_textures
            .as_bind_group(
                &pipelines.jump_flooding_seeds_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let solid_forces_bins_bind_group = t
            .solid_forces_bins
            .as_bind_group(
                &pipelines.solid_forces_bins_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let sample_forces_bind_group = t
            .sample_forces
            .as_bind_group(
                &pipelines.sample_forces_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let forces_to_solid_bind_group = t
            .forces_to_solid
            .as_bind_group(
                &pipelines.forces_to_solid_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands.entity(t.entity).insert((
            FluidBindGroups {
                velocity_bind_group,
                velocity_u_bind_group,
                velocity_v_bind_group,
                velocity_intermediate_bind_group,
                solid_velocity_bind_group,
                solid_center_bind_group,
                pressure_bind_group,
                divergence_bind_group,
                local_forces_bind_group,
                levelset_bind_group,
                jump_flooding_seeds_bind_group,
                solid_forces_bins_bind_group,
                sample_forces_bind_group,
                forces_to_solid_bind_group,
                uniform_bind_group,
                uniform_index: t.simulation_uniform_index.index(),
            },
            JumpFloodingUniformBindGroups {
                jump_flooding_step_bind_groups: jump_flooding_step_bind_groups.into_boxed_slice(),
            },
        ));
    }
}

pub(super) fn prepare_fluid_bind_group_for_resources(
    mut commands: Commands,
    pipelines: Res<FluidPipelines>,
    obstacles: Res<SolidObstaclesBuffer>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let mut param = (gpu_images, fallback_image, buffers);
    let obstacles_bind_group = obstacles
        .as_bind_group(
            &pipelines.obstacles_bind_group_layout,
            &render_device,
            &mut param,
        )
        .unwrap()
        .bind_group;
    commands.insert_resource(FluidBindGroupResources {
        obstacles_bind_group,
    });
}
