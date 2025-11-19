use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::uniform_buffer, AsBindGroup, BindGroup, BindGroupLayout,
            BindGroupLayoutEntries, CachedComputePipelineId, ComputePipelineDescriptor,
            PipelineCache, ShaderStages,
        },
        renderer::RenderDevice,
        storage::GpuShaderStorageBuffer,
        texture::{FallbackImage, GpuImage},
        Render, RenderApp, RenderSystems,
    },
};

use crate::{definition::SimulationUniform, pipeline::Pipeline};

pub(crate) struct AdvectionPlugin;

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct AdvectionResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub v0: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = WriteOnly)]
    pub u1: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = WriteOnly)]
    pub v1: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct AdvectionPipeline {
    pub advect_u_pipeline: CachedComputePipelineId,
    pub advect_v_pipeline: CachedComputePipelineId,
    advection_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub(crate) struct AdvectionBindGroups {
    pub advection_bind_group: BindGroup,
}

impl Plugin for AdvectionPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/advect_velocity.wgsl");

        app.add_plugins((ExtractComponentPlugin::<AdvectionResource>::default(),));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_group.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<AdvectionPipeline>();
    }
}

impl Pipeline for AdvectionPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.advect_u_pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.advect_v_pipeline)
    }
}

impl FromWorld for AdvectionPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let uniform_bind_group_layout = render_device.create_bind_group_layout(
            Some("uniform bind group layout"),
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<SimulationUniform>(true),
            ),
        );

        let advection_bind_group_layout = AdvectionResource::bind_group_layout(render_device);

        let advect_u_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("AdvectUPipeline".into()),
            layout: vec![
                advection_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
            ],
            shader: load_embedded_asset!(asset_server, "shaders/advect_velocity.wgsl"),
            entry_point: Some("advect_u".into()),
            ..default()
        });

        let advect_v_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("AdvectVPipeline".into()),
            layout: vec![
                advection_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
            ],
            shader: load_embedded_asset!(asset_server, "shaders/advect_velocity.wgsl"),
            entry_point: Some("advect_v".into()),
            ..default()
        });

        AdvectionPipeline {
            advect_u_pipeline,
            advect_v_pipeline,
            advection_bind_group_layout,
        }
    }
}

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<AdvectionPipeline>,
    query: Query<(Entity, &AdvectionResource)>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let mut param = (gpu_images, fallback_image, buffers);
    for (entity, advection_resource) in &query {
        let advection_bind_group = advection_resource
            .as_bind_group(
                &pipeline.advection_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands.entity(entity).insert(AdvectionBindGroups {
            advection_bind_group,
        });
    }
}
