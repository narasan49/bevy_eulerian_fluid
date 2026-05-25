pub mod advect_levelset;
pub mod advect_velocity;
pub mod apply_forces;
pub mod divergence;
pub mod extrapolate_velocity;
pub mod fluid_source;
pub mod fluid_uniform;
pub mod initialize_resources;
pub mod levelset_gradient;
pub mod projection;
pub mod reinitialize_levelset;
pub mod solid_body;
pub mod solve_u;
pub mod solve_v;
pub mod solve_w;
pub mod workgroup;

use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin, graph::CameraDriverLabel,
        render_graph::RenderGraph, RenderApp,
    },
    shader::load_shader_library,
};
use bevy_eulerian_fluid_common::fluid_compute_pass::FluidComputePassPlugin;

use crate::{
    core::{
        apply_forces::ApplyForcesPass,
        extrapolate_velocity::ExtrapolateVelocityPassPlugin,
        fluid_source::FluidSourcePlugin,
        reinitialize_levelset::ReinitializeLevelSetPlugin,
        solid_body::{
            update_area_fraction_solid::UpdateAreaFractionPass, update_solid::UpdateSolidPass,
        },
        workgroup::WorkgroupShape,
    },
    fluid_status::FluidStatusPlugin,
    render::{EulerFluidNode, FluidLabel},
    resource::{setup_fluid_resources, EulerFluid3d, FluidGridLength},
};

pub struct Fluid3dCorePlugin {
    length_unit: f32,
}

impl Fluid3dCorePlugin {
    pub fn new(length_unit: f32) -> Self {
        if length_unit <= 0.0 {
            panic!("length_unit must be positive value.");
        }
        Self { length_unit }
    }
}

impl Plugin for Fluid3dCorePlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "core/area_fraction.wgsl");
        load_shader_library!(app, "core/workgroup_shape.wgsl");
        load_shader_library!(app, "interp.wgsl");

        app.add_plugins((
            ExtractComponentPlugin::<EulerFluid3d>::default(),
            fluid_uniform::FluidUniformPlugin,
            FluidStatusPlugin,
        ))
        .add_plugins((
            FluidComputePassPlugin::<initialize_resources::InitializeResourcesPass>::default(),
            FluidComputePassPlugin::<advect_velocity::AdvectionPass>::default(),
            FluidComputePassPlugin::<divergence::DivergencePass>::default(),
            projection::ProjectionPassPlugin,
            FluidComputePassPlugin::<solve_u::SolveUPass>::default(),
            FluidComputePassPlugin::<solve_v::SolveVPass>::default(),
            FluidComputePassPlugin::<solve_w::SolveWPass>::default(),
            ExtrapolateVelocityPassPlugin,
            FluidComputePassPlugin::<advect_levelset::AdvectLevelSetPass>::default(),
            ReinitializeLevelSetPlugin,
            FluidComputePassPlugin::<levelset_gradient::LevelSetGradientPass>::default(),
        ))
        .add_plugins((
            FluidComputePassPlugin::<UpdateSolidPass>::default(),
            FluidComputePassPlugin::<UpdateAreaFractionPass>::default(),
            FluidComputePassPlugin::<ApplyForcesPass>::default(),
            FluidSourcePlugin,
        ))
        .insert_resource(FluidGridLength(1.0 / self.length_unit))
        .add_systems(Update, setup_fluid_resources);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.insert_resource(WorkgroupShape::Size8x8x4);

        let world = render_app.world_mut();
        let fluid_node = EulerFluidNode::from_world(world);
        let mut render_graph = world.resource_mut::<RenderGraph>();
        render_graph.add_node(FluidLabel, fluid_node);
        render_graph.add_node_edge(FluidLabel, CameraDriverLabel);
    }
}
