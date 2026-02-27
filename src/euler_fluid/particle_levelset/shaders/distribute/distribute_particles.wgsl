#import bevy_fluid::particle_levelset::particle::Particle;

@group(0) @binding(0) var<storage, read> cell_offsets: array<u32>;
@group(0) @binding(1) var<storage, read> sorted_particles: array<Particle>;
@group(0) @binding(2) var<storage, read_write> levelset_correction: array<atomic<i32>>;
@group(0) @binding(3) var<storage, read_write> weight: array<atomic<i32>>;
@group(0) @binding(4) var<uniform> grid_size: vec2<u32>;

@compute @workgroup_size(8, 8, 1)
fn distribute_particles(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let cell_idx = global_invocation_id.xy;
    let idx = cell_idx.x + grid_size.x * cell_idx.y;

    for (var i = cell_offsets[idx]; i < cell_offsets[idx + 1]; i++) {
        let p = sorted_particles[i];
        distribute_particle(p, idx);
    }
}

fn distribute_particle(p: Particle, cell_idx: u32) {
    let a = fract(p.position);
    let d_phi_ij = (1.0 - a.x) * (1.0 - a.y);
    let d_phi_iplus_j = a.x * (1.0 - a.y);
    let d_phi_ijplus = (1.0 - a.x) * a.y;
    let d_phi_iplus_j_plus = a.x * a.y;
    
    atomicAdd(&levelset_correction[cell_idx], f32_to_i32(d_phi_ij * p.level));
    atomicAdd(&weight[cell_idx], f32_to_i32(d_phi_ij));
    atomicAdd(&levelset_correction[cell_idx + 1], f32_to_i32(d_phi_iplus_j * p.level));
    atomicAdd(&weight[cell_idx + 1], f32_to_i32(d_phi_iplus_j));
    atomicAdd(&levelset_correction[cell_idx + grid_size.x], f32_to_i32(d_phi_ijplus * p.level));
    atomicAdd(&weight[cell_idx + grid_size.x], f32_to_i32(d_phi_ijplus));
    atomicAdd(&levelset_correction[cell_idx + grid_size.x + 1], f32_to_i32(d_phi_iplus_j_plus * p.level));
    atomicAdd(&weight[cell_idx + grid_size.x + 1], f32_to_i32(d_phi_iplus_j_plus));
}

const SCALE = 1000.0;
fn i32_to_f32(value: i32) -> f32 {
    return f32(value) / SCALE;
}

fn f32_to_i32(value: f32) -> i32 {
    return i32(value * SCALE);
}