use bevy::{
    asset::embedded_asset,
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        extract_component::{ExtractComponentPlugin, UniformComponentPlugin},
        render_graph::{RenderGraphExt, ViewNodeRunner},
        render_resource::SpecializedRenderPipelines,
    },
    shader::load_shader_library,
};

use crate::marching_cubes::{
    compute_node::{MarchingCubesExtractLabel, MarchingCubesExtractNode},
    draw_pipeline::{MarchingCubesDrawPipeline, MarchingCubesUniform, update_transform_uniform},
    extract_pipeline::MarchingCubesExtractPipeline,
    resource::{
        MarchingCubes, MarchingCubesDrawResource, MarchingCubesExtractResource, setup_resources,
    },
    view_node::{MarchingCubesDrawLabel, MarchingCubesDrawNode},
};

pub mod compute_node;
pub mod draw_pipeline;
pub mod extract_pipeline;
pub mod lookup_table;
pub mod resource;
pub mod view_node;

pub struct MarchingCubesPlugin;

impl Plugin for MarchingCubesPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "marching_cubes/marching_cubes_extract.wgsl");
        embedded_asset!(app, "marching_cubes/marching_cubes_view.wgsl");

        load_shader_library!(app, "marching_cubes/lookup_table.wgsl");

        app.add_plugins((
            ExtractComponentPlugin::<MarchingCubesExtractResource>::default(),
            ExtractComponentPlugin::<MarchingCubesDrawResource>::default(),
            ExtractComponentPlugin::<MarchingCubesUniform>::default(),
            UniformComponentPlugin::<MarchingCubesUniform>::default(),
            ExtractComponentPlugin::<MarchingCubes>::default(),
        ))
        .add_systems(Update, (setup_resources, update_transform_uniform));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedRenderPipelines<MarchingCubesDrawPipeline>>()
            .add_systems(
                Render,
                (
                    extract_pipeline::prepare_marching_cubes_extract_bind_groups
                        .in_set(RenderSystems::PrepareBindGroups),
                    draw_pipeline::prepare_marching_cubes_draw_pipelines
                        .in_set(RenderSystems::Prepare),
                    draw_pipeline::prepare_marching_cubes_draw_bind_groups
                        .in_set(RenderSystems::PrepareBindGroups),
                ),
            )
            .add_render_graph_node::<MarchingCubesExtractNode>(Core3d, MarchingCubesExtractLabel)
            .add_render_graph_node::<ViewNodeRunner<MarchingCubesDrawNode>>(
                Core3d,
                MarchingCubesDrawLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    MarchingCubesExtractLabel,
                    Node3d::MainOpaquePass,
                    MarchingCubesDrawLabel,
                    Node3d::MainTransparentPass,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<MarchingCubesDrawPipeline>();
        render_app.init_resource::<MarchingCubesExtractPipeline>();
    }
}
