use bevy::{
    render::render_resource::{CachedComputePipelineId, CachedPipelineState, PipelineCache},
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
                panic!("{err}");
            }
            _ => false,
        }
    }
}
