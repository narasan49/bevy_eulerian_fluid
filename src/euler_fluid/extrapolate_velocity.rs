use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayout, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
        storage::GpuShaderStorageBuffer,
        texture::{FallbackImage, GpuImage},
        Render, RenderApp, RenderSystems,
    },
};

use crate::pipeline::Pipeline;

pub(crate) struct ExtrapolateVelocityPlugin;

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct ExtrapolateVelocityResource {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub v0: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadOnly)]
    pub levelset_solid: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct ExtrapolateVelocityPipeline {
    pub extrapolate_u_pipeline: CachedComputePipelineId,
    pub extrapolate_v_pipeline: CachedComputePipelineId,
    extrapolate_velocity_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub(crate) struct ExtrapolateVelocityBindGroups {
    pub extrapolate_velocity_bind_group: BindGroup,
}

impl Plugin for ExtrapolateVelocityPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/extrapolate_velocity.wgsl");

        app.add_plugins(ExtractComponentPlugin::<ExtrapolateVelocityResource>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<ExtrapolateVelocityPipeline>();
    }
}

impl Pipeline for ExtrapolateVelocityPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.extrapolate_u_pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.extrapolate_v_pipeline)
    }
}

impl FromWorld for ExtrapolateVelocityPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let extrapolate_velocity_bind_group_layout =
            ExtrapolateVelocityResource::bind_group_layout(render_device);

        let extrapolate_u_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("ExtrapolateU".into()),
                layout: vec![extrapolate_velocity_bind_group_layout.clone()],
                shader: load_embedded_asset!(asset_server, "shaders/extrapolate_velocity.wgsl"),
                entry_point: Some("extrapolate_u".into()),
                ..default()
            });
        let extrapolate_v_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("ExtrapolateV".into()),
                layout: vec![extrapolate_velocity_bind_group_layout.clone()],
                shader: load_embedded_asset!(asset_server, "shaders/extrapolate_velocity.wgsl"),
                entry_point: Some("extrapolate_v".into()),
                ..default()
            });

        ExtrapolateVelocityPipeline {
            extrapolate_u_pipeline,
            extrapolate_v_pipeline,
            extrapolate_velocity_bind_group_layout,
        }
    }
}

fn prepare_bind_groups(
    mut commands: Commands,
    pipeline: Res<ExtrapolateVelocityPipeline>,
    query: Query<(Entity, &ExtrapolateVelocityResource)>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let mut param = (gpu_images, fallback_image, buffers);
    for (entity, extrapolate_velocity) in &query {
        let extrapolate_velocity_bind_group = extrapolate_velocity
            .as_bind_group(
                &pipeline.extrapolate_velocity_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(ExtrapolateVelocityBindGroups {
                extrapolate_velocity_bind_group,
            });
    }
}
