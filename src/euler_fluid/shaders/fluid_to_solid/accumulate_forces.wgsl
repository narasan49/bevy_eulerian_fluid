#import bevy_fluid::fluid_to_solid::fixed_point_conversion::{i32_to_f32};

const MAX_SOLIDS: u32 = 256;

struct Force {
    force: vec2<f32>,
    torque: f32,
}

@group(0) @binding(0) var<storage, read_write> bins_force_x: array<atomic<i32>>;
@group(0) @binding(1) var<storage, read_write> bins_force_y: array<atomic<i32>>;
@group(0) @binding(2) var<storage, read_write> bins_torque: array<atomic<i32>>;

@group(1) @binding(0) var<storage, read_write> forces: array<Force>;

@compute @workgroup_size(1, 1, 1)
fn accumulate_forces(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.x;
    let length = arrayLength(&forces);
    if (length <= idx) {
        return;
    }

    let force = vec2<f32>(
        i32_to_f32(atomicLoad(&bins_force_x[idx])),
        i32_to_f32(atomicLoad(&bins_force_y[idx]))
    );
    let torque = i32_to_f32(atomicLoad(&bins_torque[idx]));
    forces[idx] = Force(force, torque);
}