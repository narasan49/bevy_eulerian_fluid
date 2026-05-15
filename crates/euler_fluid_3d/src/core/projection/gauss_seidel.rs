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
    core::{
        fluid_uniform::{FluidUniformBindGroup, FluidUniformBindGroupLayout},
        workgroup::WorkgroupShape,
    },
    resource::FluidResources,
};

pub(crate) struct GaussSeidelPass;

impl FluidComputePass for GaussSeidelPass {
    type Pipeline = GaussSeidelPipeline;
    type Resource = GaussSeidelResource;
    type BG = GaussSeidelBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "gauss_seidel.wgsl");
    }
}

#[derive(Clone, Debug)]
pub struct GaussSeidelConfig {
    pub num_iterations: u32,
}

impl Default for GaussSeidelConfig {
    fn default() -> Self {
        Self { num_iterations: 20 }
    }
}

#[derive(Component, ExtractComponent, Clone, AsBindGroup)]
pub(crate) struct GaussSeidelResource {
    #[storage_texture(0, image_format = R32Float, dimension = "3d", access = ReadWrite)]
    p: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    div: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, dimension = "3d", access = ReadOnly)]
    levelset_air: Handle<Image>,
    #[storage_texture(3, image_format = Rgba32Float, dimension = "3d", access = ReadOnly)]
    area_fraction_solid: Handle<Image>,
    #[uniform(4)]
    weight: f32,
    #[uniform(5)]
    resolution_scale: f32,
}

impl GaussSeidelResource {
    pub fn new(resources: &FluidResources) -> Self {
        Self {
            p: resources.p.clone(),
            div: resources.div.clone(),
            levelset_air: resources.levelset_air0.clone(),
            area_fraction_solid: resources.area_fraction_solid.clone(),
            weight: 1.9,
            resolution_scale: 1.0,
        }
    }
}

#[derive(Resource)]
pub(crate) struct GaussSeidelPipeline {
    pipelines: [CachedComputePipelineId; 2],
    pub bind_group_layout: BindGroupLayoutDescriptor,
}

impl FromWorld for GaussSeidelPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let workgroup_shape = world.resource::<WorkgroupShape>();

        let bind_group_layout = GaussSeidelResource::bind_group_layout_descriptor(render_device);
        let uniform_bind_group_layout = world.resource::<FluidUniformBindGroupLayout>();

        let pipeline_red = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("gauss_seidel_pipeline_red".into()),
            layout: vec![
                bind_group_layout.clone(),
                uniform_bind_group_layout.0.clone(),
            ],
            shader: load_embedded_asset!(world, "gauss_seidel.wgsl"),
            entry_point: Some("gauss_seidel_red".into()),
            shader_defs: vec![workgroup_shape.shader_def()],
            ..default()
        });

        let pipeline_black = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("gauss_seidel_pipeline_black".into()),
            layout: vec![
                bind_group_layout.clone(),
                uniform_bind_group_layout.0.clone(),
            ],
            shader: load_embedded_asset!(world, "gauss_seidel.wgsl"),
            entry_point: Some("gauss_seidel_black".into()),
            shader_defs: vec![workgroup_shape.shader_def()],
            ..default()
        });

        Self {
            pipelines: [pipeline_red, pipeline_black],
            bind_group_layout,
        }
    }
}

impl GaussSeidelPipeline {
    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.pipelines[0])
            && is_pipeline_loaded(pipeline_cache, self.pipelines[1])
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_group: &BindGroup,
        uniform_bind_group: &FluidUniformBindGroup,
        num_workgroups: UVec3,
        config: &GaussSeidelConfig,
    ) {
        let pipeline_red = pipeline_cache
            .get_compute_pipeline(self.pipelines[0])
            .unwrap();
        let pipeline_black = pipeline_cache
            .get_compute_pipeline(self.pipelines[1])
            .unwrap();

        pass.set_bind_group(0, bind_group, &[]);
        pass.set_bind_group(
            1,
            &uniform_bind_group.bind_group,
            &[uniform_bind_group.index],
        );
        for _ in 0..config.num_iterations {
            pass.set_pipeline(pipeline_red);
            pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);
            pass.set_pipeline(pipeline_black);
            pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);
        }
    }
}

impl HasBindGroupLayout for GaussSeidelPipeline {
    fn bind_group_layout(&self) -> &bevy::render::render_resource::BindGroupLayoutDescriptor {
        &self.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct GaussSeidelBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for GaussSeidelBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
