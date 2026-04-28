use bevy::{
    diagnostic::{Diagnostic, DiagnosticPath, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::{
    diagnostics::component::{FluidMaxVelocityMagnitude, FluidMinVelocityMagnitude, FluidVolume},
    settings::FluidSettings,
};

#[derive(Component, Clone, Copy)]
pub(crate) enum ItemMarker {
    FPS,
    FrameCount,
    Resolution,
    ComputeShader,
    Volume,
    MinVelocity,
    MaxVelocity,
}

pub(crate) fn setup_diagnostics_ui(mut commands: Commands) {
    let font_size = 10.0;

    let items = vec![
        ("FPS: ", ItemMarker::FPS),
        ("Frame Count: ", ItemMarker::FrameCount),
        ("Resolution: ", ItemMarker::Resolution),
        ("GPU (ms): ", ItemMarker::ComputeShader),
        ("Volume (Approx): ", ItemMarker::Volume),
        ("Min Velocity Mag: ", ItemMarker::MinVelocity),
        ("Max Velocity Mag: ", ItemMarker::MaxVelocity),
    ];
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            right: Val::Percent(1.0),
            top: Val::Percent(1.0),
            bottom: Val::Auto,
            left: Val::Auto,
            ..default()
        })
        .with_children(|commands| {
            items.iter().for_each(|(label, marker)| {
                commands
                    .spawn((
                        Text::new(*label),
                        TextFont::from_font_size(font_size),
                        TextColor::WHITE,
                    ))
                    .with_child((
                        TextSpan::default(),
                        TextFont::from_font_size(font_size),
                        TextColor::WHITE,
                        *marker,
                    ));
            });
        });
}

pub(crate) fn update_diagnostics_ui(
    mut query: Query<(&mut TextSpan, &ItemMarker)>,
    fluid_query: Query<(
        &FluidSettings,
        Option<&FluidVolume>,
        Option<&FluidMinVelocityMagnitude>,
        Option<&FluidMaxVelocityMagnitude>,
    )>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let (settings, volume, min_velocity, max_velocity) = fluid_query
        .single()
        .expect("FluidDiagnostics can work when there is exactly one fluid compoent.");
    for (mut text, marker) in &mut query {
        match marker {
            ItemMarker::FPS => {
                if let Some(fps) = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|fps| fps.smoothed())
                {
                    **text = format!("{fps:>4.0}");
                } else {
                    **text = "N/A".into();
                }
            }
            ItemMarker::FrameCount => {
                if let Some(count) = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FRAME_COUNT)
                    .and_then(|count| count.smoothed())
                {
                    **text = format!("{count:>4.0}");
                } else {
                    **text = "N/A".into();
                }
            }
            ItemMarker::Resolution => {
                **text = format!("{}x{}", settings.size.x, settings.size.y);
            }
            ItemMarker::ComputeShader => {
                let diagnostics_path = DiagnosticPath::new("render/eulerian_fluid/elapsed_gpu");
                if let Some(gpu) = diagnostics
                    .get(&diagnostics_path)
                    .and_then(Diagnostic::average)
                {
                    **text = format!("{gpu:0.4}");
                } else {
                    **text = "N/A".into();
                }
            }
            ItemMarker::Volume => {
                if let Some(volume) = volume {
                    **text = format!("{}", volume.0);
                } else {
                    **text = "N/A".into();
                }
            }
            ItemMarker::MinVelocity => {
                if let Some(min_velocity) = min_velocity {
                    **text = format!("{}", min_velocity.0);
                } else {
                    **text = "N/A".into();
                }
            }
            ItemMarker::MaxVelocity => {
                if let Some(max_velocity) = max_velocity {
                    **text = format!("{:0.4}", max_velocity.0);
                } else {
                    **text = "N/A".into();
                }
            }
        }
    }
}
