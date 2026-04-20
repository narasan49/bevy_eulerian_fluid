extern crate bevy_eulerian_fluid;

use avian2d::{
    prelude::{IntoCollider, RigidBody},
    PhysicsPlugins,
};
use bevy::{
    asset::AssetMetaCheck,
    camera::ScalingMode,
    color::palettes,
    prelude::*,
    render::{
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
};

use bevy_eulerian_fluid::{
    fluid_source::{
        FluidSource, FluidSourceMode, FluidSourceOneshot, FluidSourceShape, FluidSourceVelocity,
    },
    projection::{multi_grid::MultiGridConfig, ProjectionMethod},
    settings::{FluidSettings, FluidTextures},
    FluidPlugin,
};
use example_utils::{
    fps_counter::FpsCounterPlugin,
    material::{ExampleMaterialsPlugin, VorticityMaterial},
    mouse_motion,
};

const LENGTH_UNIT: f32 = 50.0;
const SIZE: UVec2 = UVec2::new(512, 256);

fn main() {
    let mut app = App::new();
    // [workaround] Asset meta files cannot be found on browser.
    // see also: https://github.com/bevyengine/bevy/issues/10157
    let meta_check = if cfg!(target_arch = "wasm32") {
        AssetMetaCheck::Never
    } else {
        AssetMetaCheck::Always
    };

    app.add_plugins(
        DefaultPlugins
            .set(RenderPlugin {
                render_creation: bevy::render::settings::RenderCreation::Automatic(WgpuSettings {
                    backends: Some(Backends::DX12 | Backends::BROWSER_WEBGPU),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check,
                ..default()
            }),
    )
    .add_plugins(FluidPlugin::new(LENGTH_UNIT))
    .add_plugins(PhysicsPlugins::default().with_length_unit(LENGTH_UNIT))
    .add_plugins((FpsCounterPlugin, ExampleMaterialsPlugin))
    .add_systems(Startup, setup_scene)
    .add_systems(Update, on_fluid_setup)
    .add_systems(Update, mouse_motion);

    app.run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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

    let half_size = SIZE.as_vec2() * 0.5;

    commands
        .spawn((
            FluidSettings {
                rho: 99.7, // water density in 2D
                gravity: Vec2::ZERO,
                size: SIZE,
            },
            ProjectionMethod::MultiGrid(MultiGridConfig::default()),
            Mesh2d(meshes.add(Rectangle::from_size(SIZE.as_vec2()))),
        ))
        .with_children(|commands| {
            commands.spawn((
                FluidSource {
                    active: true,
                    mode: FluidSourceMode::Source,
                },
                FluidSourceShape::Aabb { half_size },
                FluidSourceOneshot,
                Transform::default(),
            ));

            commands.spawn((
                FluidSource {
                    active: true,
                    mode: FluidSourceMode::Source,
                },
                FluidSourceShape::Aabb {
                    half_size: Vec2::splat(20.0),
                },
                Transform::from_translation(Vec3::new(-0.75 * half_size.x, 0.0, 0.0)),
                FluidSourceVelocity(Vec2::new(200.0, 0.0)),
            ));
        });

    let circle = Circle::new(10.0);
    let mesh = meshes.add(circle);
    let material = materials.add(Color::Srgba(palettes::css::LIGHT_CYAN));

    commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_translation(Vec3::new(
            -0.5 * half_size.x + circle.radius * 5.0,
            0.0,
            1.0,
        )),
        circle.collider(),
        RigidBody::Static,
    ));

    let rect = Rectangle::from_size(Vec2::new(0.75 * SIZE.x as f32, 10.0));
    let rect_meth = meshes.add(rect);
    commands.spawn((
        Mesh2d(rect_meth.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
        rect.collider(),
        RigidBody::Static,
    ));
    commands.spawn((
        Mesh2d(rect_meth.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_translation(Vec3::new(0.0, -50.0, 1.0)),
        rect.collider(),
        RigidBody::Static,
    ));
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(Entity, &FluidTextures), Added<FluidTextures>>,
    mut materials: ResMut<Assets<VorticityMaterial>>,
) {
    for (entity, fluid_texture) in &query {
        let material = materials.add(VorticityMaterial {
            u: fluid_texture.u.clone(),
            v: fluid_texture.v.clone(),
        });

        commands.entity(entity).insert(MeshMaterial2d(material));
    }
}
