extern crate bevy_eulerian_fluid;

use avian2d::PhysicsPlugins;
use bevy::{
    asset::AssetMetaCheck,
    camera::ScalingMode,
    prelude::*,
    render::{
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
};

use bevy_eulerian_fluid::{
    fluid_source::{FluidSource, FluidSourceMode, FluidSourceOneshot, FluidSourceShape},
    material::VelocityMaterial,
    projection::{gauss_seidel::GaussSeidelConfig, ProjectionMethod},
    settings::{FluidSettings, FluidTextures},
    FluidPlugin,
};
use example_utils::{
    fps_counter::FpsCounterPlugin,
    material::{ExampleMaterialsPlugin, LevelsetMaterial},
    mouse_motion,
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;
const SIZE: UVec2 = UVec2::splat(256);
const LENGTH_UNIT: f32 = 10.0;

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
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (WIDTH, HEIGHT).into(),
                    title: "bevy fluid".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            })
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

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal {
                viewport_width: WIDTH as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    let mesh = meshes.add(Rectangle::from_size(SIZE.as_vec2()));
    commands
        .spawn((
            FluidSettings {
                rho: 99.7, // water density in 2D
                gravity: Vec2::Y * 9.8,
                size: SIZE,
            },
            ProjectionMethod::GaussSeidel(GaussSeidelConfig { num_iterations: 20 }),
            Mesh2d(mesh),
            Transform::default()
                .with_translation((SIZE.as_vec2() * Vec2::new(-0.5, 0.0)).extend(0.0)),
        ))
        .with_child((
            FluidSource {
                active: true,
                mode: FluidSourceMode::Source,
            },
            Transform::from_translation((Vec2::new(0.0, -0.2) * SIZE.as_vec2()).extend(0.0)),
            FluidSourceShape::Aabb {
                half_size: 0.5 * Vec2::new(1.0, 0.6) * SIZE.as_vec2(),
            },
            FluidSourceOneshot,
        ));
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(Entity, &FluidTextures), Added<FluidTextures>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LevelsetMaterial>>,
    mut velocity_materials: ResMut<Assets<VelocityMaterial>>,
) {
    for (entity, fluid_textures) in &query {
        let material = materials.add(LevelsetMaterial {
            levelset: fluid_textures.levelset_air.clone(),
            base_color: Vec3::new(0.0, 0.0, 1.0),
        });

        commands.entity(entity).insert(MeshMaterial2d(material));

        let material_velocity = velocity_materials.add(VelocityMaterial {
            u_range: Vec2::new(-20.0, 20.0),
            v_range: Vec2::new(-20.0, 20.0),
            u: fluid_textures.u.clone(),
            v: fluid_textures.v.clone(),
        });

        let mesh = meshes.add(Rectangle::from_size(SIZE.as_vec2()));
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material_velocity),
            Transform::default()
                .with_translation((SIZE.as_vec2() * Vec2::new(0.5, 0.0)).extend(0.0)),
        ));

        // Draw labels for each panel
        commands.spawn((
            Text::new("Left: Surface, Right: Velocity"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor::WHITE,
        ));
    }
}
