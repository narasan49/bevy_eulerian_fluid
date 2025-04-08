use bevy::prelude::*;

#[derive(Component)]
pub struct Circle {
    pub radius: f32,
}

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);
