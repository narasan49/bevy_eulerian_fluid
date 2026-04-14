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
};

use crate::fluid_source::{FluidSource, FluidSourceShape, FluidSourceVelocity};

pub const MAX_FLUID_SOURCE: usize = 16;

pub(super) struct FluidSourceUniformPlugin;

impl Plugin for FluidSourceUniformPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<FluidSourceUniform>::default(),
            UniformComponentPlugin::<FluidSourceUniform>::default(),
        ));

        app.add_systems(FixedPostUpdate, update_fluid_source_buffer);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(
            Render,
            prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }
}

#[derive(Clone, Copy, ShaderType, Default)]
pub(crate) struct FluidSourceData {
    pub center: Vec2,
    pub data: Vec2,
    pub velocity: Vec2,
    pub shape_type: u32,
    pub mode: u32,
}

#[derive(Component, ExtractComponent, Clone, ShaderType, Default)]
pub(crate) struct FluidSourceUniform {
    pub data: [FluidSourceData; MAX_FLUID_SOURCE],
    pub count: u32,
}

#[derive(Component)]
pub(crate) struct FluidSourceUniformBindGroup {
    pub bind_group: BindGroup,
    pub index: u32,
}

fn update_fluid_source_buffer(
    mut q_fluid: Query<(&mut FluidSourceUniform, Option<&Children>)>,
    q_source: Query<(
        &FluidSource,
        &FluidSourceShape,
        &Transform,
        &FluidSourceVelocity,
    )>,
) {
    for (mut uniform, children) in &mut q_fluid {
        let mut count = 0;
        let mut data = [FluidSourceData::default(); MAX_FLUID_SOURCE];
        let Some(children) = children else {
            continue;
        };
        for &child in children {
            if let Ok((source, shape, transform, velocity)) = q_source.get(child) {
                if !source.active {
                    continue;
                }
                if count >= MAX_FLUID_SOURCE {
                    warn!(
                        "The maximum number of fluid source per fluid component is {}.",
                        MAX_FLUID_SOURCE
                    );
                    break;
                }
                data[count] = FluidSourceData {
                    center: transform.translation.xy(),
                    data: shape.to_vec2(),
                    velocity: velocity.0,
                    shape_type: shape.shape_type_digit(),
                    mode: source.mode.to_u32(),
                };
                count += 1;
            }
        }

        uniform.count = count as u32;
        uniform.data = data;
    }
}

fn prepare_bind_groups(
    mut commands: Commands,
    fluid_source_uniform: Res<ComponentUniforms<FluidSourceUniform>>,
    query: Query<(Entity, &DynamicUniformIndex<FluidSourceUniform>)>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
) {
    let fluid_source_uniform = fluid_source_uniform.uniforms();

    let bind_group_layout_descriptor = BindGroupLayoutDescriptor::new(
        "FluidSourceUniformBindGroupLayout",
        &BindGroupLayoutEntries::single(
            ShaderStages::COMPUTE,
            uniform_buffer::<FluidSourceUniform>(true),
        ),
    );

    let uniform_bind_group = render_device.create_bind_group(
        "FluidSourceUniformBindGroup",
        &pipeline_cache.get_bind_group_layout(&bind_group_layout_descriptor),
        &BindGroupEntries::single(fluid_source_uniform),
    );

    for (entity, uniform_index) in &query {
        commands.entity(entity).insert(FluidSourceUniformBindGroup {
            bind_group: uniform_bind_group.clone(),
            index: uniform_index.index(),
        });
    }
}
