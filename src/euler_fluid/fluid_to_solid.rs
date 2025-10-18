use crate::{
    definition::{FluidGridLength, ForcesToSolid, SolidEntities, SolidForcesBins, MAX_SOLIDS},
    physics_time::PhysicsFrameInfo,
};
use avian2d::prelude::{ExternalForce, RigidBody};
use bevy::{
    gizmos::grid,
    prelude::*,
    render::{gpu_readback::ReadbackComplete, storage::ShaderStorageBuffer},
};

pub(crate) fn forces_to_solid_readback(
    trigger: Trigger<ReadbackComplete>,
    mut query: Query<(&mut ExternalForce, &RigidBody)>,
    query_solidentities: Query<&SolidEntities>,
    grid_length: Res<FluidGridLength>,
    physics_frame_info: Res<PhysicsFrameInfo>,
    mut last_physics_step: Local<u64>,
) {
    if physics_frame_info.step_number == *last_physics_step {
        // info!("Skipping forces to solid readback for physics step {}. GPU readback has already been performed for this step.", *last_physics_step);
        return;
    }
    *last_physics_step = physics_frame_info.step_number;

    let data: Vec<Vec2> = trigger.event().to_shader_type();
    for fluids in &query_solidentities {
        for (idx, entity) in fluids.entities.iter().enumerate() {
            let rigid_body = query.get_mut(*entity);
            if let Ok((mut external_force, rigid_body)) = rigid_body {
                if *rigid_body == RigidBody::Dynamic {
                    let mut force = data[idx] * physics_frame_info.delta_secs / grid_length.0;
                    force.y *= -1.0;
                    external_force.set_force(force);
                }
            }
        }
    }
}

pub(crate) fn initialize_buffer(
    query: Query<(&ForcesToSolid, &SolidForcesBins)>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    // info!("Initializing forces to solid buffers");
    for (forces_to_solid, bins) in query.iter() {
        let bins_x = buffers.get_mut(&bins.bins_x).unwrap();
        bins_x.set_data(vec![0.0; MAX_SOLIDS]);

        let bins_y = buffers.get_mut(&bins.bins_y).unwrap();
        bins_y.set_data(vec![0.0; MAX_SOLIDS]);

        let forces_buffer = buffers.get_mut(&forces_to_solid.forces).unwrap();
        forces_buffer.set_data(vec![Vec2::ZERO; MAX_SOLIDS]);
    }
}
