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

#[derive(Component, Clone, Default, Debug)]
pub(crate) enum FluidStatus {
    #[default]
    Start,
    Initialize,
    Update,
}

fn extract_fluid_status(mut commands: Commands, mut main_world: ResMut<MainWorld>) {
    let mut fluid_status_query = main_world.query::<(RenderEntity, &mut FluidStatus)>();

    for (render_entity, mut fluid_status) in fluid_status_query.iter_mut(&mut main_world) {
        match *fluid_status {
            FluidStatus::Initialize => {}
            FluidStatus::Update => {}
            FluidStatus::Start => {
                commands.entity(render_entity).insert(FluidStatus::Start);
                *fluid_status = FluidStatus::Initialize;
            }
        }
    }
}
