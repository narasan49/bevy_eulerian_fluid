use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::uniform_buffer, AsBindGroup, BindGroup, BindGroupLayout,
            BindGroupLayoutEntries, CachedComputePipelineId, ComputePipelineDescriptor,
            PipelineCache, ShaderStages, ShaderType,
        },
        renderer::RenderDevice,
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        texture::{FallbackImage, GpuImage},
        Render, RenderApp, RenderSystems,
    },
};

use crate::{fluid_uniform::SimulationUniform, pipeline::Pipeline};

pub(crate) struct ApplyForcesPlugin;

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct ApplyForcesResource {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub u1: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub v1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage(3, read_only, visibility(compute))]
    pub forces_to_fluid: Handle<ShaderStorageBuffer>,
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct ForceToFluid {
    pub force: Vec2,
    pub position: Vec2,
}

#[derive(Component, Default)]
pub struct ForcesToFluid {
    pub forces: Vec<ForceToFluid>,
}

#[derive(Resource)]
pub(crate) struct ApplyForcesPipeline {
    pub apply_forces_u_pipeline: CachedComputePipelineId,
    pub apply_forces_v_pipeline: CachedComputePipelineId,
    apply_forces_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub(crate) struct ApplyForcesBindGroups {
    pub apply_forces_bind_group: BindGroup,
}

impl Plugin for ApplyForcesPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/apply_forces.wgsl");

        app.add_plugins(ExtractComponentPlugin::<ApplyForcesResource>::default());
        app.add_systems(Update, apply_forces_and_clear);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_group.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<ApplyForcesPipeline>();
    }
}

impl Pipeline for ApplyForcesPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.apply_forces_u_pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.apply_forces_v_pipeline)
    }
}

impl FromWorld for ApplyForcesPipeline {
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

        let apply_forces_bind_group_layout = ApplyForcesResource::bind_group_layout(render_device);

        let apply_forces_u_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("ApplyForcesUPipeline".into()),
                layout: vec![
                    apply_forces_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                shader: load_embedded_asset!(asset_server, "shaders/apply_forces.wgsl"),
                entry_point: Some("apply_forces_u".into()),
                ..default()
            });

        let apply_forces_v_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("ApplyForcesVPipeline".into()),
                layout: vec![
                    apply_forces_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                shader: load_embedded_asset!(asset_server, "shaders/apply_forces.wgsl"),
                entry_point: Some("apply_forces_v".into()),
                ..default()
            });

        ApplyForcesPipeline {
            apply_forces_u_pipeline,
            apply_forces_v_pipeline,
            apply_forces_bind_group_layout,
        }
    }
}

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ApplyForcesPipeline>,
    query: Query<(Entity, &ApplyForcesResource)>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let mut param = (gpu_images, fallback_image, buffers);
    for (entity, apply_forces_resource) in &query {
        let apply_forces_bind_group = apply_forces_resource
            .as_bind_group(
                &pipeline.apply_forces_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands.entity(entity).insert(ApplyForcesBindGroups {
            apply_forces_bind_group,
        });
    }
}

fn apply_forces_and_clear(
    mut query: Query<(&mut ForcesToFluid, &ApplyForcesResource)>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (mut forces_to_fluid, apply_forces_resource) in &mut query {
        let forces_buffer = buffers
            .get_mut(&apply_forces_resource.forces_to_fluid)
            .unwrap();
        forces_buffer.set_data(forces_to_fluid.forces.clone());
        forces_to_fluid.forces.clear();
    }
}
