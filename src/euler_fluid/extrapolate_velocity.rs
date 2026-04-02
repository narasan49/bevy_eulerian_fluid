use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupEntries, BindGroupLayoutDescriptor,
            CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache,
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
pub struct InitializeUValid {
    #[storage_texture(0, image_format = R32Sint, access = WriteOnly)]
    pub is_u_valid: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct InitializeVValid {
    #[storage_texture(0, image_format = R32Sint, access = WriteOnly)]
    pub is_v_valid: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub levelset_air: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct ExtrapolateUResource {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Sint, access = ReadOnly)]
    pub in_is_u_valid: Handle<Image>,
    #[storage_texture(2, image_format = R32Sint, access = WriteOnly)]
    pub out_is_u_valid: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct ExtrapolateVResource {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub v0: Handle<Image>,
    #[storage_texture(1, image_format = R32Sint, access = ReadOnly)]
    pub in_is_v_valid: Handle<Image>,
    #[storage_texture(2, image_format = R32Sint, access = WriteOnly)]
    pub out_is_v_valid: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct ExtrapolateVelocityPipeline {
    pub initialize_u_valid_pipeline: CachedComputePipelineId,
    pub initialize_v_valid_pipeline: CachedComputePipelineId,
    pub extrapolate_u_pipeline: CachedComputePipelineId,
    pub extrapolate_v_pipeline: CachedComputePipelineId,
    initialize_u_valid_bind_group_layout: BindGroupLayoutDescriptor,
    initialize_v_valid_bind_group_layout: BindGroupLayoutDescriptor,
    extrapolate_u_bind_group_layout: BindGroupLayoutDescriptor,
    extrapolate_v_bind_group_layout: BindGroupLayoutDescriptor,
}

#[derive(Component)]
pub(crate) struct ExtrapolateVelocityBindGroups {
    pub initialize_u_valid_bind_group: BindGroup,
    pub initialize_v_valid_bind_group: BindGroup,
    pub extrapolate_u_bind_group: BindGroup,
    pub extrapolate_u_reverse_bind_group: BindGroup,
    pub extrapolate_v_bind_group: BindGroup,
    pub extrapolate_v_reverse_bind_group: BindGroup,
}

impl Plugin for ExtrapolateVelocityPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/extrapolate/initialize_u_valid.wgsl");
        embedded_asset!(app, "shaders/extrapolate/initialize_v_valid.wgsl");
        embedded_asset!(app, "shaders/extrapolate/extrapolate_u.wgsl");
        embedded_asset!(app, "shaders/extrapolate/extrapolate_v.wgsl");

        app.add_plugins((
            ExtractComponentPlugin::<InitializeUValid>::default(),
            ExtractComponentPlugin::<InitializeVValid>::default(),
            ExtractComponentPlugin::<ExtrapolateUResource>::default(),
            ExtractComponentPlugin::<ExtrapolateVResource>::default(),
        ));

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

        let initialize_u_valid_bind_group_layout =
            InitializeUValid::bind_group_layout_descriptor(render_device);
        let initialize_v_valid_bind_group_layout =
            InitializeVValid::bind_group_layout_descriptor(render_device);

        let extrapolate_u_bind_group_layout =
            ExtrapolateUResource::bind_group_layout_descriptor(render_device);
        let extrapolate_v_bind_group_layout =
            ExtrapolateVResource::bind_group_layout_descriptor(render_device);

        let initialize_u_valid_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("InitializeUValid".into()),
                layout: vec![initialize_u_valid_bind_group_layout.clone()],
                shader: load_embedded_asset!(
                    asset_server,
                    "shaders/extrapolate/initialize_u_valid.wgsl"
                ),
                entry_point: Some("initialize_u_valid".into()),
                ..default()
            });

        let initialize_v_valid_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("InitializeVValid".into()),
                layout: vec![initialize_v_valid_bind_group_layout.clone()],
                shader: load_embedded_asset!(
                    asset_server,
                    "shaders/extrapolate/initialize_v_valid.wgsl"
                ),
                entry_point: Some("initialize_v_valid".into()),
                ..default()
            });

        let extrapolate_u_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("ExtrapolateU".into()),
                layout: vec![extrapolate_u_bind_group_layout.clone()],
                shader: load_embedded_asset!(
                    asset_server,
                    "shaders/extrapolate/extrapolate_u.wgsl"
                ),
                entry_point: Some("extrapolate_u".into()),
                ..default()
            });

        let extrapolate_v_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("ExtrapolateV".into()),
                layout: vec![extrapolate_v_bind_group_layout.clone()],
                shader: load_embedded_asset!(
                    asset_server,
                    "shaders/extrapolate/extrapolate_v.wgsl"
                ),
                entry_point: Some("extrapolate_v".into()),
                ..default()
            });

        ExtrapolateVelocityPipeline {
            initialize_u_valid_pipeline,
            initialize_v_valid_pipeline,
            extrapolate_u_pipeline,
            extrapolate_v_pipeline,
            initialize_u_valid_bind_group_layout,
            initialize_v_valid_bind_group_layout,
            extrapolate_u_bind_group_layout,
            extrapolate_v_bind_group_layout,
        }
    }
}

