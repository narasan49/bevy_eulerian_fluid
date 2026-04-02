use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{texture_storage_2d, uniform_buffer},
            AsBindGroup, BindGroup, BindGroupEntries, BindGroupLayoutDescriptor,
            BindGroupLayoutEntries, CachedComputePipelineId, ComputePipelineDescriptor,
            PipelineCache, ShaderStages, ShaderType, StorageTextureAccess, TextureFormat,
            UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        storage::GpuShaderStorageBuffer,
        texture::{FallbackImage, GpuImage},
        Render, RenderApp, RenderSystems,
    },
};

use crate::{pipeline::Pipeline, settings::FluidSettings};

pub(crate) struct JumpFloodingPlugin;

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct JumpFloodingInitializeSeedsResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub levelset_air1: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct JumpFloodingCalculateSdfResource {
    #[storage_texture(0, image_format = R32Float, access = WriteOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub levelset_air1: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent)]
pub(crate) struct JumpFloodingSeedsTextures(pub [Handle<Image>; 2]);

#[derive(Component, Clone, ExtractComponent, ShaderType)]
pub(crate) struct JumpFloodingUniform {
    pub step: u32,
}

#[derive(Resource)]
pub(crate) struct JumpFloodingPipeline {
    pub init_seeds_pipeline: CachedComputePipelineId,
    pub iterate_pipeline: CachedComputePipelineId,
    pub sdf_pipeline: CachedComputePipelineId,
    init_seeds_bind_group_layout: BindGroupLayoutDescriptor,
    jump_flooding_step_bind_group_layout: BindGroupLayoutDescriptor,
    sdf_bind_group_layout: BindGroupLayoutDescriptor,
    read_only_seeds_bind_group_layout: BindGroupLayoutDescriptor,
    write_only_seeds_bind_group_layout: BindGroupLayoutDescriptor,
}

#[derive(Component)]
pub(crate) struct JumpFloodingBindGroups {
    pub init_seeds_bind_group: BindGroup,
    pub jump_flooding_step_bind_groups: Box<[BindGroup]>,
    pub sdf_bind_group: BindGroup,
    pub read_only_seeds_bind_groups: Box<[BindGroup]>,
    pub write_only_seeds_bind_groups: Box<[BindGroup]>,
}

impl Plugin for JumpFloodingPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/jump_flooding/initialize.wgsl");
        embedded_asset!(app, "shaders/jump_flooding/iterate.wgsl");
        embedded_asset!(app, "shaders/jump_flooding/calculate_sdf.wgsl");

        app.add_plugins((
            ExtractComponentPlugin::<JumpFloodingInitializeSeedsResource>::default(),
            ExtractComponentPlugin::<JumpFloodingSeedsTextures>::default(),
            ExtractComponentPlugin::<JumpFloodingCalculateSdfResource>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<JumpFloodingPipeline>();
    }
}

impl Pipeline for JumpFloodingPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.init_seeds_pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.iterate_pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.sdf_pipeline)
    }
}

impl FromWorld for JumpFloodingPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let jump_flooding_step_bind_group_layout = BindGroupLayoutDescriptor::new(
            "JumpFloodingStepBindGroupLayout",
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<JumpFloodingUniform>(false),
            ),
        );
        let read_only_seeds_bind_group_layout = BindGroupLayoutDescriptor::new(
            "ReadOnlySeedsBindGroupLayout",
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                texture_storage_2d(TextureFormat::Rg32Float, StorageTextureAccess::ReadOnly),
            ),
        );
        let write_only_seeds_bind_group_layout = BindGroupLayoutDescriptor::new(
            "WriteOnlySeedsBindGroupLayout",
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                texture_storage_2d(TextureFormat::Rg32Float, StorageTextureAccess::WriteOnly),
            ),
        );
        let init_seeds_bind_group_layout =
            JumpFloodingInitializeSeedsResource::bind_group_layout_descriptor(render_device);
        let sdf_bind_group_layout =
            JumpFloodingCalculateSdfResource::bind_group_layout_descriptor(render_device);

        let init_seeds_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("ReinitializeLevelset_InitializeSeeds".into()),
                layout: vec![
                    init_seeds_bind_group_layout.clone(),
                    write_only_seeds_bind_group_layout.clone(),
                ],
                shader: load_embedded_asset!(asset_server, "shaders/jump_flooding/initialize.wgsl"),
                entry_point: Some("initialize".into()),
                ..default()
            });

        let iterate_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("ReinitializeLevelset_Iterate".into()),
            layout: vec![
                read_only_seeds_bind_group_layout.clone(),
                write_only_seeds_bind_group_layout.clone(),
                jump_flooding_step_bind_group_layout.clone(),
            ],
            shader: load_embedded_asset!(asset_server, "shaders/jump_flooding/iterate.wgsl"),
            entry_point: Some("iterate".into()),
            ..default()
        });

        let sdf_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("ReinitializeLevelset_Sdf".into()),
            layout: vec![
                sdf_bind_group_layout.clone(),
                read_only_seeds_bind_group_layout.clone(),
            ],
            shader: load_embedded_asset!(asset_server, "shaders/jump_flooding/calculate_sdf.wgsl"),
            entry_point: Some("calculate_sdf".into()),
            ..default()
        });

        JumpFloodingPipeline {
            init_seeds_pipeline,
            iterate_pipeline,
            sdf_pipeline,
            init_seeds_bind_group_layout,
            jump_flooding_step_bind_group_layout,
            sdf_bind_group_layout,
            read_only_seeds_bind_group_layout,
            write_only_seeds_bind_group_layout,
        }
    }
}

fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<JumpFloodingPipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    query: Query<(
        Entity,
        &FluidSettings,
        &JumpFloodingInitializeSeedsResource,
        &JumpFloodingCalculateSdfResource,
        &JumpFloodingSeedsTextures,
    )>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, settings, init_seeds_resource, sdf_resource, seeds_textures) in &query {
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
            jump_flooding_step_bind_groups.push(
                render_device.create_bind_group(
                    Some("JumpFloodingStepBindGroup"),
                    &pipeline_cache
                        .get_bind_group_layout(&pipeline.jump_flooding_step_bind_group_layout),
                    &BindGroupEntries::single(buffer.binding().unwrap()),
                ),
            );
        }

        let init_seeds_bind_group = init_seeds_resource
            .as_bind_group(
                &pipeline.init_seeds_bind_group_layout,
                &render_device,
                &pipeline_cache,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let sdf_bind_group = sdf_resource
            .as_bind_group(
                &pipeline.sdf_bind_group_layout,
                &render_device,
                &pipeline_cache,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let seeds0 = param.0.get(&seeds_textures.0[0]).unwrap();
        let seeds1 = param.0.get(&seeds_textures.0[1]).unwrap();

        let mut read_only_seeds_bind_groups = Vec::with_capacity(2);
        read_only_seeds_bind_groups.push(render_device.create_bind_group(
            Some("ReadOnlySeedsBindGroup0"),
            &pipeline_cache.get_bind_group_layout(&pipeline.read_only_seeds_bind_group_layout),
            &BindGroupEntries::single(&seeds0.texture_view),
        ));
        read_only_seeds_bind_groups.push(render_device.create_bind_group(
            Some("ReadOnlySeedsBindGroup1"),
            &pipeline_cache.get_bind_group_layout(&pipeline.read_only_seeds_bind_group_layout),
            &BindGroupEntries::single(&seeds1.texture_view),
        ));

        let mut write_only_seeds_bind_groups = Vec::with_capacity(2);
        write_only_seeds_bind_groups.push(render_device.create_bind_group(
            Some("WriteOnlySeedsBindGroup0"),
            &pipeline_cache.get_bind_group_layout(&pipeline.write_only_seeds_bind_group_layout),
            &BindGroupEntries::single(&seeds0.texture_view),
        ));
        write_only_seeds_bind_groups.push(render_device.create_bind_group(
            Some("WriteOnlySeedsBindGroup1"),
            &pipeline_cache.get_bind_group_layout(&pipeline.write_only_seeds_bind_group_layout),
            &BindGroupEntries::single(&seeds1.texture_view),
        ));

        commands.entity(entity).insert(JumpFloodingBindGroups {
            init_seeds_bind_group,
            jump_flooding_step_bind_groups: jump_flooding_step_bind_groups.into_boxed_slice(),
            sdf_bind_group,
            read_only_seeds_bind_groups: read_only_seeds_bind_groups.into_boxed_slice(),
            write_only_seeds_bind_groups: write_only_seeds_bind_groups.into_boxed_slice(),
        });
    }
}
