pub mod definition;
pub mod fluid_bind_group;
pub mod fluid_to_solid;
pub mod obstacle;
pub mod physics_time;
pub mod render_node;
pub mod setup_components;

use crate::definition::{
    FluidGridLength, SampleForcesResource, SolidCenterTextures, SolidObstaclesBuffer,
};
use crate::euler_fluid::definition::{FluidSettings, LevelsetTextures};
use crate::euler_fluid::fluid_bind_group::FluidBindGroups;
use crate::fluid_bind_group::FluidShaderResourcePlugin;
use crate::material::FluidMaterialPlugin;
use bevy::render::RenderSystems;
use bevy::shader::load_shader_library;
use bevy::{
    prelude::*,
    render::{
        extract_component::{ExtractComponentPlugin, UniformComponentPlugin},
        extract_resource::ExtractResourcePlugin,
        graph::CameraDriverLabel,
        render_graph::RenderGraph,
        Render, RenderApp,
    },
};
use definition::{
    DivergenceTextures, ForcesToSolid, JumpFloodingSeedsTextures, LocalForces, PressureTextures,
    SimulationUniform, SolidForcesBins, SolidVelocityTextures, VelocityTextures,
    VelocityTexturesIntermediate, VelocityTexturesU, VelocityTexturesV,
};
use fluid_bind_group::FluidPipelines;

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
        app.add_plugins(ExtractResourcePlugin::<SolidObstaclesBuffer>::default())
            .add_plugins(ExtractComponentPlugin::<FluidSettings>::default())
            .add_plugins(ExtractComponentPlugin::<FluidBindGroups>::default())
            .add_plugins(ExtractComponentPlugin::<VelocityTextures>::default())
            .add_plugins(ExtractComponentPlugin::<VelocityTexturesU>::default())
            .add_plugins(ExtractComponentPlugin::<VelocityTexturesV>::default())
            .add_plugins(ExtractComponentPlugin::<VelocityTexturesIntermediate>::default())
            .add_plugins(ExtractComponentPlugin::<SolidVelocityTextures>::default())
            .add_plugins(ExtractComponentPlugin::<SolidCenterTextures>::default())
            .add_plugins(ExtractComponentPlugin::<PressureTextures>::default())
            .add_plugins(ExtractComponentPlugin::<DivergenceTextures>::default())
            .add_plugins(ExtractComponentPlugin::<LevelsetTextures>::default())
            .add_plugins(ExtractComponentPlugin::<JumpFloodingSeedsTextures>::default())
            .add_plugins(ExtractComponentPlugin::<LocalForces>::default())
            .add_plugins(ExtractComponentPlugin::<ForcesToSolid>::default())
            .add_plugins(ExtractComponentPlugin::<SolidForcesBins>::default())
            .add_plugins(ExtractComponentPlugin::<SampleForcesResource>::default())
            .add_plugins(ExtractComponentPlugin::<SimulationUniform>::default())
            .add_plugins(UniformComponentPlugin::<SimulationUniform>::default())
            .add_plugins(FluidMaterialPlugin)
            .add_plugins(FluidShaderResourcePlugin)
            .insert_resource(FluidGridLength(1.0 / self.length_unit))
            .add_systems(Update, obstacle::construct_rigid_body_buffer_for_gpu)
            .add_systems(Update, fluid_to_solid::initialize_buffer)
            .add_systems(Update, watch_fluid_component);

        app.add_plugins((
            physics_time::PhysicsFramePlugin,
            definition::FluidParametersPlugin,
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(
                Render,
                fluid_bind_group::prepare_resource_recompute_levelset
                    .in_set(RenderSystems::PrepareResources),
            )
            .add_systems(
                Render,
                fluid_bind_group::prepare_fluid_bind_groups
                    .in_set(RenderSystems::PrepareBindGroups),
            )
            .add_systems(
                Render,
                fluid_bind_group::prepare_fluid_bind_group_for_resources
                    .in_set(RenderSystems::PrepareBindGroups),
            );

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

    fn finish(&self, app: &mut App) {
        // app.init_resource::<Obstacles>();
        app.init_resource::<SolidObstaclesBuffer>();

        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<FluidPipelines>();
    }
}
