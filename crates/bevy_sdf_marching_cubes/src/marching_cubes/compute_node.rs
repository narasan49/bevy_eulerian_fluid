use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{Node, RenderLabel},
        render_resource::{ComputePassDescriptor, PipelineCache},
        storage::GpuShaderStorageBuffer,
    },
};

use crate::marching_cubes::{
    extract_pipeline::{MarchingCubesExtractBindGroup, MarchingCubesExtractPipeline},
    resource::{MarchingCubes, MarchingCubesExtractResource},
};

#[derive(RenderLabel, Debug, Clone, Hash, PartialEq, Eq)]
pub struct MarchingCubesExtractLabel;

pub struct MarchingCubesExtractNode {
    query: QueryState<(
        &'static MarchingCubes,
        &'static MarchingCubesExtractBindGroup,
        &'static MarchingCubesExtractResource,
    )>,
}

impl FromWorld for MarchingCubesExtractNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            query: world.query_filtered(),
        }
    }
}

impl Node for MarchingCubesExtractNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> std::result::Result<(), bevy::render::render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let extract_pipeline = world.resource::<MarchingCubesExtractPipeline>();
        let buffers = world.resource::<RenderAssets<GpuShaderStorageBuffer>>();

        let Some(pipeline) = pipeline_cache.get_compute_pipeline(extract_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        for (marching_cubes, bind_group, extract_resource) in self.query.iter_manual(world) {
            let Some(indirect_args) = buffers.get(&extract_resource.indirect_args) else {
                continue;
            };
            let num_workgroups = marching_cubes.resolution / 8;

            render_context
                .command_encoder()
                .clear_buffer(&indirect_args.buffer, 0, Some(4u64));

            info_once!("[once] running MC extract node");
            let mut pass: bevy::render::render_resource::ComputePass<'_> = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("marching_cubes_extract"),
                    ..default()
                });

            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group.bind_group, &[]);
            pass.dispatch_workgroups(num_workgroups.x, num_workgroups.y, num_workgroups.z);
        }

        // Vertex bufferのサイズに応じてindirect dispatch
        // pass.dispatch_workgroups_indirect(indirect_buffer, indirect_offset);

        Ok(())
    }
}
