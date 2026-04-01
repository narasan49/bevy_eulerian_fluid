#import bevy_fluid::particle_levelset::constants::PARTICLE_WORKWGROUP_SIZE
#import bevy_fluid::particle_levelset::particle::{Particle, particle_radius}

@group(0) @binding(0) var levelset_air: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var<storage, read> particles_count: u32;
@group(0) @binding(2) var<storage, read_write> particles: array<Particle>;

@compute @workgroup_size(PARTICLE_WORKWGROUP_SIZE)
fn update_particle_radii(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let p_idx = global_invocation_id.x;
    if p_idx >= particles_count {
        return;
    }

    let cell_idx = vec2i(particles[p_idx].position);

    particles[p_idx].radius = particle_radius(textureLoad(levelset_air, cell_idx).r);
}