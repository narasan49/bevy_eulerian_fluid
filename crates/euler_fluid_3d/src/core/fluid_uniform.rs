use bevy::{
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_resource::{
            binding_types::uniform_buffer, BindGroup, BindGroupEntries, BindGroupLayoutDescriptor,
            BindGroupLayoutEntries, PipelineCache, ShaderStages, ShaderType,
        },
        renderer::RenderDevice,
        Render, RenderApp, RenderSystems,
    },
    shader::load_shader_library,
};

pub struct FluidUniformPlugin;

impl Plugin for FluidUniformPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "fluid_uniform.wgsl");
        app.add_plugins((
            ExtractComponentPlugin::<FluidUniform>::default(),
            UniformComponentPlugin::<FluidUniform>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        let bind_group_layout_descriptor = BindGroupLayoutDescriptor::new(
            "fluid_uniform_bind_group_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<FluidUniform>(true),
            ),
        );

        render_app
            .add_systems(
                Render,
                prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
            )
            .insert_resource(FluidUniformBindGroupLayout(bind_group_layout_descriptor));
    }
}

#[derive(Component, ExtractComponent, ShaderType, Clone, Copy, Default)]
pub struct FluidUniform {
    pub dx: f32,
    pub dt: f32,
    pub rho: f32,
    pub gravity: Vec3,
    pub transform: Mat4,
    pub resolution: UVec3,
}

#[derive(Resource)]
pub struct FluidUniformBindGroupLayout(pub BindGroupLayoutDescriptor);

#[derive(Component)]
pub struct FluidUniformBindGroup {
    pub bind_group: BindGroup,
    pub index: u32,
}

pub(crate) fn prepare_bind_groups(
    mut commands: Commands,
    fluid_uniform: Res<ComponentUniforms<FluidUniform>>,
    bind_group_layout_descriptor: Res<FluidUniformBindGroupLayout>,
    query: Query<(Entity, &DynamicUniformIndex<FluidUniform>)>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
) {
    let simulation_uniform = fluid_uniform.uniforms();

    let bind_group = render_device.create_bind_group(
        "fluid_uniform_bind_group",
        &pipeline_cache.get_bind_group_layout(&bind_group_layout_descriptor.0),
        &BindGroupEntries::single(simulation_uniform),
    );

    for (entity, uniform_index) in &query {
        commands.entity(entity).insert(FluidUniformBindGroup {
            bind_group: bind_group.clone(),
            index: uniform_index.index(),
        });
    }
}

// fn update_simulation_uniform(
//     mut query: Query<(&mut FluidUniform, &Fluid3d, &Transform)>,
//     time_step: Res<FluidTimeStep>,
//     grid_length: Res<FluidGridLength>,
// ) {
//     for (mut uniform, settings, transform) in &mut query {
//         uniform.dx = grid_length.0;
//         uniform.dt = time_step.0;
//         uniform.rho = settings.rho;
//         uniform.gravity = settings.gravity;
//         uniform.fluid_transform = transform.to_matrix();
//         uniform.size = settings.size.as_vec2();
//     }
// }
