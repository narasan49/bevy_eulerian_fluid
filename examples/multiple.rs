extern crate bevy_eulerian_fluid;

use avian2d::PhysicsPlugins;
use bevy::{camera::ScalingMode, prelude::*};
use bevy_eulerian_fluid::{
    diagnostics::FluidDiagnosticsPlugin,
    fluid_source::{FluidSource, FluidSourceMode, FluidSourceOneshot, FluidSourceShape},
    material::VelocityMaterial,
    settings::{FluidSettings, FluidTextures},
    FluidPlugin,
};
use example_utils::{mouse_motion, overlay::OverlayPlugin, ExampleDefaultPlugins};

const LENGTH_UNIT: f32 = 50.0;

fn main() {
    App::new()
        .add_plugins(ExampleDefaultPlugins)
        .add_plugins(FluidPlugin::new(LENGTH_UNIT))
        .add_plugins(PhysicsPlugins::default().with_length_unit(LENGTH_UNIT))
        .add_plugins((FluidDiagnosticsPlugin, OverlayPlugin::<16>))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (mouse_motion, on_fluid_setup))
        .run();
}

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let size = 128u32;
    let nx = 4;
    let ny = 2;
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1.2 * (size * nx) as f32,
                min_height: 1.2 * (size * ny) as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    for i in 0..nx {
        for j in 0..ny {
            let mesh = meshes.add(Rectangle::from_size(Vec2::splat(size as f32)));
            let translation = Vec3::new(
                (i * size) as f32 * 1.1 - size as f32 * 1.6,
                (j * size) as f32 * 1.1 - size as f32 * 0.6,
                0.0,
            );
            commands
                .spawn((
                    FluidSettings {
                        rho: 99.7, // water density in 2D
                        gravity: Vec2::ZERO,
                        size: UVec2::splat(size),
                    },
                    Transform::default().with_translation(translation),
                    Mesh2d(mesh),
                ))
                .with_child((
                    FluidSource {
                        active: true,
                        mode: FluidSourceMode::Source,
                    },
                    FluidSourceShape::Aabb {
                        half_size: Vec2::splat(size as f32),
                    },
                    FluidSourceOneshot,
                    Transform::default(),
                ));
        }
    }

    commands.spawn((
        Text::new("V: Toggle Velocity Overlay"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor::WHITE,
    ));
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(Entity, &FluidTextures), Added<FluidTextures>>,
    mut materials: ResMut<Assets<VelocityMaterial>>,
) {
    for (entity, fluid_texture) in &query {
        let material = materials.add(VelocityMaterial {
            u_range: Vec2::new(-100.0, 100.0),
            v_range: Vec2::new(-100.0, 100.0),
            u: fluid_texture.u.clone(),
            v: fluid_texture.v.clone(),
        });

        commands.entity(entity).insert(MeshMaterial2d(material));
    }
}
