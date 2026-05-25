use bevy::{
    prelude::*,
    render::{
        render_graph::{RenderLabel, ViewNode},
        render_resource::{PipelineCache, RenderPassDescriptor, StoreOp},
        view::{ViewDepthTexture, ViewTarget, ViewUniformOffset},
    },
};

use crate::marching_cubes::draw_pipeline::{
    MarchingCubesDrawBindings, MarchingCubesDrawPipelineId,
};

#[derive(RenderLabel, Debug, Clone, Hash, PartialEq, Eq)]
pub struct MarchingCubesDrawLabel;

pub struct MarchingCubesDrawNode {
    fluids: QueryState<&'static MarchingCubesDrawBindings>,
}

impl FromWorld for MarchingCubesDrawNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            fluids: world.query_filtered(),
        }
    }
}

impl ViewNode for MarchingCubesDrawNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewDepthTexture,
        &'static MarchingCubesDrawPipelineId,
        &'static ViewUniformOffset,
    );
    fn update(&mut self, world: &mut World) {
        self.fluids.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        (target, depth, pipeline_id, view_offset): bevy::ecs::query::QueryItem<
            'w,
            '_,
            Self::ViewQuery,
        >,
        world: &'w World,
    ) -> std::result::Result<(), bevy::render::render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id.0) else {
            info!("RenderPipeline in not ready yet.");
            return Ok(());
        };

        for mc_bindings in self.fluids.iter_manual(world) {
            info_once!("[once] ruuning MC view node");
            let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("marching_cubes_draw"),
                color_attachments: &[Some(target.get_color_attachment())],
                depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_render_pipeline(pipeline);
            pass.set_vertex_buffer(0, mc_bindings.vertex_buffer.slice(..));
            pass.set_bind_group(
                0,
                &mc_bindings.uniform_bind_group,
                &[mc_bindings.uniform_index],
            );
            pass.set_bind_group(1, &mc_bindings.view_bind_group, &[view_offset.offset]);
            pass.draw_indirect(&mc_bindings.indirect_buffer, 0);
        }

        Ok(())
    }
}
