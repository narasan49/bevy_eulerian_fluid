#import bevy_fluid::particle_levelset::particle::{Particle, is_particle_valid, spawn_and_attract_particle};
#import bevy_fluid::particle_levelset::constants::{BAND_WIDTH};
#import bevy_fluid::hash::{hash11, hash22};

@group(0) @binding(0) var<storage, read> particles_to_be_added: array<u32>;
@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<storage, read_write> particles_count: atomic<u32>;
@group(0) @binding(3) var levelset_air: texture_storage_2d<r32float, read>;
@group(0) @binding(4) var grad_levelset_air: texture_storage_2d<rg32float, read>;
@group(0) @binding(5) var<uniform> sign: f32;

@compute @workgroup_size(8, 8, 1)
fn add_particles(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.xy;
    let dim = textureDimensions(levelset_air);
    let fdim = vec2<f32>(dim);

    var bmin = 0.0;
    var bmax = 0.0;
    if sign == 1.0 {
        bmin = 0.1;
        bmax = BAND_WIDTH;
    } else if sign == -1.0 {
        bmin = -BAND_WIDTH;
        bmax = -0.1;
    } else {
        return;
    }

    let idx_1d = idx.x + dim.x * idx.y;
    let x_base = vec2<f32>(idx);
    let n_spawn = particles_to_be_added[idx_1d];

    for (var i = 0u; i < n_spawn; i++) {
        let seed = idx_1d + i;
        let x = x_base + hash22(x_base + vec2<f32>(seed));
        let phi_goal = hash11(f32(seed)) * (bmax - bmin) + bmin;

        let particle = spawn_and_attract_particle(x, phi_goal, bmin, bmax, fdim, levelset_air, grad_levelset_air, sign);
        if is_particle_valid(particle) {
            let idx = atomicAdd(&particles_count, 1);
            particles[idx] = particle;
        }
    }
}