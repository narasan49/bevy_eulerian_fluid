pub mod fluid_source_uniform;
pub mod update_fluid_source;

use bevy::prelude::*;

use crate::{
    fluid_source::{
        fluid_source_uniform::FluidSourceUniformPlugin, update_fluid_source::UpdateFluidSourcePass,
    },
    plugin::FluidComputePassPlugin,
};

pub(super) struct FluidSourcePlugin;

impl Plugin for FluidSourcePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FluidSourceUniformPlugin,
            FluidComputePassPlugin::<UpdateFluidSourcePass>::default(),
        ));
    }
}

/// Defines source/sink of fluid. If [`FluidSource`] is spawned as a child of [`crate::euler_fluid::FluidSettings`] component, fluids are sourced or drained with respect to [`FluidSource::mode`] while [`FluidSource::active`] is `true`.
/// The shape is configured by [`FluidSourceShape`].
/// If [`FluidSource::mode`] is [`FluidSourceMode::Source`], the velocity of the souce should be defined by [`FluidSourceVelocity`].
///
/// If you want the fluid to be sourced only on startup, spawn [`FluidSource`] with [`FluidSourceOneshot`]
///
/// See [`crate::euler_fluid::FluidSettings`] for more details.
#[derive(Component, Default)]
#[require(FluidSourceShape, FluidSourceVelocity, Transform)]
pub struct FluidSource {
    pub active: bool,
    pub mode: FluidSourceMode,
}

#[derive(Default)]
pub enum FluidSourceMode {
    #[default]
    Source,
    Sink,
}

impl FluidSourceMode {
    fn to_u32(&self) -> u32 {
        match &self {
            FluidSourceMode::Source => 0,
            FluidSourceMode::Sink => 1,
        }
    }
}

#[derive(Component)]
pub enum FluidSourceShape {
    Circle { radius: f32 },
    Aabb { half_size: Vec2 },
}

impl FluidSourceShape {
    pub fn to_vec2(&self) -> Vec2 {
        match &self {
            FluidSourceShape::Circle { radius } => Vec2::new(*radius, 0.0),
            FluidSourceShape::Aabb { half_size } => *half_size,
        }
    }

    pub fn shape_type_digit(&self) -> u32 {
        match &self {
            FluidSourceShape::Circle { radius: _ } => 0,
            FluidSourceShape::Aabb { half_size: _ } => 1,
        }
    }
}

impl Default for FluidSourceShape {
    fn default() -> Self {
        Self::Circle { radius: 1.0 }
    }
}

#[derive(Component, Default)]
pub struct FluidSourceVelocity(pub Vec2);

/// Spawning a FluidSourceOneshot with [`FluidSource`], [`FluidSource`] will be added only on startup.
/// Used to setup initial fluid domain.
#[derive(Component)]
pub struct FluidSourceOneshot;
