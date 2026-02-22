#import bevy_fluid::particle_levelset::particle::Particle

@group(0) @binding(0) var<storage, read> particles: array<Particle>;
@group(0) @binding(1) var<storage, read> particle_count: u32;
@group(0) @binding(2) var<storage, read> cell_offsets: array<u32>;
@group(0) @binding(3) var<storage, read_write> sorted_particles: array<Particle>;
@group(0) @binding(4) var<uniform> grid_size: vec2<u32>;
@group(0) @binding(5) var<storage, read_write> cell_cursor: array<atomic<u32>>;


@compute @workgroup_size(256, 1, 1)
fn sort_particles(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.x;
    if (idx >= particle_count) {
        return;
    }
    let p = particles[idx];
    let p_cell_location = vec2<u32>(p.position);
    let cell_idx = p_cell_location.x + u32(grid_size.x) * p_cell_location.y;

    let cursor = atomicAdd(&cell_cursor[cell_idx], 1);

    sorted_particles[cell_offsets[cell_idx] + cursor] = p;
}