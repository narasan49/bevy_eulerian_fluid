pub mod jump_flooding;

use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{ComputePass, PipelineCache},
    },
};

use crate::{
    pipeline::{is_pipeline_loaded, DispatchFluidPass},
    reinitialize_levelset::jump_flooding::{
        JumpFloodingBindGroups, JumpFloodingPipeline, JumpFloodingPlugin,
    },
};

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub enum ReinitializeMethod {
    #[default]
    JumpFlooding,
}

#[derive(QueryData)]
pub(crate) struct ReinitializeLevelSetBindGroupQuery {
    pub jump_flooding_bind_groups: Option<&'static JumpFloodingBindGroups>,
}

pub(crate) struct ReinitializeLevelSetPlugin;

impl Plugin for ReinitializeLevelSetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            JumpFloodingPlugin,
            ExtractComponentPlugin::<ReinitializeMethod>::default(),
        ));
    }
}

pub(crate) fn is_pipeline_ready(world: &World, pipeline_cache: &PipelineCache) -> bool {
    let pipeline = world.resource::<JumpFloodingPipeline>();
    is_pipeline_loaded(pipeline_cache, pipeline.init_seeds_pipeline)
        && is_pipeline_loaded(pipeline_cache, pipeline.iterate_pipeline)
        && is_pipeline_loaded(pipeline_cache, pipeline.sdf_pipeline)
}

pub(crate) fn dispatch(
    world: &World,
    method: &ReinitializeMethod,
    pipeline_cache: &PipelineCache,
    pass: &mut ComputePass,
    bind_groups: ReinitializeLevelSetBindGroupQueryItem,
    size: UVec2,
) {
    match method {
        ReinitializeMethod::JumpFlooding => {
            pass.push_debug_group("Reinitialize levelset");

            let pipeline = world.resource::<JumpFloodingPipeline>();
            let bind_groups = bind_groups.jump_flooding_bind_groups.unwrap();
            let init_seeds_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.init_seeds_pipeline)
                .unwrap();
            let iterate_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.iterate_pipeline)
                .unwrap();
            let sdf_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.sdf_pipeline)
                .unwrap();

            pass.set_pipeline(init_seeds_pipeline);
            pass.set_bind_group(0, &bind_groups.init_seeds_bind_group, &[]);
            pass.set_bind_group(1, &bind_groups.write_only_seeds_bind_groups[0], &[]);
            pass.dispatch_center(size);

            pass.set_pipeline(&iterate_pipeline);
            let mut src_idx = 0;
            let mut dst_idx = 1;

            for bind_group in &bind_groups.jump_flooding_step_bind_groups {
                pass.set_bind_group(0, &bind_groups.read_only_seeds_bind_groups[src_idx], &[]);
                pass.set_bind_group(1, &bind_groups.write_only_seeds_bind_groups[dst_idx], &[]);
                pass.set_bind_group(2, bind_group, &[]);
                pass.dispatch_center(size);

                std::mem::swap(&mut src_idx, &mut dst_idx);
            }

            pass.set_pipeline(&sdf_pipeline);
            pass.set_bind_group(0, &bind_groups.sdf_bind_group, &[]);
            pass.set_bind_group(1, &bind_groups.read_only_seeds_bind_groups[src_idx], &[]);

            pass.dispatch_center(size);
            pass.pop_debug_group();
        }
    }
}
