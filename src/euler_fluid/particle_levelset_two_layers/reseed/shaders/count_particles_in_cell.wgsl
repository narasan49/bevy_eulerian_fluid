#import bevy_fluid::particle_levelset::particle::Particle
#import bevy_fluid::particle_levelset::constants::PARTICLE_WORKWGROUP_SIZE

@group(0) @binding(0) var<storage, read> particles: array<Particle>;
@group(0) @binding(1) var<storage, read> particles_count: u32;
@group(0) @binding(2) var<storage, read_write> num_particles_in_cell: array<atomic<u32>>;
@group(0) @binding(3) var<uniform> grid_size: vec2<u32>;

@compute @workgroup_size(PARTICLE_WORKWGROUP_SIZE)
fn count_particles_in_cell(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.x;
    if (idx >= particles_count) {
        return;
    }
    let particle_pos = particles[idx].position;
    let p_pos_u32 = vec2<u32>(particle_pos);
    let cell_idx = p_pos_u32.x + u32(grid_size.x) * p_pos_u32.y;

    atomicAdd(&num_particles_in_cell[cell_idx], 1);
}