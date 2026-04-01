#import bevy_fluid::particle_levelset::constants::PARTICLE_WORKWGROUP_SIZE;
#import bevy_fluid::particle_levelset::particle::{Particle, is_particle_escaped_u32};
#import bevy_fluid::coordinate::interp2d_center

@group(0) @binding(0) var<storage, read> positive_particles_count: u32;
@group(0) @binding(1) var<storage, read_write> positive_particles: array<Particle>;
@group(0) @binding(2) var<storage, read> negative_particles_count: u32;
@group(0) @binding(3) var<storage, read_write> negative_particles: array<Particle>;
@group(0) @binding(4) var levelset_air: texture_storage_2d<r32float, read>;

@compute @workgroup_size(PARTICLE_WORKWGROUP_SIZE)
fn mark_escaped_particles(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.x;
    if (idx < positive_particles_count) {
        positive_particles[idx].escaped = is_particle_escaped_u32(positive_particles[idx], levelset_air);
    }

    if (idx < negative_particles_count) {
        negative_particles[idx].escaped = is_particle_escaped_u32(negative_particles[idx], levelset_air);
    }
}