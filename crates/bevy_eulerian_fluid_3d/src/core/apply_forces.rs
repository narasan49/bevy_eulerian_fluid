use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayoutDescriptor, CachedComputePipelineId,
            ComputePass, ComputePipelineDescriptor, PipelineCache, ShaderType,
        },
        renderer::RenderDevice,
        storage::ShaderStorageBuffer,
    },
};
use bevy_eulerian_fluid_common::{
    fluid_compute_pass::FluidComputePass,
    fluid_compute_pipeline::{is_pipeline_loaded, HasBindGroupLayout},
};

use crate::{
    core::{
        fluid_uniform::{FluidUniformBindGroup, FluidUniformBindGroupLayout},
        workgroup::{workgroup_size_xyz, WorkgroupShape},
    },
    resource::FluidResources,
};

pub(crate) struct ApplyForcesPass;

impl FluidComputePass for ApplyForcesPass {
    type Pipeline = ApplyForcesPipeline;
    type Resource = ApplyForcesResource;
    type BG = ApplyForcesBindGroup;
    fn register_assets(app: &mut App) {
        embedded_asset!(app, "apply_forces.wgsl");
        app.add_systems(Update, apply_forces_and_clear);
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct ApplyForcesResource {
    #[storage_texture(0, image_format = R32Float, dimension = "3d", access = ReadWrite)]
    pub u1: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, dimension = "3d", access = ReadWrite)]
    pub v1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, dimension = "3d", access = ReadWrite)]
    pub w1: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage(4, read_only, visibility(compute))]
    pub forces_to_fluid: Handle<ShaderStorageBuffer>,
    #[storage_texture(5, image_format = Rgba32Float, dimension = "3d", access = ReadOnly)]
    pub area_fraction_solid: Handle<Image>,
}

impl ApplyForcesResource {
    pub fn new(resources: &FluidResources, buffers: &mut Assets<ShaderStorageBuffer>) -> Self {
        let forces_to_fluid =
            buffers.add(ShaderStorageBuffer::from(vec![ForceToFluid::default(); 0]));
        Self {
            u1: resources.u1.clone(),
            v1: resources.v1.clone(),
            w1: resources.w1.clone(),
            levelset_air0: resources.levelset_air0.clone(),
            forces_to_fluid,
            area_fraction_solid: resources.area_fraction_solid.clone(),
        }
    }
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct ForceToFluid {
    pub force: Vec3,
    pub position: Vec3,
}

#[derive(Component, Default)]
pub struct ForcesToFluid {
    pub forces: Vec<ForceToFluid>,
}

#[derive(Resource)]
pub(crate) struct ApplyForcesPipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayoutDescriptor,
}

impl ApplyForcesPipeline {
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.pipeline)
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_group: &ApplyForcesBindGroup,
        uniform_bind_group: &FluidUniformBindGroup,
        workgroup_shape: &WorkgroupShape,
        size: UVec3,
    ) {
        pass.push_debug_group("apply_forces");
        let pipeline = pipeline_cache.get_compute_pipeline(self.pipeline).unwrap();

        let num_workgroups = workgroup_size_xyz(size, workgroup_shape);

        pass.set_bind_group(0, &bind_group.bind_group, &[]);
        pass.set_bind_group(
            1,
            &uniform_bind_group.bind_group,
            &[uniform_bind_group.index],
        );

        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);

        pass.pop_debug_group();
    }
}

impl FromWorld for ApplyForcesPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let uniform_bind_group_layout = world.resource::<FluidUniformBindGroupLayout>();

        let bind_group_layout = ApplyForcesResource::bind_group_layout_descriptor(render_device);

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("apply_forces_pipeline".into()),
            layout: vec![
                bind_group_layout.clone(),
                uniform_bind_group_layout.0.clone(),
            ],
            shader: load_embedded_asset!(asset_server, "apply_forces.wgsl"),
            entry_point: Some("apply_forces".into()),
            ..default()
        });

        ApplyForcesPipeline {
            pipeline,
            bind_group_layout,
        }
    }
}

impl HasBindGroupLayout for ApplyForcesPipeline {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor {
        &self.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct ApplyForcesBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for ApplyForcesBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
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
