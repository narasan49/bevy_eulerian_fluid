use std::borrow::Cow;

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

use crate::definition::{ForcesToSolid, SolidForcesBins};
use crate::euler_fluid::{ACCUMULATE_FORCES_SHADER_HANDLE, SAMPLE_FORCES_SHADER_HANDLE};

use super::definition::{
    DivergenceTextures, FluidSettings, JumpFloodingSeedsTextures, JumpFloodingUniform,
    JumpFloodingUniformBuffer, LevelsetTextures, LocalForces, Obstacles, PressureTextures,
    SimulationUniform, SolidVelocityTextures, VelocityTextures, VelocityTexturesIntermediate,
    VelocityTexturesU, VelocityTexturesV,
};

pub(super) const INITIALIZE_GRID_CENTER_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0xD9C0123A6DC94D01AA0D8BEF9784EC16);
pub(super) const INITIALIZE_VELOCITY_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0xE517B3F694A9446B970368B971BF631E);

pub(super) const UPDATE_SOLID_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x3B7E226FADA549C1A6662BCED3B83535);
pub(super) const UPDATE_SOLID_PRESSURE_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0xD9A3A8F98001448A946FD0AE976F9B96);
pub(super) const ADVECT_VELOCITY_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x4C394851214E47D3879CA7E1837A2D07);
pub(super) const APPLY_FORCE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x446049599EE84040B9FFE7C00783F856);
pub(super) const DIVERGENCE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0xD31C2EF5DE254DC097F20C813A5A0C6D);
pub(super) const JACOBI_ITERATION_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x8BB3FAA20BC24FB4B790C11A8A2F8E63);
pub(super) const SOLVE_VELOCITY_U_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x1B95362358B242BCA68804444013F99E);
pub(super) const SOLVE_VELOCITY_V_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0xbfae85ad7e30440aad02c9ad2870ea51);
pub(super) const EXTRAPOLATE_VELOCITY_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x90978F4EBD3942E78B116D156801D737);

pub(super) const RECOMPUTE_LEVELSET_INITIALIZE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0xAFC6EC29854A413CB0E4113506AE2254);
pub(super) const RECOMPUTE_LEVELSET_ITERATE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x5BD03953327243328E2D0F3373934E39);
pub(super) const RECOMPUTE_LEVELSET_SDF_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x45710E52048449A5A59D382284974B38);
pub(super) const ADVECT_LEVELSET_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x4165F4894F76420E8D67FC83E3466ACA);

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
}

