use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::uniform_buffer, AsBindGroup, BindGroup, BindGroupEntries,
            BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache, ShaderStages, ShaderType, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        storage::GpuShaderStorageBuffer,
        texture::{FallbackImage, GpuImage},
        Render, RenderApp, RenderSystems,
    },
};

use crate::{definition::FluidSettings, pipeline::Pipeline};

pub(crate) struct ReinitializeLevelsetPlugin;

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct ReinitLevelsetInitializeSeedsResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_air1: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = WriteOnly)]
    pub jump_flooding_seeds_x: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = WriteOnly)]
    pub jump_flooding_seeds_y: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct ReinitLevelsetIterateResource {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub jump_flooding_seeds_x: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub jump_flooding_seeds_y: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct ReinitLevelsetCalculateSdfResource {
    #[storage_texture(0, image_format = R32Float, access = WriteOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub levelset_air1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub jump_flooding_seeds_x: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadOnly)]
    pub jump_flooding_seeds_y: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, ShaderType)]
pub(crate) struct JumpFloodingUniform {
    pub step: u32,
}

#[derive(Resource)]
pub(crate) struct ReinitLevelsetPipeline {
    pub init_seeds_pipeline: CachedComputePipelineId,
    pub iterate_pipeline: CachedComputePipelineId,
    pub sdf_pipeline: CachedComputePipelineId,
    init_seeds_bind_group_layout: BindGroupLayout,
    iterate_bind_group_layout: BindGroupLayout,
    jump_flooding_step_bind_group_layout: BindGroupLayout,
    sdf_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub(crate) struct ReinitLevelsetBindGroups {
    pub init_seeds_bind_group: BindGroup,
    pub iterate_bind_group: BindGroup,
    pub jump_flooding_step_bind_groups: Box<[BindGroup]>,
    pub sdf_bind_group: BindGroup,
}

impl Plugin for ReinitializeLevelsetPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/recompute_levelset/initialize.wgsl");
        embedded_asset!(app, "shaders/recompute_levelset/iterate.wgsl");
        embedded_asset!(app, "shaders/recompute_levelset/calculate_sdf.wgsl");

        app.add_plugins((
            ExtractComponentPlugin::<ReinitLevelsetInitializeSeedsResource>::default(),
            ExtractComponentPlugin::<ReinitLevelsetIterateResource>::default(),
            ExtractComponentPlugin::<ReinitLevelsetCalculateSdfResource>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<ReinitLevelsetPipeline>();
    }
}

impl Pipeline for ReinitLevelsetPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.init_seeds_pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.iterate_pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.sdf_pipeline)
    }
}

impl FromWorld for ReinitLevelsetPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let jump_flooding_step_bind_group_layout = render_device.create_bind_group_layout(
            Some("JumpFloodingStepBindGroupLayout"),
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<JumpFloodingUniform>(false),
            ),
        );
        let init_seeds_bind_group_layout =
            ReinitLevelsetInitializeSeedsResource::bind_group_layout(render_device);
        let iterate_bind_group_layout =
            ReinitLevelsetIterateResource::bind_group_layout(render_device);
        let sdf_bind_group_layout =
            ReinitLevelsetCalculateSdfResource::bind_group_layout(render_device);

        let init_seeds_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("ReinitializeLevelset_InitializeSeeds".into()),
                layout: vec![init_seeds_bind_group_layout.clone()],
                shader: load_embedded_asset!(
                    asset_server,
                    "shaders/recompute_levelset/initialize.wgsl"
                ),
                entry_point: Some("initialize".into()),
                ..default()
            });

        let iterate_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("ReinitializeLevelset_Iterate".into()),
            layout: vec![
                iterate_bind_group_layout.clone(),
                jump_flooding_step_bind_group_layout.clone(),
            ],
            shader: load_embedded_asset!(asset_server, "shaders/recompute_levelset/iterate.wgsl"),
            entry_point: Some("iterate".into()),
            ..default()
        });

        let sdf_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("ReinitializeLevelset_Sdf".into()),
            layout: vec![sdf_bind_group_layout.clone()],
            shader: load_embedded_asset!(
                asset_server,
                "shaders/recompute_levelset/calculate_sdf.wgsl"
            ),
            entry_point: Some("calculate_sdf".into()),
            ..default()
        });

        ReinitLevelsetPipeline {
            init_seeds_pipeline,
            iterate_pipeline,
            sdf_pipeline,
            init_seeds_bind_group_layout,
            iterate_bind_group_layout,
            jump_flooding_step_bind_group_layout,
            sdf_bind_group_layout,
        }
    }
}

fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<ReinitLevelsetPipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    query: Query<(
        Entity,
        &FluidSettings,
        &ReinitLevelsetInitializeSeedsResource,
        &ReinitLevelsetIterateResource,
        &ReinitLevelsetCalculateSdfResource,
    )>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, settings, init_seeds_resource, iterate_resource, sdf_resource) in &query {
        // steps for jump flooding algorithm: 1, 2, ..., 2^k, where: 2^k < max(size.0, size.1) <= 2^(k+1)
        let max_power = ((settings.size.max_element() as f32).log2() - 1.0).floor() as usize;
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

        let mut jump_flooding_step_bind_groups = Vec::with_capacity(jump_flooding_buffer.len());
        for buffer in &jump_flooding_buffer {
            jump_flooding_step_bind_groups.push(render_device.create_bind_group(
                Some("JumpFloodingStepBindGroup"),
                &pipeline.jump_flooding_step_bind_group_layout,
                &BindGroupEntries::single(buffer.binding().unwrap()),
            ));
        }

        let init_seeds_bind_group = init_seeds_resource
            .as_bind_group(
                &pipeline.init_seeds_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let iterate_bind_group = iterate_resource
            .as_bind_group(
                &pipeline.iterate_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;
        let sdf_bind_group = sdf_resource
            .as_bind_group(&pipeline.sdf_bind_group_layout, &render_device, &mut param)
            .unwrap()
            .bind_group;

        commands.entity(entity).insert(ReinitLevelsetBindGroups {
            init_seeds_bind_group,
            iterate_bind_group,
            jump_flooding_step_bind_groups: jump_flooding_step_bind_groups.into_boxed_slice(),
            sdf_bind_group,
        });
    }
}
