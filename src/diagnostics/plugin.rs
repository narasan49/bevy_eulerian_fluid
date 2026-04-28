use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::{
        diagnostic::RenderDiagnosticsPlugin,
        extract_component::ExtractComponentPlugin,
        gpu_readback::{Readback, ReadbackComplete},
        render_graph::RenderGraph,
        storage::ShaderStorageBuffer,
        RenderApp,
    },
};

use crate::{
    diagnostics::{
        calculate_volume::{CalculateVolumePass, CalculateVolumeResource},
        component::{FluidMaxVelocityMagnitude, FluidMinVelocityMagnitude, FluidVolume, GridSize},
        max_velocity::{MaxVelocityPass, MaxVelocityResource},
        min_velocity::{MinVelocityPass, MinVelocityResource},
        render_node::{DiagnosticsLabel, DiagnosticsNode},
        ui::{setup_diagnostics_ui, update_diagnostics_ui},
    },
    plugin::FluidComputePassPlugin,
    render_node::FluidLabel,
    settings::{FluidSettings, FluidTextures},
};

pub struct FluidDiagnosticsPlugin;

impl Plugin for FluidDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FluidComputePassPlugin::<CalculateVolumePass>::default(),
            FluidComputePassPlugin::<MinVelocityPass>::default(),
            FluidComputePassPlugin::<MaxVelocityPass>::default(),
            ExtractComponentPlugin::<GridSize>::default(),
            FrameTimeDiagnosticsPlugin::default(),
            RenderDiagnosticsPlugin,
        ))
        .add_systems(Startup, setup_diagnostics_ui)
        .add_systems(Update, (on_fluid_setup, update_diagnostics_ui));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        let mut render_world = render_app.world_mut();
        let diagnostics_node = DiagnosticsNode::new(&mut render_world);
        let mut render_graph = render_world.resource_mut::<RenderGraph>();
        render_graph.add_node(DiagnosticsLabel, diagnostics_node);
        render_graph.add_node_edge(DiagnosticsLabel, FluidLabel);
    }
}

fn on_fluid_setup(
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    query: Query<(Entity, &FluidTextures, &FluidSettings), Added<FluidTextures>>,
) {
    for (entity, fluid_textures, settings) in &query {
        let calculate_volume_resource = CalculateVolumeResource::new(&mut buffers, fluid_textures);
        let volume_entity = commands
            .spawn((
                calculate_volume_resource.clone(),
                Readback::buffer(calculate_volume_resource.sum.clone()),
                GridSize(settings.size),
            ))
            .observe(volume_readback)
            .id();

        let min_velocity_resource = MinVelocityResource::new(&mut buffers, fluid_textures);
        let min_velocity_entity = commands
            .spawn((
                min_velocity_resource.clone(),
                Readback::buffer(min_velocity_resource.sum.clone()),
                GridSize(settings.size),
            ))
            .observe(min_velocity_readback)
            .id();

        let max_velocity_resource = MaxVelocityResource::new(&mut buffers, fluid_textures);
        let max_velocity_entity = commands
            .spawn((
                max_velocity_resource.clone(),
                Readback::buffer(max_velocity_resource.sum.clone()),
                GridSize(settings.size),
            ))
            .observe(max_velocity_readback)
            .id();

        commands
            .entity(entity)
            .insert((
                FluidVolume(0.0),
                FluidMinVelocityMagnitude(0.0),
                FluidMaxVelocityMagnitude(0.0),
            ))
            .add_children(&[volume_entity, min_velocity_entity, max_velocity_entity]);
    }
}

fn volume_readback(
    trigger: On<ReadbackComplete>,
    mut fluid_query: Query<&mut FluidVolume, With<FluidSettings>>,
    query: Query<&ChildOf, With<CalculateVolumeResource>>,
) {
    let child = query.get(trigger.entity);
    let Ok(child) = child else {
        return;
    };

    let Ok(mut fluid_volume) = fluid_query.get_mut(child.parent()) else {
        return;
    };
    fluid_volume.0 = trigger.event().to_shader_type();
}

fn min_velocity_readback(
    trigger: On<ReadbackComplete>,
    mut fluid_query: Query<&mut FluidMinVelocityMagnitude, With<FluidSettings>>,
    query: Query<&ChildOf, With<MinVelocityResource>>,
) {
    let child = query.get(trigger.entity);
    let Ok(child) = child else {
        return;
    };

    let Ok(mut min_velocity) = fluid_query.get_mut(child.parent()) else {
        return;
    };
    min_velocity.0 = trigger.event().to_shader_type();
}

fn max_velocity_readback(
    trigger: On<ReadbackComplete>,
    mut fluid_query: Query<&mut FluidMaxVelocityMagnitude, With<FluidSettings>>,
    query: Query<&ChildOf, With<MaxVelocityResource>>,
) {
    let child = query.get(trigger.entity);
    let Ok(child) = child else {
        return;
    };

    let Ok(mut max_velocity) = fluid_query.get_mut(child.parent()) else {
        return;
    };
    max_velocity.0 = trigger.event().to_shader_type();
}
