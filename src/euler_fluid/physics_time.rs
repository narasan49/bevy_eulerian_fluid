use avian2d::prelude::{Physics, PhysicsSchedule, PhysicsStepSet};
use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        RenderApp,
    },
};

pub struct PhysicsFramePlugin;

impl Plugin for PhysicsFramePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsFrameInfo>()
            .add_plugins(ExtractResourcePlugin::<PhysicsFrameInfo>::default())
            .add_systems(
                PhysicsSchedule,
                update_physics_frame_info.after(PhysicsStepSet::Last),
            );

        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<CurrentPhysicsStepNumberRenderWorld>();
    }
}

#[derive(Resource, Debug, Clone, Copy, Default, ExtractResource)]
pub struct PhysicsFrameInfo {
    pub step_number: u64,
    pub delta_secs: f32,
}

/// Step number of physics simulation. This is updated in [`crate::euler_fluid::render_node::EulerFluidNode`].
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CurrentPhysicsStepNumberRenderWorld(pub u64);

pub(crate) fn update_physics_frame_info(
    time: Res<Time<Physics>>,
    mut step: ResMut<PhysicsFrameInfo>,
) {
    let delta = time.delta_secs();
    step.delta_secs = delta;
    step.step_number += 1;
    info!("delta time: {step:?}");
}
