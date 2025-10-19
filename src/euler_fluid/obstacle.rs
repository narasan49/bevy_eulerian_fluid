use avian2d::{
    parry::shape::{Ball, Cuboid, ShapeType, Triangle},
    prelude::*,
};
use bevy::{
    prelude::*,
    render::{render_resource::ShaderType, storage::ShaderStorageBuffer},
};

use crate::definition::{FluidSettings, SolidEntities};

use super::definition::SolidObstaclesBuffer;

#[derive(ShaderType, Default, Copy, Clone)]
pub struct ShapeVariant {
    pub shape: u32,
    pub values: [f32; 6],
}

// #[repr(C, align(16))]
#[derive(ShaderType, Default, Copy, Clone)]
pub struct SolidObstacle {
    pub entity_id: u32,
    pub shape: ShapeVariant,
    pub transform: Mat4,
    pub inverse_transform: Mat4,
    pub linear_velocity: Vec2,
    pub angular_velocity: f32,
}

impl ShapeVariant {
    pub fn from_ball(ball: &Ball) -> Self {
        Self {
            shape: ShapeType::Ball as u32,
            values: [ball.radius, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn from_rectangle(rect: &Cuboid) -> Self {
        Self {
            shape: ShapeType::Cuboid as u32,
            values: [rect.half_extents.x, rect.half_extents.y, 0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn from_triangle(triangle: &Triangle) -> Self {
        Self {
            shape: ShapeType::Triangle as u32,
            values: [
                triangle.a.x,
                triangle.a.y,
                triangle.b.x,
                triangle.b.y,
                triangle.c.x,
                triangle.c.y,
            ],
        }
    }
}

pub(crate) fn construct_rigid_body_buffer_for_gpu(
    query: Query<(
        Entity,
        &GlobalTransform,
        &Collider,
        &LinearVelocity,
        &AngularVelocity,
        &RigidBody,
    )>,
    obstacles_buffer: Res<SolidObstaclesBuffer>,
    mut query_fluid: Query<&mut SolidEntities, With<FluidSettings>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let solid_obstacles = query
        .iter()
        .enumerate()
        .filter_map(
            |(idx, (_, transform, collider, linear_velocity, angular_velocity, _rigid_body))| {
                // match rigid_body {
                //     RigidBody::Static | RigidBody::Kinematic => return None,
                //     RigidBody::Dynamic => {}
                // }

                let shape_type = collider.shape().shape_type();
                let shape = match shape_type {
                    ShapeType::Ball => {
                        Some(ShapeVariant::from_ball(collider.shape().as_ball().unwrap()))
                    }
                    ShapeType::Cuboid => Some(ShapeVariant::from_rectangle(
                        collider.shape().as_cuboid().unwrap(),
                    )),
                    ShapeType::Triangle => Some(ShapeVariant::from_triangle(
                        collider.shape().as_triangle().unwrap(),
                    )),
                    _ => {
                        warn!("Unsupported shape type for solid: {:?}", shape_type);
                        None
                    }
                };

                match shape {
                    Some(shape) => {
                        let transform = transform.to_matrix();
                        Some(SolidObstacle {
                            entity_id: idx as u32,
                            shape,
                            transform,
                            inverse_transform: transform.inverse(),
                            linear_velocity: linear_velocity.0,
                            angular_velocity: angular_velocity.0,
                        })
                    }
                    None => None,
                }
            },
        )
        .collect::<Vec<_>>();

    for mut fluids in &mut query_fluid {
        let solid_entities = query
            .iter()
            .map(|(entity, _, _, _, _, _)| entity)
            .collect::<Vec<_>>();
        fluids.entities = solid_entities;
    }

    let obstacles_buffer = buffers.get_mut(&obstacles_buffer.obstacles).unwrap();
    obstacles_buffer.set_data(solid_obstacles);
}
