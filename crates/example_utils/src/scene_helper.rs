use avian2d::prelude::{IntoCollider, RigidBody};
use bevy::prelude::*;

pub fn spawn_walls<const X: u32, const Y: u32>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let wall_thickness = 10.0;
    let wall_rect = Rectangle::new(wall_thickness, Y as f32);
    let wall_mesh = meshes.add(wall_rect);
    let wall_material = materials.add(Color::srgb(0.5, 0.5, 0.5));

    let floor_rect = Rectangle::new(X as f32 + 2.0 * wall_thickness, wall_thickness);
    let floor_mesh = meshes.add(floor_rect);

    commands.spawn((
        Mesh2d(wall_mesh.clone()),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz((X as f32 + wall_thickness) * 0.5, 0.0, 0.0),
        RigidBody::Static,
        wall_rect.collider(),
    ));

    commands.spawn((
        Mesh2d(wall_mesh.clone()),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz((X as f32 + wall_thickness) * -0.5, 0.0, 0.0),
        RigidBody::Static,
        wall_rect.collider(),
    ));

    commands.spawn((
        Mesh2d(floor_mesh.clone()),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(0.0, (Y as f32 + wall_thickness) * -0.5, 0.0),
        RigidBody::Static,
        floor_rect.collider(),
    ));
}
