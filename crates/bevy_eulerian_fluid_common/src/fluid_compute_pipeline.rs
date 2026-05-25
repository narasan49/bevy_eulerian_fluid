use std::path::PathBuf;

use bevy::{
    asset::AssetPath,
    prelude::*,
    render::{
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayoutDescriptor, CachedComputePipelineId,
            CachedPipelineState, ComputePass, ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
    },
    shader::PipelineCacheError,
};

pub fn is_pipeline_loaded(
    pipeline_cache: &PipelineCache,
    pipeline: CachedComputePipelineId,
) -> bool {
    match pipeline_cache.get_compute_pipeline_state(pipeline) {
        CachedPipelineState::Ok(_) => true,
        CachedPipelineState::Err(PipelineCacheError::ShaderNotLoaded(_)) => false,
        CachedPipelineState::Err(err) => {
            panic!("Failed to load compute pipeline: {err}");
        }
        _ => false,
    }
}

pub trait HasBindGroupLayout {
    fn bind_group_layout(&self) -> &BindGroupLayoutDescriptor;
}

pub struct SingleComputePipeline {
    pub pipeline: CachedComputePipelineId,
    pub bind_group_layout: BindGroupLayoutDescriptor,
}

impl SingleComputePipeline {
    pub fn new<B: AsBindGroup>(
        world: &mut World,
        label: &'static str,
        shader: PathBuf,
        entry_point: &'static str,
    ) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let bind_group_layout = B::bind_group_layout_descriptor(render_device);

        let pipeline = queue_compute_pipeline(
            world,
            label,
            shader,
            entry_point,
            vec![bind_group_layout.clone()],
        );

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    pub fn is_ready(&self, pipeline_cache: &PipelineCache) -> bool {
        is_pipeline_loaded(pipeline_cache, self.pipeline)
    }

    pub fn dispatch(
        &self,
        pipeline_cache: &PipelineCache,
        pass: &mut ComputePass,
        bind_group: &BindGroup,
        num_workgroups: UVec3,
    ) {
        let pipeline = pipeline_cache.get_compute_pipeline(self.pipeline).unwrap();
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);
    }
}

pub fn queue_compute_pipeline(
    world: &mut World,
    label: &'static str,
    shader: PathBuf,
    entry_point: &'static str,
    layouts: Vec<BindGroupLayoutDescriptor>,
) -> CachedComputePipelineId {
    let pipeline_cache = world.resource::<PipelineCache>();
    let asset_server = world.resource::<AssetServer>();

    pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(label.into()),
        layout: layouts,
        shader: asset_server.load(AssetPath::from_path_buf(shader).with_source("embedded")),
        entry_point: Some(entry_point.into()),
        ..default()
    })
}
