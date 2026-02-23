#import bevy_fluid::coordinate::{tvd_rk3, interp2d_center};
#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::particle_levelset::particle::Particle;

@group(0) @binding(0) var<storage, read> count: atomic<u32>;
@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var u0: texture_storage_2d<r32float, read>;
@group(0) @binding(3) var v0: texture_storage_2d<r32float, read>;
@group(0) @binding(4) var levelset_air: texture_storage_2d<r32float, read>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

const PARTICLE_WORKWGROUP_SIZE: u32 = 256;

@compute @workgroup_size(PARTICLE_WORKWGROUP_SIZE, 1, 1)
fn advect_particles(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32,
) {
    let n = count / PARTICLE_WORKWGROUP_SIZE;
    for (var i: u32 = 0; i < n; i++) {
        let idx = n * local_index + i;

        if (idx >= count) {
            return;
        }
        let particle_position = particles[idx].position;
        let new_particle_position = tvd_rk3(u0, v0, particle_position, constants.dt);

        particles[idx].position = new_particle_position;
        particles[idx].level = interp2d_center(levelset_air, new_particle_position);
    }

}