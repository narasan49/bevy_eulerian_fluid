use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_eulerian_fluid::{
    settings::FluidSettings,
    velocity_overlay::{
        InitialOverlayVisibility, VelocityOverlay, VelocityOverlayGroup, VelocityOverlayPlugin,
    },
};

pub struct OverlayPlugin<const I: u32>;

impl<const I: u32> Plugin for OverlayPlugin<I> {
    fn build(&self, app: &mut App) {
        app.add_plugins(VelocityOverlayPlugin)
            .add_systems(Update, on_fluid_spawn::<I>)
            .add_systems(
                Update,
                toggle_arrow_visibility.run_if(input_just_pressed(KeyCode::KeyV)),
            );
    }
}

fn on_fluid_spawn<const I: u32>(
    mut commands: Commands,
    query: Query<Entity, Added<FluidSettings>>,
) {
    for entity in &query {
        commands.entity(entity).insert((
            VelocityOverlay {
                max_clamp_speed: 20.0,
                bin_size: UVec2::splat(I),
                color: LinearRgba::WHITE,
            },
            InitialOverlayVisibility(Visibility::Hidden),
        ));
    }
}

fn toggle_arrow_visibility(mut query: Query<&mut Visibility, With<VelocityOverlayGroup>>) {
    for mut visibility in &mut query {
        match *visibility {
            Visibility::Inherited => {
                *visibility = Visibility::Hidden;
            }
            Visibility::Hidden => {
                *visibility = Visibility::Visible;
            }
            Visibility::Visible => {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
