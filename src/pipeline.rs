use bevy::{
    prelude::*,
    render::render_resource::{
        CachedComputePipelineId, CachedPipelineState, ComputePass, PipelineCache,
    },
    shader::PipelineCacheError,
};

pub trait Pipeline {
    fn is_pipeline_state_ready(&self, pipeline_cache: &PipelineCache) -> bool;

    fn is_pipeline_loaded(
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
}

pub trait DispatchFluidPass {
    const WORKGROUP_SIZE: u32 = 8;

    fn dispatch_center(&mut self, size: UVec2);

    fn dispatch_x_major(&mut self, size: UVec2);

    fn dispatch_y_major(&mut self, size: UVec2);
}

impl DispatchFluidPass for ComputePass<'_> {
    fn dispatch_center(&mut self, size: UVec2) {
        self.dispatch_workgroups(
            size.x / Self::WORKGROUP_SIZE,
            size.y / Self::WORKGROUP_SIZE,
            1,
        );
    }

    fn dispatch_x_major(&mut self, size: UVec2) {
        self.dispatch_workgroups(
            size.x + 1,
            size.y / Self::WORKGROUP_SIZE / Self::WORKGROUP_SIZE,
            1,
        );
    }

    fn dispatch_y_major(&mut self, size: UVec2) {
        self.dispatch_workgroups(
            size.x / Self::WORKGROUP_SIZE / Self::WORKGROUP_SIZE,
            size.y + 1,
            1,
        );
    }
}
