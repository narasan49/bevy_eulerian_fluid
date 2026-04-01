#import bevy_fluid::coordinate::{tvd_rk3, interp2d_center};
#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::particle_levelset::particle::Particle;
#import bevy_fluid::particle_levelset::constants::PARTICLE_WORKWGROUP_SIZE;

@group(0) @binding(0) var<storage, read> positive_particles_count: u32;
@group(0) @binding(1) var<storage, read_write> positive_particles: array<Particle>;
@group(0) @binding(2) var<storage, read> negative_particles_count: u32;
@group(0) @binding(3) var<storage, read_write> negative_particles: array<Particle>;
@group(0) @binding(4) var u0: texture_storage_2d<r32float, read>;
@group(0) @binding(5) var v0: texture_storage_2d<r32float, read>;
@group(0) @binding(6) var levelset_air: texture_storage_2d<r32float, read>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(PARTICLE_WORKWGROUP_SIZE, 1, 1)
fn advect_particles(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.x;
    if (idx < positive_particles_count) {
        let particle_position = positive_particles[idx].position;
        let new_particle_position = tvd_rk3(u0, v0, particle_position, constants.dt);

        positive_particles[idx].position = new_particle_position;
    }

    if (idx < negative_particles_count) {
        let particle_position = negative_particles[idx].position;
        let new_particle_position = tvd_rk3(u0, v0, particle_position, constants.dt);

        negative_particles[idx].position = new_particle_position;
    }
}