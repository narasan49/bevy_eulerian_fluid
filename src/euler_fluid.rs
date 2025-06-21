pub mod definition;
pub mod fluid_bind_group;
pub mod obstacle;
pub mod render_node;
pub mod setup_components;

use crate::euler_fluid::definition::{FluidSettings, LevelsetTextures};
use crate::euler_fluid::fluid_bind_group::FluidBindGroups;
use crate::material::FluidMaterialPlugin;
use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        extract_component::{ExtractComponentPlugin, UniformComponentPlugin},
        extract_resource::ExtractResourcePlugin,
        graph::CameraDriverLabel,
        render_graph::RenderGraph,
        Render, RenderApp, RenderSet,
    },
};
use definition::{
    DivergenceTextures, ForcesToSolid, JumpFloodingSeedsTextures, LocalForces, Obstacles,
    PressureTextures, SimulationUniform, SolidForcesBins, SolidVelocityTextures, VelocityTextures,
    VelocityTexturesIntermediate, VelocityTexturesU, VelocityTexturesV,
};
use fluid_bind_group::FluidPipelines;

use render_node::{EulerFluidNode, FluidLabel};

use setup_components::watch_fluid_component;

const FLUID_UNIFORM_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x8B9323522322463BA8CF530771C532EF);

const AREA_FRACTION_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x02488F1BF9B14CB2892350B9C578F330);

const COORDINATE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x9F8E2E5B1E5F40C096C31175C285BF11);

const LEVELSET_UTILS_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x998B1DF79E3044B89B0029DCDD0B2B2C);

const SAMPLE_FORCES_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x9DCC97E56F80433A94A50E50DF357E6A);

const ACCUMULATE_FORCES_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0xFF0774E1DC464BEEBC4E502073563979);

const FIXED_POINT_CONVERSION_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0xD734D82B93BF4EC4831C2A627F813304);

pub struct FluidPlugin;

impl Plugin for FluidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<Obstacles>::default())
            .add_plugins(ExtractComponentPlugin::<FluidSettings>::default())
            .add_plugins(ExtractComponentPlugin::<FluidBindGroups>::default())
            .add_plugins(ExtractComponentPlugin::<VelocityTextures>::default())
            .add_plugins(ExtractComponentPlugin::<VelocityTexturesU>::default())
            .add_plugins(ExtractComponentPlugin::<VelocityTexturesV>::default())
            .add_plugins(ExtractComponentPlugin::<VelocityTexturesIntermediate>::default())
            .add_plugins(ExtractComponentPlugin::<SolidVelocityTextures>::default())
            .add_plugins(ExtractComponentPlugin::<PressureTextures>::default())
            .add_plugins(ExtractComponentPlugin::<DivergenceTextures>::default())
            .add_plugins(ExtractComponentPlugin::<LevelsetTextures>::default())
            .add_plugins(ExtractComponentPlugin::<JumpFloodingSeedsTextures>::default())
            .add_plugins(ExtractComponentPlugin::<LocalForces>::default())
            .add_plugins(ExtractComponentPlugin::<ForcesToSolid>::default())
            .add_plugins(ExtractComponentPlugin::<SolidForcesBins>::default())
            .add_plugins(ExtractComponentPlugin::<SimulationUniform>::default())
            .add_plugins(UniformComponentPlugin::<SimulationUniform>::default())
            .add_plugins(FluidMaterialPlugin)
            .add_systems(
                Update,
                (
                    obstacle::update_obstacle_circle,
                    obstacle::update_obstacle_rectangle,
                ),
            )
            .add_systems(Update, watch_fluid_component);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(
                Render,
                fluid_bind_group::prepare_resource_recompute_levelset
                    .in_set(RenderSet::PrepareResources),
            )
            .add_systems(
                Render,
                fluid_bind_group::prepare_fluid_bind_groups.in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                fluid_bind_group::prepare_fluid_bind_group_for_resources
                    .in_set(RenderSet::PrepareBindGroups),
            );

        let mut world = render_app.world_mut();
        let euler_fluid_node = EulerFluidNode::new(&mut world);
        let mut render_graph = world.resource_mut::<RenderGraph>();
        render_graph.add_node(FluidLabel, euler_fluid_node);
        render_graph.add_node_edge(FluidLabel, CameraDriverLabel);

        load_internal_asset!(
            app,
            FLUID_UNIFORM_SHADER_HANDLE,
            "euler_fluid/shaders/fluid_uniform.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::INITIALIZE_GRID_CENTER_SHADER_HANDLE,
            "euler_fluid/shaders/initialize_grid_center.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::INITIALIZE_VELOCITY_SHADER_HANDLE,
            "euler_fluid/shaders/initialize_velocity.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::UPDATE_SOLID_SHADER_HANDLE,
            "euler_fluid/shaders/update_solid.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::UPDATE_SOLID_PRESSURE_HANDLE,
            "euler_fluid/shaders/update_solid_pressure.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::ADVECT_VELOCITY_SHADER_HANDLE,
            "euler_fluid/shaders/advect_velocity.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::APPLY_FORCE_SHADER_HANDLE,
            "euler_fluid/shaders/apply_force.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::DIVERGENCE_SHADER_HANDLE,
            "euler_fluid/shaders/divergence.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::JACOBI_ITERATION_SHADER_HANDLE,
            "euler_fluid/shaders/jacobi_iteration.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::SOLVE_VELOCITY_U_SHADER_HANDLE,
            "euler_fluid/shaders/solve_velocity_u.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::SOLVE_VELOCITY_V_SHADER_HANDLE,
            "euler_fluid/shaders/solve_velocity_v.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::EXTRAPOLATE_VELOCITY_SHADER_HANDLE,
            "euler_fluid/shaders/extrapolate_velocity.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::RECOMPUTE_LEVELSET_INITIALIZE_SHADER_HANDLE,
            "euler_fluid/shaders/recompute_levelset/initialize.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::RECOMPUTE_LEVELSET_ITERATE_SHADER_HANDLE,
            "euler_fluid/shaders/recompute_levelset/iterate.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::RECOMPUTE_LEVELSET_SDF_SHADER_HANDLE,
            "euler_fluid/shaders/recompute_levelset/calculate_sdf.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            fluid_bind_group::ADVECT_LEVELSET_SHADER_HANDLE,
            "euler_fluid/shaders/advect_levelset.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            AREA_FRACTION_SHADER_HANDLE,
            "euler_fluid/shaders/utils/area_fraction.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            COORDINATE_SHADER_HANDLE,
            "euler_fluid/shaders/utils/coordinate.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            LEVELSET_UTILS_SHADER_HANDLE,
            "euler_fluid/shaders/utils/levelset_utils.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SAMPLE_FORCES_SHADER_HANDLE,
            "euler_fluid/shaders/fluid_to_solid/sample_forces.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            ACCUMULATE_FORCES_SHADER_HANDLE,
            "euler_fluid/shaders/fluid_to_solid/accumulate_forces.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            FIXED_POINT_CONVERSION_SHADER_HANDLE,
            "euler_fluid/shaders/fluid_to_solid/fixed_point_conversion.wgsl",
            Shader::from_wgsl
        );
    }

    fn finish(&self, app: &mut App) {
        app.init_resource::<Obstacles>();

        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<FluidPipelines>();
    }
}
