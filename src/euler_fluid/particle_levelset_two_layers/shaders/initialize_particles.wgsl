#import bevy_fluid::coordinate::{interp2d_center, interp2d_center_rg32float}
#import bevy_fluid::particle_levelset::particle::{Particle, is_particle_valid, spawn_and_attract_particle};
#import bevy_fluid::particle_levelset::constants::{MAX_PARTICLES_PER_CELL, BAND_WIDTH};
#import bevy_fluid::hash::{hash11, hash22};

@group(0) @binding(0) var<storage, read_write> positive_particles_count: atomic<u32>;
@group(0) @binding(1) var<storage, read_write> positive_particles: array<Particle>;
@group(0) @binding(2) var<storage, read_write> negative_particles_count: atomic<u32>;
@group(0) @binding(3) var<storage, read_write> negative_particles: array<Particle>;
@group(0) @binding(4) var levelset_air: texture_storage_2d<r32float, read>;
@group(0) @binding(5) var grad_levelset_air: texture_storage_2d<rg32float, read>;
@group(0) @binding(6) var interface_band_mask: texture_storage_2d<r8uint, read>;

// const MAX_ITER: u32 = 15;

@compute @workgroup_size(8, 8, 1)
fn initialize_particles(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.xy;
    let x_base = vec2<f32>(idx);

    let is_near_interface = textureLoad(interface_band_mask, idx).r;
    if is_near_interface == 0u {
        return;
    }

    let dim = textureDimensions(levelset_air);
    let fdim = vec2<f32>(dim);

    let bmin = 0.1;
    let bmax = BAND_WIDTH;
    let bmin_neg = -bmax;
    let bmax_neg = -bmin;

    let cell_id = idx.x + dim.x * idx.y;
    for (var i = 0u; i < MAX_PARTICLES_PER_CELL; i++) {
        let seed = cell_id + i;
        let x = x_base + hash22(x_base + vec2<f32>(seed));
        let phi_goal = hash11(f32(seed)) * (bmax - bmin) + bmin;

        let positive_particle = spawn_and_attract_particle(x, phi_goal, bmin, bmax, fdim, levelset_air, grad_levelset_air, 1.0);
        if is_particle_valid(positive_particle) {
            let idx = atomicAdd(&positive_particles_count, 1);
            positive_particles[idx] = positive_particle;
        }

        let seed_neg = cell_id + i + 1000u;
        let x_neg = x_base + hash22(x_base + vec2<f32>(seed_neg));
        let phi_goal_neg = hash11(f32(seed_neg)) * (bmax_neg - bmin_neg) + bmin_neg;

        let negative_particle = spawn_and_attract_particle(x, phi_goal_neg, bmin_neg, bmax_neg, fdim, levelset_air, grad_levelset_air, -1.0);
        if is_particle_valid(negative_particle) {
            let idx = atomicAdd(&negative_particles_count, 1);
            negative_particles[idx] = negative_particle;
        }
    }
}

// fn attract(x: vec2<f32>, lambda: f32, phi_goal: f32) -> vec2<f32> {
//     let phi = interp2d_center(levelset_air, x);
//     let grad_phi = interp2d_center_rg32float(grad_levelset_air, x);
//     let grad_phi_norm = normalize(grad_phi);

//     return x + lambda * (phi_goal - phi) * grad_phi_norm;
// }

// fn particle_radius(phi: f32) -> f32 {
//     let rmax = 0.5;
//     let rmin = 0.1;
//     return clamp(abs(phi), rmin, rmax);
// }

// fn spawn_and_attract_positive_particles(x: vec2<f32>, phi_goal: f32, bmin: f32, bmax: f32, fdim: f32) {
//     var lambda = 1.0;
//     var x_new = x;
//     var phi_new = 0.0;

//     x_new = attract(x_new, lambda, phi_goal);
//     var acceptable = false;
//     for (var j = 0u; j < MAX_ITER; j++) {
//         while (any(x_new < vec2<f32>(0.0)) || any(fdim - vec2<f32>(1.0) < x_new)) {
//             lambda *= 0.5;
//             x_new = attract(x_new, lambda, phi_goal);
//         }

//         phi_new = interp2d_center(levelset_air, x_new);
//         if bmin <= phi_new && phi_new <= bmax {
//             acceptable = true;
//             break;
//         } else {
//             lambda = 1.0;
//         }
//     }

//     if acceptable {
//         let idx = atomicAdd(&positive_particles_count, 1);
//         positive_particles[idx].position = x_new;
//         positive_particles[idx].radius = particle_radius(phi_new);
//     }
// }

// fn spawn_and_attract_negative_particles(x: vec2<f32>, phi_goal: f32, bmin: f32, bmax: f32, fdim: f32) {
//     var lambda = 1.0;
//     var x_new = x;
//     var phi_new = 0.0;

//     x_new = attract(x_new, lambda, phi_goal);
//     var acceptable = false;
//     for (var j = 0u; j < MAX_ITER; j++) {
//         while (any(x_new < vec2<f32>(0.0)) || any(fdim - vec2<f32>(1.0) < x_new)) {
//             lambda *= 0.5;
//             x_new = attract(x_new, lambda, phi_goal);
//         }

//         phi_new = interp2d_center(levelset_air, x_new);
//         if bmin <= phi_new && phi_new <= bmax {
//             acceptable = true;
//             break;
//         } else {
//             lambda = 1.0;
//         }
//     }

//     if acceptable {
//         let idx = atomicAdd(&negative_particles_count, 1);
//         negative_particles[idx].position = x_new;
//         negative_particles[idx].radius = particle_radius(phi_new);
//     }
// }