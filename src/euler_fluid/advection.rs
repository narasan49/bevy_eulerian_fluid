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

use crate::{
    fluid_uniform::{uniform_bind_group_layout_desc, SimulationUniformBindGroup},
    pipeline::{DispatchFluidPass, HasBindGroupLayout, Pipeline},
    plugin::FluidComputePass,
};

pub(crate) struct AdvectionPass;

impl FluidComputePass for AdvectionPass {
    type Pipeline = AdvectionPipeline;
    type Resource = AdvectionResource;
    type BG = AdvectionBindGroup;

    fn register_assets(app: &mut App) {
        embedded_asset!(app, "shaders/advect_velocity.wgsl");
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub(crate) struct AdvectionResource {
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
    advection_bind_group_layout: BindGroupLayoutDescriptor,
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

        let uniform_bind_group_layout = uniform_bind_group_layout_desc();

        let advection_bind_group_layout =
            AdvectionResource::bind_group_layout_descriptor(render_device);

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

impl HasBindGroupLayout for AdvectionPipeline {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor {
        &self.advection_bind_group_layout
    }
}

#[derive(Component)]
pub(crate) struct AdvectionBindGroup {
    pub bind_group: BindGroup,
}

impl From<BindGroup> for AdvectionBindGroup {
    fn from(bind_group: BindGroup) -> Self {
        Self { bind_group }
    }
}

pub(crate) fn dispatch(
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    advection_bind_groups: &AdvectionBindGroup,
    uniform_bind_group: &SimulationUniformBindGroup,
    advection_pipeline: &AdvectionPipeline,
    size: UVec2,
) {
    pass.push_debug_group("Advect velocity");
    let advect_u_pipeline = pipeline_cache
        .get_compute_pipeline(advection_pipeline.advect_u_pipeline)
        .unwrap();
    let advect_v_pipeline = pipeline_cache
        .get_compute_pipeline(advection_pipeline.advect_v_pipeline)
        .unwrap();

    pass.set_pipeline(&advect_u_pipeline);
    pass.set_bind_group(0, &advection_bind_groups.bind_group, &[]);
    pass.set_bind_group(
        1,
        &uniform_bind_group.bind_group,
        &[uniform_bind_group.index],
    );
    pass.dispatch_x_major(size);

    pass.set_pipeline(&advect_v_pipeline);
    pass.dispatch_y_major(size);
    pass.pop_debug_group();
}
