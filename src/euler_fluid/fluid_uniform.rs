use avian2d::physics_transform::PhysicsTransformSystems;
use bevy::{
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_resource::{
            binding_types::uniform_buffer, BindGroup, BindGroupEntries, BindGroupLayout,
            BindGroupLayoutEntries, ShaderStages, ShaderType,
        },
        renderer::RenderDevice,
        Render, RenderApp, RenderSystems,
    },
};

use crate::{
    definition::{FluidGridLength, FluidSettings},
    physics_time::FluidTimeStep,
};

pub(crate) struct SimulationUniformPlugin;

#[derive(Component, ExtractComponent, ShaderType, Clone, Copy, Default)]
pub struct SimulationUniform {
    pub dx: f32,
    pub dt: f32,
    pub rho: f32,
    pub gravity: Vec2,
    pub initial_fluid_level: f32,
    pub fluid_transform: Mat4,
    pub size: Vec2,
}

#[derive(Resource)]
pub(crate) struct SimulationUniformBindGroupLayout(pub BindGroupLayout);

#[derive(Component)]
pub(crate) struct SimulationUniformBindGroup {
    pub bind_group: BindGroup,
    pub index: u32,
}

impl Plugin for SimulationUniformPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<SimulationUniform>::default(),
            UniformComponentPlugin::<SimulationUniform>::default(),
        ))
        .add_systems(
            FixedPostUpdate,
            update_simulation_uniform.after(PhysicsTransformSystems::Propagate),
        );

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<SimulationUniformBindGroupLayout>();
    }
}

impl FromWorld for SimulationUniformBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let uniform_bind_group_layout = create_uniform_bind_group_layout(render_device);
        SimulationUniformBindGroupLayout(uniform_bind_group_layout)
    }
}

pub(crate) fn create_uniform_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(
        Some("UniformBindGroupLayout"),
        &BindGroupLayoutEntries::single(
            ShaderStages::COMPUTE,
            uniform_buffer::<SimulationUniform>(true),
        ),
    )
}

pub(crate) fn prepare_bind_groups(
    mut commands: Commands,
    simulation_uniform: Res<ComponentUniforms<SimulationUniform>>,
    bind_group_layout: Res<SimulationUniformBindGroupLayout>,
    query: Query<(Entity, &DynamicUniformIndex<SimulationUniform>)>,
    render_device: Res<RenderDevice>,
) {
    let simulation_uniform = simulation_uniform.uniforms();
    let uniform_bind_group = render_device.create_bind_group(
        "SimulationUniformBindGroup",
        &bind_group_layout.0,
        &BindGroupEntries::single(simulation_uniform),
    );

    for (entity, uniform_index) in &query {
        commands.entity(entity).insert(SimulationUniformBindGroup {
            bind_group: uniform_bind_group.clone(),
            index: uniform_index.index(),
        });
    }
}

fn update_simulation_uniform(
    mut query: Query<(&mut SimulationUniform, &FluidSettings, &Transform)>,
    time_step: Res<FluidTimeStep>,
    grid_length: Res<FluidGridLength>,
) {
    for (mut uniform, settings, transform) in &mut query {
        uniform.dx = grid_length.0;
        uniform.dt = time_step.0;
        uniform.rho = settings.rho;
        uniform.gravity = settings.gravity;
        uniform.initial_fluid_level = settings.initial_fluid_level;
        uniform.fluid_transform = transform.to_matrix();
        uniform.size = settings.size.as_vec2();
    }
}
