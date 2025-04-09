use bevy::{prelude::*, render::storage::ShaderStorageBuffer};

use super::definition::{CircleObstacle, Obstacles, RectangleObstacle};

#[derive(Component)]
pub struct SolidCircle {
    pub radius: f32,
}

impl SolidCircle {
    pub fn from_circle(circle: Circle) -> Self {
        SolidCircle {
            radius: circle.radius,
        }
    }
}

#[derive(Component)]
pub struct SolidRectangle {
    pub half_size: Vec2,
}

impl SolidRectangle {
    pub fn from_rectangle(rectangle: Rectangle) -> Self {
        SolidRectangle {
            half_size: rectangle.half_size,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component, Deref, DerefMut, Default)]
pub struct AngularVelocity(pub f32);

pub(crate) fn update_obstacle_circle(
    query: Query<(&SolidCircle, &Transform, &Velocity)>,
    obstacles: Res<Obstacles>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let circles = query
        .iter()
        .map(|(circle, transform, velocity)| {
            return CircleObstacle {
                radius: circle.radius,
                transform: transform.compute_matrix(),
                velocity: velocity.0,
            };
        })
        .collect::<Vec<_>>();

    let circles_buffer = buffers.get_mut(&obstacles.circles).unwrap();
    circles_buffer.set_data(circles);
}

pub(crate) fn update_obstacle_rectangle(
    query: Query<(&SolidRectangle, &Transform, &Velocity, &AngularVelocity)>,
    obstacles: Res<Obstacles>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let rectangles = query
        .iter()
        .map(|(rectangle, transform, velocity, angular_velocity)| {
            let mat4 = transform.compute_matrix();
            return RectangleObstacle {
                half_size: rectangle.half_size,
                transform: mat4,
                inverse_transform: mat4.inverse(),
                velocity: velocity.0,
                angular_velocity: angular_velocity.0,
            };
        })
        .collect::<Vec<_>>();

    let rectangle_buffer = buffers.get_mut(&obstacles.rectangles).unwrap();
    rectangle_buffer.set_data(rectangles);
}