impl FromWorld for FluidPipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();

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
        let local_forces_bind_group_layout = LocalForces::bind_group_layout(render_device);
        let pressure_bind_group_layout = PressureTextures::bind_group_layout(render_device);
        let divergence_bind_group_layout = DivergenceTextures::bind_group_layout(render_device);
        let levelset_bind_group_layout = LevelsetTextures::bind_group_layout(render_device);
        let obstacles_bind_group_layout = Obstacles::bind_group_layout(render_device);
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

        let initialize_velocity_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue InitializeVelocityPipeline")),
                layout: vec![velocity_bind_group_layout.clone()],
                push_constant_ranges: vec![],
                shader: INITIALIZE_VELOCITY_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("initialize_velocity"),
                zero_initialize_workgroup_memory: false,
            });

        let initialize_grid_center_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue InitializeGridCenterPipeline")),
                layout: vec![
                    levelset_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: INITIALIZE_GRID_CENTER_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("initialize_grid_center"),
                zero_initialize_workgroup_memory: false,
            });

        let update_solid_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue UpdateSolidPipeline")),
                layout: vec![
                    solid_velocity_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                    obstacles_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: UPDATE_SOLID_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("update_solid"),
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
                shader: UPDATE_SOLID_PRESSURE_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("update_solid_pressure"),
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
            shader: ADVECT_VELOCITY_SHADER_HANDLE,
            shader_defs: vec![],
            entry_point: Cow::from("advect_u"),
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
            shader: ADVECT_VELOCITY_SHADER_HANDLE,
            shader_defs: vec![],
            entry_point: Cow::from("advect_v"),
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
                shader: APPLY_FORCE_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("apply_force_u"),
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
                shader: APPLY_FORCE_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("apply_force_v"),
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
                shader: DIVERGENCE_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("divergence"),
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
                shader: JACOBI_ITERATION_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("jacobi_iteration"),
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
                shader: JACOBI_ITERATION_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("jacobi_iteration_reverse"),
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
                shader: SOLVE_VELOCITY_U_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("solve_velocity_u"),
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
                shader: SOLVE_VELOCITY_V_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("solve_velocity_v"),
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
                shader: EXTRAPOLATE_VELOCITY_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("extrapolate_u"),
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
                shader: EXTRAPOLATE_VELOCITY_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("extrapolate_v"),
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
                shader: RECOMPUTE_LEVELSET_INITIALIZE_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("initialize"),
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
                shader: RECOMPUTE_LEVELSET_ITERATE_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("iterate"),
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
                shader: RECOMPUTE_LEVELSET_SDF_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("calculate_sdf"),
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
                shader: ADVECT_LEVELSET_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("advect_levelset"),
                zero_initialize_workgroup_memory: false,
            });

        let sample_forces_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue SampleForcesPipeline")),
                layout: vec![
                    solid_forces_bins_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                    pressure_bind_group_layout.clone(),
                    solid_velocity_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: SAMPLE_FORCES_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("sample_forces_to_solid"),
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
                shader: ACCUMULATE_FORCES_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::from("accumulate_forces"),
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
    pub pressure_bind_group: BindGroup,
    pub divergence_bind_group: BindGroup,
    pub local_forces_bind_group: BindGroup,
    pub levelset_bind_group: BindGroup,
    pub jump_flooding_seeds_bind_group: BindGroup,
    pub solid_forces_bins_bind_group: BindGroup,
    pub forces_to_solid_bind_group: BindGroup,
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

pub(super) fn prepare_fluid_bind_groups(
    mut commands: Commands,
    pipelines: Res<FluidPipelines>,
    simulation_uniform: Res<ComponentUniforms<SimulationUniform>>,
    query: Query<(
        Entity,
        &VelocityTextures,
        &VelocityTexturesU,
        &VelocityTexturesV,
        &VelocityTexturesIntermediate,
        &SolidVelocityTextures,
        &PressureTextures,
        &DivergenceTextures,
        &LevelsetTextures,
        &LocalForces,
        &DynamicUniformIndex<SimulationUniform>,
        &JumpFloodingSeedsTextures,
        &JumpFloodingUniformBuffer,
        &SolidForcesBins,
        &ForcesToSolid,
    )>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let mut param = (gpu_images, fallback_image, buffers);
    for (
        entity,
        velocity_textures,
        velocity_textures_u,
        velocity_textures_v,
        velocity_textures_intermediate,
        solid_velocity_textures,
        pressure_textures,
        divergence_textures,
        levelset_textures,
        local_forces,
        simulation_uniform_index,
        jump_flooding_seeds_textures,
        jump_flooding_uniform_buffer,
        solid_forces_bins,
        forces_to_solid,
    ) in &query
    {
        let simulation_uniform = simulation_uniform.uniforms();
        let uniform_bind_group = render_device.create_bind_group(
            "Simulation Uniform BindGroup",
            &pipelines.uniform_bind_group_layout,
            &BindGroupEntries::single(simulation_uniform),
        );

        let velocity_bind_group = velocity_textures
            .as_bind_group(
                &pipelines.velocity_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let velocity_u_bind_group = velocity_textures_u
            .as_bind_group(
                &pipelines.velocity_u_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let velocity_v_bind_group = velocity_textures_v
            .as_bind_group(
                &pipelines.velocity_v_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let velocity_intermediate_bind_group = velocity_textures_intermediate
            .as_bind_group(
                &pipelines.velocity_intermediate_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let solid_velocity_bind_group = solid_velocity_textures
            .as_bind_group(
                &pipelines.solid_velocity_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let pressure_bind_group = pressure_textures
            .as_bind_group(
                &pipelines.pressure_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let divergence_bind_group = divergence_textures
            .as_bind_group(
                &pipelines.divergence_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let local_forces_bind_group = local_forces
            .as_bind_group(
                &pipelines.local_forces_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let mut jump_flooding_step_bind_groups =
            Vec::with_capacity(jump_flooding_uniform_buffer.buffer.len());
        for buffer in &jump_flooding_uniform_buffer.buffer {
            jump_flooding_step_bind_groups.push(render_device.create_bind_group(
                Some("Create JumpFloodingStepBindGroup"),
                &pipelines.jump_flooding_uniform_bind_group_layout,
                &BindGroupEntries::single(buffer.binding().unwrap()),
            ));
        }

        let levelset_bind_group = levelset_textures
            .as_bind_group(
                &pipelines.levelset_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let jump_flooding_seeds_bind_group = jump_flooding_seeds_textures
            .as_bind_group(
                &pipelines.jump_flooding_seeds_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let solid_forces_bins_bind_group = solid_forces_bins
            .as_bind_group(
                &pipelines.solid_forces_bins_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;
        let forces_to_solid_bind_group = forces_to_solid
            .as_bind_group(
                &pipelines.forces_to_solid_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands.entity(entity).insert((
            FluidBindGroups {
                velocity_bind_group,
                velocity_u_bind_group,
                velocity_v_bind_group,
                velocity_intermediate_bind_group,
                solid_velocity_bind_group,
                pressure_bind_group,
                divergence_bind_group,
                local_forces_bind_group,
                levelset_bind_group,
                jump_flooding_seeds_bind_group,
                solid_forces_bins_bind_group,
                forces_to_solid_bind_group,
                uniform_bind_group,
                uniform_index: simulation_uniform_index.index(),
            },
            JumpFloodingUniformBindGroups {
                jump_flooding_step_bind_groups: jump_flooding_step_bind_groups.into_boxed_slice(),
            },
        ));
    }
}

pub(super) fn prepare_fluid_bind_group_for_resources(
    mut commands: Commands,
    pilelines: Res<FluidPipelines>,
    obstacles: Res<Obstacles>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let mut param = (gpu_images, fallback_image, buffers);
    let obstacles_bind_group = obstacles
        .as_bind_group(
            &pilelines.obstacles_bind_group_layout,
            &render_device,
            &mut param,
        )
        .unwrap()
        .bind_group;
    commands.insert_resource(FluidBindGroupResources {
        obstacles_bind_group,
    });
}
