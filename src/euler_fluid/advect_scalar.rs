use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayoutDescriptor, ComputePipelineDescriptor,
            PipelineCache,
        },
        renderer::RenderDevice,
    },
};

use crate::{
    fluid_uniform::uniform_bind_group_layout_desc,
    pipeline::{HasBindGroupLayout, SingleComputePipeline},
    plugin::FluidComputePass,
};

pub(crate) struct AdvectScalarPass;

impl FluidComputePass for AdvectScalarPass {
    type Pipeline = AdvectScalarPipeline;
    type Resource = AdvectLevelsetResource;
    type BG = AdvectScalarBindGroups;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/advect_levelset.wgsl");
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct AdvectLevelsetResource {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadOnly)]
    pub v0: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadOnly)]
    pub levelset_air0: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = WriteOnly)]
    pub levelset_air1: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct AdvectScalarPipeline {
    pub pipeline: SingleComputePipeline,
}

impl FromWorld for AdvectScalarPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let uniform_bind_group_layout = uniform_bind_group_layout_desc();
        let bind_group_layout = AdvectLevelsetResource::bind_group_layout_descriptor(render_device);

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("AdvectLevelsetPipeline".into()),
            layout: vec![bind_group_layout.clone(), uniform_bind_group_layout.clone()],
            shader: load_embedded_asset!(asset_server, "shaders/advect_levelset.wgsl"),
            shader_defs: vec!["CUBIC".into()],
            entry_point: Some("advect_levelset".into()),
            ..default()
        });

        AdvectScalarPipeline {
            pipeline: SingleComputePipeline {
                pipeline,
                bind_group_layout,
            },
        }
    }
}

impl HasBindGroupLayout for AdvectScalarPipeline {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor {
        &self.pipeline.bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct AdvectScalarBindGroups {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for AdvectScalarBindGroups {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}
