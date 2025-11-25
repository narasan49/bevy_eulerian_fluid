use crate::{
    fluid_uniform::create_uniform_bind_group_layout,
    obstacle::{SolidEntities, SolidObstaclesBuffer},
    physics_time::PhysicsFrameInfo,
    pipeline::Pipeline,
    settings::FluidGridLength,
};
use avian2d::prelude::{Forces, RigidBody, RigidBodyForces};
use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::ExtractResourcePlugin,
        gpu_readback::ReadbackComplete,
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayout, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache, ShaderType,
        },
        renderer::RenderDevice,
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        texture::{FallbackImage, GpuImage},
        Render, RenderApp, RenderSystems,
    },
};

pub const MAX_SOLIDS: usize = 256;

pub(crate) struct FluidToSolidForcesPlugin;

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct SampleForcesResource {
    #[storage(0, visibility(compute))]
    pub bins_force_x: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub bins_force_y: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub bins_torque: Handle<ShaderStorageBuffer>,
    #[storage_texture(3, image_format = R32Float, access = ReadOnly)]
    pub levelset_solid: Handle<Image>,
    #[storage_texture(4, image_format = R32Sint, access = ReadOnly)]
    pub solid_id: Handle<Image>,
    #[storage_texture(5, image_format = R32Float, access = ReadOnly)]
    pub p1: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct AccumulateForcesResource {
    #[storage(0, visibility(compute))]
    pub bins_force_x: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub bins_force_y: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute))]
    pub bins_torque: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute))]
    pub forces: Handle<ShaderStorageBuffer>,
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct FluidToSolidForce {
    pub force: Vec2,
    pub torque: f32,
}

#[derive(Resource)]
pub(crate) struct FluidToSolidForcesPipeline {
    pub sample_forces_pipeline: CachedComputePipelineId,
    pub accumulate_forces_pipeline: CachedComputePipelineId,
    sample_forces_bind_group_layout: BindGroupLayout,
    accumulate_forces_bind_group_layout: BindGroupLayout,
    solid_obstacles_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub(crate) struct FluidToSolidForcesBindGroups {
    pub sample_forces_bind_group: BindGroup,
    pub accumulate_forces_bind_group: BindGroup,
}

#[derive(Resource)]
pub(crate) struct SolidObstaclesBindGroups {
    pub solid_obstacles_bind_group: BindGroup,
}

impl Plugin for FluidToSolidForcesPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/fluid_to_solid/sample_forces.wgsl");
        embedded_asset!(app, "shaders/fluid_to_solid/accumulate_forces.wgsl");

        app.add_plugins((
            ExtractComponentPlugin::<SampleForcesResource>::default(),
            ExtractComponentPlugin::<AccumulateForcesResource>::default(),
            ExtractResourcePlugin::<SolidObstaclesBuffer>::default(),
        ))
        .add_systems(Update, initialize_buffer);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        app.init_resource::<SolidObstaclesBuffer>();

        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<FluidToSolidForcesPipeline>();
    }
}

impl FromWorld for SolidObstaclesBuffer {
    fn from_world(world: &mut World) -> Self {
        let mut buffers = world.resource_mut::<Assets<ShaderStorageBuffer>>();
        let obstacles = buffers.add(ShaderStorageBuffer::from(vec![0; 0]));
        Self { obstacles }
    }
}

impl Pipeline for FluidToSolidForcesPipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        Self::is_pipeline_loaded(pipeline_cache, self.sample_forces_pipeline)
            && Self::is_pipeline_loaded(pipeline_cache, self.accumulate_forces_pipeline)
    }
}

impl FromWorld for FluidToSolidForcesPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let uniform_bind_group_layout = create_uniform_bind_group_layout(render_device);
        let sample_forces_bind_group_layout =
            SampleForcesResource::bind_group_layout(render_device);
        let accumulate_forces_bind_group_layout =
            AccumulateForcesResource::bind_group_layout(render_device);
        let solid_obstacles_bind_group_layout =
            SolidObstaclesBuffer::bind_group_layout(render_device);

        let sample_forces_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("SampleForcesPipeline".into()),
                layout: vec![
                    sample_forces_bind_group_layout.clone(),
                    solid_obstacles_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                shader: load_embedded_asset!(
                    asset_server,
                    "shaders/fluid_to_solid/sample_forces.wgsl"
                ),
                entry_point: Some("sample_forces_to_solid".into()),
                ..default()
            });

        let accumulate_forces_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("AccumulateForcesPipeline".into()),
                layout: vec![accumulate_forces_bind_group_layout.clone()],
                shader: load_embedded_asset!(
                    asset_server,
                    "shaders/fluid_to_solid/accumulate_forces.wgsl"
                ),
                entry_point: Some("accumulate_forces".into()),
                ..default()
            });

        FluidToSolidForcesPipeline {
            sample_forces_pipeline,
            accumulate_forces_pipeline,
            sample_forces_bind_group_layout,
            accumulate_forces_bind_group_layout,
            solid_obstacles_bind_group_layout,
        }
    }
}