fn prepare_bind_groups(
    mut commands: Commands,
    pipeline: Res<ExtrapolateVelocityPipeline>,
    query: Query<(
        Entity,
        &InitializeUValid,
        &InitializeVValid,
        &ExtrapolateUResource,
        &ExtrapolateVResource,
    )>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let mut param = (gpu_images, fallback_image, buffers);
    for (entity, initialize_u_valid, initialize_v_valid, extrapolate_u, extrapolate_v) in &query {
        let initialize_v_valid_bind_group = initialize_v_valid
            .as_bind_group(
                &pipeline.initialize_v_valid_bind_group_layout,
                &render_device,
                &pipeline_cache,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let initialize_u_valid_bind_group = initialize_u_valid
            .as_bind_group(
                &pipeline.initialize_u_valid_bind_group_layout,
                &render_device,
                &pipeline_cache,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let extrapolate_u_bind_group = extrapolate_u
            .as_bind_group(
                &pipeline.extrapolate_u_bind_group_layout,
                &render_device,
                &pipeline_cache,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let extrapolate_v_bind_group = extrapolate_v
            .as_bind_group(
                &pipeline.extrapolate_v_bind_group_layout,
                &render_device,
                &pipeline_cache,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let u0 = param.0.get(&extrapolate_u.u0).unwrap();
        let in_is_u_valid = param.0.get(&extrapolate_u.in_is_u_valid).unwrap();
        let out_is_u_valid = param.0.get(&extrapolate_u.out_is_u_valid).unwrap();

        let extrapolate_u_reverse_bind_group = render_device.create_bind_group(
            None,
            &pipeline_cache.get_bind_group_layout(&pipeline.extrapolate_u_bind_group_layout),
            &BindGroupEntries::sequential((
                &u0.texture_view,
                &out_is_u_valid.texture_view,
                &in_is_u_valid.texture_view,
            )),
        );

        let v0 = param.0.get(&extrapolate_v.v0).unwrap();
        let in_is_v_valid = param.0.get(&extrapolate_v.in_is_v_valid).unwrap();
        let out_is_v_valid = param.0.get(&extrapolate_v.out_is_v_valid).unwrap();

        let extrapolate_v_reverse_bind_group = render_device.create_bind_group(
            None,
            &pipeline_cache.get_bind_group_layout(&pipeline.extrapolate_v_bind_group_layout),
            &BindGroupEntries::sequential((
                &v0.texture_view,
                &out_is_v_valid.texture_view,
                &in_is_v_valid.texture_view,
            )),
        );

        commands
            .entity(entity)
            .insert(ExtrapolateVelocityBindGroups {
                initialize_u_valid_bind_group,
                initialize_v_valid_bind_group,
                extrapolate_u_bind_group,
                extrapolate_v_bind_group,
                extrapolate_u_reverse_bind_group,
                extrapolate_v_reverse_bind_group,
            });
    }
}
