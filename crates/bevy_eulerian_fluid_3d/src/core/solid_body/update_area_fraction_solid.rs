use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayoutDescriptor, CachedComputePipelineId,
            ComputePass, ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
    },
};
use bevy_eulerian_fluid_common::{
    fluid_compute_pass::FluidComputePass,
    fluid_compute_pipeline::{is_pipeline_loaded, HasBindGroupLayout},
};

use crate::{
    core::workgroup::{workgroup_size_xyz, WorkgroupShape},
    resource::FluidResources,
};

pub(crate) struct UpdateAreaFractionPass;

impl FluidComputePass for UpdateAreaFractionPass {
    type Pipeline = UpdateAreaFractionPipeline;
    type Resource = UpdateAreaFractionResource;
    type BG = UpdateAreaFractionBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "update_area_fraction_solid.wgsl");
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct UpdateAreaFractionResource {
    #[storage_texture(0, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    pub levelset_solid: Handle<Image>,
    #[storage_texture(1, image_format = Rgba32Float, dimension = "3d", access = WriteOnly)]
    pub area_fraction_solid: Handle<Image>,
}

impl UpdateAreaFractionResource {
    pub fn new(resources: &FluidResources) -> Self {
        Self {
            levelset_solid: resources.levelset_solid.clone(),
            area_fraction_solid: resources.area_fraction_solid.clone(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct UpdateAreaFractionPipeline {
    pub pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayoutDescriptor,
}
impl UpdateAreaFractionPipeline {
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.pipeline)
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_group: &UpdateAreaFractionBindGroup,
        workgroup_shape: &WorkgroupShape,
        size: UVec3,
    ) {
        pass.push_debug_group("update_area_fraction_solid");
        let pipeline = pipeline_cache.get_compute_pipeline(self.pipeline).unwrap();

        let num_workgroups = workgroup_size_xyz(size, workgroup_shape);

        pass.set_bind_group(0, &bind_group.bind_group, &[]);

        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);

        pass.pop_debug_group();
    }
}

impl FromWorld for UpdateAreaFractionPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let workgroup_shape = world.resource::<WorkgroupShape>();

        let bind_group_layout =
            UpdateAreaFractionResource::bind_group_layout_descriptor(render_device);

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("update_area_fraction_solid_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: load_embedded_asset!(world, "update_area_fraction_solid.wgsl"),
            entry_point: Some("update_area_fraction_solid".into()),
            shader_defs: vec![workgroup_shape.shader_def()],
            ..default()
        });

        UpdateAreaFractionPipeline {
            pipeline,
            bind_group_layout,
        }
    }
}

impl HasBindGroupLayout for UpdateAreaFractionPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct UpdateAreaFractionBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for UpdateAreaFractionBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
