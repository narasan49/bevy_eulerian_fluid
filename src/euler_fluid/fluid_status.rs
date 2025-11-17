use bevy::{
    prelude::*,
    render::{sync_world::RenderEntity, MainWorld, RenderApp},
};

pub(crate) struct FluidStatusPlugin;

impl Plugin for FluidStatusPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(ExtractSchedule, extract_fluid_status);
    }
}

#[derive(Component, Clone, Copy, Default, Debug)]
pub enum FluidStatus {
    #[default]
    Reset,
    Uninitialized,
    Initialized,
}

#[derive(Component)]
pub struct ResetFluid(bool);

fn extract_fluid_status(mut commands: Commands, mut main_world: ResMut<MainWorld>) {
    let mut fluid_status_query = main_world.query::<(RenderEntity, Option<&mut FluidStatus>)>();

    for (render_entity, fluid_status) in fluid_status_query.iter_mut(&mut main_world) {
        if let Some(mut fluid_status) = fluid_status {
            match *fluid_status {
                FluidStatus::Uninitialized => {}
                FluidStatus::Initialized => {}
                FluidStatus::Reset => {
                    commands.entity(render_entity).insert(FluidStatus::Reset);
                    *fluid_status = FluidStatus::Uninitialized;
                }
            }
        }
    }
}
