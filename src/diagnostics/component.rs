use bevy::{prelude::*, render::extract_component::ExtractComponent};

#[derive(Component, ExtractComponent, Clone)]
pub(crate) struct GridSize(pub UVec2);

#[derive(Component)]
pub struct FluidVolume(pub f32);

#[derive(Component)]
pub struct FluidMinVelocityMagnitude(pub f32);

#[derive(Component)]
pub struct FluidMaxVelocityMagnitude(pub f32);
