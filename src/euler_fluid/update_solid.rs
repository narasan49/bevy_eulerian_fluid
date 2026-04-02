use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayoutDescriptor, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
        storage::GpuShaderStorageBuffer,
        texture::{FallbackImage, GpuImage},
        Render, RenderApp, RenderSystems,
    },
};

use crate::{
    fluid_uniform::uniform_bind_group_layout_desc, obstacle::SolidObstaclesBuffer,
    pipeline::Pipeline,
};

pub(crate) struct UpdateSolidPlugin;

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct UpdateSolidResource {
    #[storage_texture(0, image_format = R32Float, access = WriteOnly)]
    pub u_solid: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = WriteOnly)]
    pub v_solid: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = WriteOnly)]
    pub levelset_solid: Handle<Image>,
    #[storage_texture(3, image_format = R32Sint, access = WriteOnly)]
    pub solid_id: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct UpdateSolidPipeline {
    pub update_solid_pipeline: CachedComputePipelineId,
    update_solid_bind_group_layout: BindGroupLayoutDescriptor,
}

#[derive(Component)]
pub(crate) struct UpdateSolidBindGroups {
    pub update_solid_bind_group: BindGroup,
}

impl Plugin for UpdateSolidPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/update_solid.wgsl");

        app.add_plugins((ExtractComponentPlugin::<UpdateSolidResource>::default(),));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<UpdateSolidPipeline>();
    }
}

impl Pipeline for UpdateSolidPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.update_solid_pipeline)
    }
}

impl FromWorld for UpdateSolidPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let uniform_bind_group_layout = uniform_bind_group_layout_desc();
        let update_solid_bind_group_layout =
            UpdateSolidResource::bind_group_layout_descriptor(render_device);
        let solid_obstacles_bind_group_layout =
            SolidObstaclesBuffer::bind_group_layout_descriptor(render_device);

        let update_solid_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("UpdateSolidPipeline".into()),
                layout: vec![
                    update_solid_bind_group_layout.clone(),
                    solid_obstacles_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                shader: load_embedded_asset!(asset_server, "shaders/update_solid.wgsl"),
                entry_point: Some("update_solid".into()),
                ..default()
            });

        UpdateSolidPipeline {
            update_solid_pipeline,
            update_solid_bind_group_layout,
        }
    }
}

fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<UpdateSolidPipeline>,
    query: Query<(Entity, &UpdateSolidResource)>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, update_solid_resource) in &query {
        let update_solid_bind_group = update_solid_resource
            .as_bind_group(
                &pipeline.update_solid_bind_group_layout,
                &render_device,
                &pipeline_cache,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands.entity(entity).insert(UpdateSolidBindGroups {
            update_solid_bind_group,
        });
    }
}
