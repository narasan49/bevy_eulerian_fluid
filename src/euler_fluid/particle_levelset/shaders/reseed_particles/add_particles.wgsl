#import bevy_fluid::coordinate::{interp2d_center, interp2d_center_rg32float}
#import bevy_fluid::particle_levelset::particle::Particle;

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<storage, read_write> particle_count: atomic<u32>;
@group(0) @binding(2) var<storage, read> cell_particle_counts: array<u32>;
@group(0) @binding(3) var interface_band_mask: texture_storage_2d<r8uint, read>;
@group(0) @binding(4) var levelset_air: texture_storage_2d<r32float, read>;
@group(0) @binding(5) var grad_levelset_air: texture_storage_2d<rg32float, read>;

@compute @workgroup_size(8, 8, 1)
fn add_particles(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.xy;
    let dim = textureDimensions(interface_band_mask);
    let idx_1d = idx.x + dim.x * idx.y;

    let mask = textureLoad(interface_band_mask, idx).r;
    if mask == 1 {
        let num_particles_in_cell = cell_particle_counts[idx_1d];
        for (var i = num_particles_in_cell; i < 4; i++) {
            let p_idx = atomicAdd(&particle_count, 1);
            let position = vec2<f32>(idx) + rnd_vec2(vec2<f32>(idx) + vec2<f32>(i));
            let pos = attraction(levelset_air, grad_levelset_air, position);
            particles[p_idx] = Particle(pos, 0.0);
        }
    }
}

fn rnd_vec2(x: vec2<f32>) -> vec2<f32> {
    return fract(sin(x * 1000.0));
}

fn attraction(
    levelset_air: texture_storage_2d<r32float, read>,
    grad_levelset_air: texture_storage_2d<rg32float, read>,
    x: vec2<f32>,
) -> vec2<f32> {
    let phi = interp2d_center(levelset_air, x);
    let grad_phi = interp2d_center_rg32float(grad_levelset_air, x);
    let grad_phi_norm = normalize(grad_phi);

    return x - phi * grad_phi_norm;
}