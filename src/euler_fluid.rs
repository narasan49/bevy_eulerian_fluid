pub mod advecct_scalar;
pub mod advection;
pub mod apply_forces;
pub mod definition;
pub mod divergence;
pub mod extrapolate_velocity;
pub mod fluid_status;
pub mod fluid_to_solid;
pub mod fluid_uniform;
pub mod initialize;
pub mod obstacle;
pub mod physics_time;
pub mod pipeline;
pub mod reinitialize_levelset;
pub mod render_node;
pub mod setup_components;
pub mod solve_pressure;
pub mod solve_velocity;
pub mod update_solid;

use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin, graph::CameraDriverLabel,
        render_graph::RenderGraph, RenderApp,
    },
    shader::load_shader_library,
};

use crate::material::FluidMaterialPlugin;
use definition::FluidGridLength;
use definition::FluidSettings;
use render_node::{EulerFluidNode, FluidLabel};
use setup_components::watch_fluid_component;

pub struct FluidPlugin {
    length_unit: f32,
}

impl FluidPlugin {
    pub fn new(length_unit: f32) -> Self {
        if length_unit <= 0.0 {
            panic!("length_unit must be positive value.");
        }
        Self { length_unit }
    }
}

impl Plugin for FluidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<FluidSettings>::default())
            .add_plugins((
                initialize::InitializePlugin,
                update_solid::UpdateSolidPlugin,
                advection::AdvectionPlugin,
                apply_forces::ApplyForcesPlugin,
                divergence::DivergencePlugin,
                fluid_uniform::SimulationUniformPlugin,
                solve_pressure::SolvePressurePlugin,
                solve_velocity::SolveVelocityPlugin,
                extrapolate_velocity::ExtrapolateVelocityPlugin,
                advecct_scalar::AdvectScalarPlugin,
                reinitialize_levelset::ReinitializeLevelsetPlugin,
                fluid_to_solid::FluidToSolidForcesPlugin,
            ))
            .add_plugins(FluidMaterialPlugin)
            .add_plugins((
                fluid_status::FluidStatusPlugin,
                physics_time::PhysicsFramePlugin,
            ))
            .insert_resource(FluidGridLength(1.0 / self.length_unit))
            .add_systems(Update, obstacle::construct_rigid_body_buffer_for_gpu)
            .add_systems(Update, watch_fluid_component);

        let render_app = app.sub_app_mut(RenderApp);

        let mut world = render_app.world_mut();
        let euler_fluid_node = EulerFluidNode::new(&mut world);
        let mut render_graph = world.resource_mut::<RenderGraph>();
        render_graph.add_node(FluidLabel, euler_fluid_node);
        render_graph.add_node_edge(FluidLabel, CameraDriverLabel);

        load_shader_library!(app, "euler_fluid/shaders/fluid_uniform.wgsl");
        load_shader_library!(app, "euler_fluid/shaders/utils/area_fraction.wgsl");
        load_shader_library!(app, "euler_fluid/shaders/utils/coordinate.wgsl");
        load_shader_library!(app, "euler_fluid/shaders/utils/levelset_utils.wgsl");
        load_shader_library!(
            app,
            "euler_fluid/shaders/fluid_to_solid/fixed_point_conversion.wgsl"
        );
        load_shader_library!(app, "euler_fluid/shaders/solid_obstacle.wgsl");
    }
}
