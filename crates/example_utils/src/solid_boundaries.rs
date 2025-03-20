use bevy::prelude::*;
use bevy_eulerian_fluid::geometry::{RectangleComponent, Velocity};

pub fn solid_boundaries<const W: usize, const H: usize>(mut commands: Commands) {
    // Top wall
    commands.spawn((
        RectangleComponent {
            rectangle: Rectangle {
                half_size: Vec2::new(W as f32, 3.0),
            },
        },
        Transform::IDENTITY,
        Velocity(Vec2::ZERO),
    ));

    // Left wall
    commands.spawn((
        RectangleComponent {
            rectangle: Rectangle {
                half_size: Vec2::new(3.0, H as f32),
            },
        },
        Transform::IDENTITY,
        Velocity(Vec2::ZERO),
    ));

    // Bottom wall
    commands.spawn((
        RectangleComponent {
            rectangle: Rectangle {
                half_size: Vec2::new(W as f32, 3.0),
            },
        },
        Transform::from_translation(Vec3::new(0.0, H as f32 - 1.0, 0.0)),
        Velocity(Vec2::ZERO),
    ));

    // Right wall
    commands.spawn((
        RectangleComponent {
            rectangle: Rectangle {
                half_size: Vec2::new(3.0, H as f32),
            },
        },
        Transform::from_translation(Vec3::new(W as f32 - 1.0, 0.0, 0.0)),
        Velocity(Vec2::ZERO),
    ));
}
