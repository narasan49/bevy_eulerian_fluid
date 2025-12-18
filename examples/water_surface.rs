extern crate bevy_eulerian_fluid;

use avian2d::PhysicsPlugins;
use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    render::{
        render_resource::AsBindGroup,
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
};

use bevy_eulerian_fluid::{
    material::VelocityMaterial,
    settings::{FluidSettings, FluidTextures},
    FluidPlugin,
};
use example_utils::{fps_counter::FpsCounterPlugin, mouse_motion};

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
    .add_plugins(FpsCounterPlugin)
    .add_plugins(Material2dPlugin::<CustomMaterial>::default())
    .add_systems(Startup, setup_scene)
    .add_systems(Update, on_fluid_setup)
    .add_systems(Update, mouse_motion);

    app.run();
}

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn(Camera2d);

    let mesh = meshes.add(Rectangle::from_size(SIZE.as_vec2()));
    commands.spawn((
        FluidSettings {
            rho: 99.7, // water density in 2D
            gravity: Vec2::Y * 9.8,
            size: SIZE,
            initial_fluid_level: 0.6,
        },
        Mesh2d(mesh),
        Transform::default().with_translation((SIZE.as_vec2() * Vec2::new(-0.5, 0.0)).extend(0.0)),
    ));
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(Entity, &FluidTextures), Added<FluidTextures>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut velocity_materials: ResMut<Assets<VelocityMaterial>>,
) {
    for (entity, fluid_textures) in &query {
        let material = materials.add(CustomMaterial {
            levelset: fluid_textures.levelset_air.clone(),
            base_color: Vec3::new(0.0, 0.0, 1.0),
            offset: 0.0,
            scale: -100.0,
        });

        commands.entity(entity).insert(MeshMaterial2d(material));

        let material_velocity = velocity_materials.add(VelocityMaterial {
            u_range: Vec2::new(-10.0, 10.0),
            v_range: Vec2::new(-10.0, 10.0),
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

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub levelset: Handle<Image>,
    #[uniform(2)]
    pub base_color: Vec3,
    #[uniform(3)]
    pub offset: f32,
    #[uniform(4)]
    pub scale: f32,
}

impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/visualize/scalar.wgsl".into()
    }
}