pub(crate) fn forces_to_solid_readback(
    trigger: On<ReadbackComplete>,
    mut query: Query<(Forces, &RigidBody)>,
    query_solidentities: Query<&SolidEntities>,
    grid_length: Res<FluidGridLength>,
    physics_frame_info: Res<PhysicsFrameInfo>,
    mut last_physics_step: Local<u64>,
) {
    if physics_frame_info.step_number == *last_physics_step {
        // info!("Skipping forces to solid readback for physics step {}. GPU readback has already been performed for this step.", *last_physics_step);
        return;
    }
    *last_physics_step = physics_frame_info.step_number;

    let data: Vec<FluidToSolidForce> = trigger.event().to_shader_type();
    for fluids in &query_solidentities {
        for (idx, entity) in fluids.entities.iter().enumerate() {
            let rigid_body = query.get_mut(*entity);
            if let Ok((mut forces, rigid_body)) = rigid_body {
                if *rigid_body == RigidBody::Dynamic {
                    let mut force = data[idx].force * physics_frame_info.delta_secs / grid_length.0;
                    force.y *= -1.0;
                    let torque = data[idx].torque * physics_frame_info.delta_secs / grid_length.0;
                    forces.apply_force(force);
                    forces.apply_torque(-torque);
                }
            }
        }
    }
}

fn initialize_buffer(
    query: Query<&AccumulateForcesResource>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    // info!("Initializing forces to solid buffers");
    for bins in query.iter() {
        let bins_force_x = buffers.get_mut(&bins.bins_force_x).unwrap();
        bins_force_x.set_data(vec![0.0; MAX_SOLIDS]);

        let bins_force_y = buffers.get_mut(&bins.bins_force_y).unwrap();
        bins_force_y.set_data(vec![0.0; MAX_SOLIDS]);

        let bins_torque = buffers.get_mut(&bins.bins_torque).unwrap();
        bins_torque.set_data(vec![0.0; MAX_SOLIDS]);

        let forces_buffer = buffers.get_mut(&bins.forces).unwrap();
        forces_buffer.set_data(vec![FluidToSolidForce::default(); MAX_SOLIDS]);
    }
}

fn prepare_bind_groups<'a>(
    mut commands: Commands,
    pipeline: Res<FluidToSolidForcesPipeline>,
    query: Query<(Entity, &SampleForcesResource, &AccumulateForcesResource)>,
    solid_obstacles: Res<SolidObstaclesBuffer>,
    render_device: Res<RenderDevice>,
    mut param: (
        Res<'a, RenderAssets<GpuImage>>,
        Res<'a, FallbackImage>,
        Res<'a, RenderAssets<GpuShaderStorageBuffer>>,
    ),
) {
    for (entity, sample_forces_resource, accumulate_forces_resource) in &query {
        let sample_forces_bind_group = sample_forces_resource
            .as_bind_group(
                &pipeline.sample_forces_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        let accumulate_forces_bind_group = accumulate_forces_resource
            .as_bind_group(
                &pipeline.accumulate_forces_bind_group_layout,
                &render_device,
                &mut param,
            )
            .unwrap()
            .bind_group;

        commands
            .entity(entity)
            .insert(FluidToSolidForcesBindGroups {
                sample_forces_bind_group,
                accumulate_forces_bind_group,
            });
    }

    let solid_obstacles_bind_group = solid_obstacles
        .as_bind_group(
            &pipeline.solid_obstacles_bind_group_layout,
            &render_device,
            &mut param,
        )
        .unwrap()
        .bind_group;

    commands.insert_resource(SolidObstaclesBindGroups {
        solid_obstacles_bind_group,
    });
}
