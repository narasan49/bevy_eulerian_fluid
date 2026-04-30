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
use example_utils::{mouse_motion, ExampleDefaultPlugins};

const SIZE: UVec2 = UVec2::splat(256);
const LENGTH_UNIT: f32 = 50.0;

fn main() {
    let mut app = App::new();
    app.add_plugins(ExampleDefaultPlugins)
        .add_plugins(FluidPlugin::new(LENGTH_UNIT))
        .add_plugins(PhysicsPlugins::default().with_length_unit(LENGTH_UNIT))
        .add_plugins(FluidDiagnosticsPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, on_fluid_setup)
        .add_systems(Update, mouse_motion);

    app.run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1.2 * SIZE.x as f32,
                min_height: 1.2 * SIZE.y as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands
        .spawn((
            FluidSettings {
                rho: 99.7, // water density in 2D
                gravity: Vec2::ZERO,
                size: SIZE,
            },
            Transform::default().with_scale(SIZE.as_vec2().extend(1.0)),
        ))
        .with_child((
            FluidSource {
                active: true,
                mode: FluidSourceMode::Source,
            },
            FluidSourceShape::Aabb {
                half_size: SIZE.as_vec2() * 0.5,
            },
            FluidSourceOneshot,
            Transform::default(),
        ));
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(Entity, &FluidTextures), Added<FluidTextures>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VelocityMaterial>>,
) {
    for (entity, fluid_texture) in &query {
        let mesh = meshes.add(Rectangle::default());
        let material = materials.add(VelocityMaterial {
            u_range: Vec2::new(-100.0, 100.0),
            v_range: Vec2::new(-100.0, 100.0),
            u: fluid_texture.u.clone(),
            v: fluid_texture.v.clone(),
        });

        commands
            .entity(entity)
            .insert((Mesh2d(mesh), MeshMaterial2d(material)));
    }
}
