#import bevy_fluid::particle_levelset::constants::PARTICLE_WORKWGROUP_SIZE;
#import bevy_fluid::particle_levelset::particle::{Particle, is_particle_escaped};
#import bevy_fluid::particle_levelset::fixed_point::f32_to_i32;

@group(0) @binding(0) var<storage, read> particles_count: u32;
@group(0) @binding(1) var<storage, read> particles: array<Particle>;
@group(0) @binding(2) var levelset_air: texture_storage_2d<r32float, read>;
@group(0) @binding(3) var<storage, read_write> phi_correction: array<atomic<i32>>;

@compute @workgroup_size(PARTICLE_WORKWGROUP_SIZE)
fn accumulate_levelset_correction(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let p_idx = global_invocation_id.x;
    if p_idx >= particles_count {
        return;
    }
    let p = particles[p_idx];
    if !is_particle_escaped(p) {
        return;
    }
    let dim = textureDimensions(levelset_air);
    let cell_idx_base = vec2<u32>(p.position);
    for (var i = 0u; i < 2u; i++) {
        for (var j = 0u; j < 2u; j++) {
            let cell_idx = cell_idx_base + vec2<u32>(i, j);
            let cell_idx_1d = cell_idx.x + dim.x * cell_idx.y;
            let phi_p = p.sign * (p.radius - distance(p.position, vec2<f32>(cell_idx)));
            if p.sign > 0.0 {
                atomicMax(&phi_correction[cell_idx_1d], f32_to_i32(phi_p));
            } else {
                atomicMin(&phi_correction[cell_idx_1d], f32_to_i32(phi_p));
            }
        }
    }
    
}
