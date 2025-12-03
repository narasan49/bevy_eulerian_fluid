pub mod construct_map;
pub mod draw_map;

use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};

pub struct VelocityOverlayPlugin;

#[derive(Component, ExtractComponent, ShaderType, Clone, Copy)]
pub struct VelocityOverlay {
    pub max_clamp_speed: f32,
    pub bin_size: UVec2,
    pub color: LinearRgba,
}

impl Default for VelocityOverlay {
    fn default() -> Self {
        Self {
            max_clamp_speed: 10.0,
            bin_size: UVec2::splat(8),
            color: LinearRgba::RED,
        }
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct VelocityOverlayGroup;

impl Plugin for VelocityOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            construct_map::ConstructVelocityArrowsPlugin,
            draw_map::DrawOverlayVelocityPlugin,
        ));
    }
}
