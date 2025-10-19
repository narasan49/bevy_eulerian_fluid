use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

#[derive(Component)]
pub struct FpsRoot;

#[derive(Component)]
pub struct FpsText;

/// FPS counter plugin comming from Unofficial Bevy Cheat Book,
/// https://bevy-cheatbook.github.io/cookbook/print-framerate.html
pub struct FpsCounterPlugin;

impl Plugin for FpsCounterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, setup_fps_text)
            .add_systems(Update, update_fps_text);
    }
}

fn setup_fps_text(mut commands: Commands) {
    let root = commands
        .spawn((
            FpsRoot,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Percent(1.0),
                top: Val::Percent(1.0),
                bottom: Val::Auto,
                left: Val::Auto,
                ..default()
            },
        ))
        .id();

    let text = commands
        .spawn((
            Text::new("FPS: "),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor::WHITE,
        ))
        .with_child((
            TextSpan::default(),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor::WHITE,
            FpsText,
        ))
        .id();
    commands.entity(root).add_child(text);
}

fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut TextSpan, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            **text = format!("{fps:>4.0}");
        } else {
            **text = "N/A".into();
        }
    }
}
