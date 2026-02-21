#import bevy_fluid::particle_levelset::particle::Particle

@group(0) @binding(0) var<storage, read> particles: array<Particle>;
@group(0) @binding(1) var<storage, read> particle_count: u32;
@group(0) @binding(2) var<storage, read_write> cell_particle_counts: array<atomic<u32>>;
@group(0) @binding(3) var<uniform> grid_size: vec2<u32>;

@compute @workgroup_size(256, 1, 1)
fn count_particles_in_cell(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    let idx = global_invocation_id.x;
    if (idx >= particle_count) {
        return;
    }
    let particle_pos = particles[idx].position;
    let p_pos_u32 = vec2<u32>(particle_pos);
    let cell_idx = p_pos_u32.x + u32(grid_size.x) * p_pos_u32.y;

    atomicAdd(&cell_particle_counts[cell_idx], 1);
}