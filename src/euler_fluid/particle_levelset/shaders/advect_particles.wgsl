#import bevy_fluid::coordinate::tvd_rk3;
#import bevy_fluid::fluid_uniform::SimulationUniform;

@group(0) @binding(0) var<storage, read> count: atomic<u32>;
@group(0) @binding(1) var<storage, read_write> particles: array<vec2<f32>>;
@group(0) @binding(2) var u0: texture_storage_2d<r32float, read>;
@group(0) @binding(3) var v0: texture_storage_2d<r32float, read>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(256, 1, 1)
fn advect_particles(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32,
) {
    let idx = global_invocation_id.x;

    let n = count / 256;
    for (var i: u32 = 0; i < n; i++) {
        let idx = n * local_index + i;

        if (idx >= count) {
            return;
        }
        let particle_position = particles[idx];
        let new_particle_position = tvd_rk3(u0, v0, particle_position, constants.dt);

        particles[idx] = new_particle_position;
    }

}