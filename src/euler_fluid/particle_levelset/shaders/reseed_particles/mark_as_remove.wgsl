#import bevy_fluid::particle_levelset::particle::Particle;

@group(0) @binding(0) var<storage, read_write> sorted_particles: array<Particle>;
@group(0) @binding(1) var<storage, read_write> alive_particles_mask: array<u32>;
@group(0) @binding(2) var<storage, read> cell_particle_counts: array<u32>;
@group(0) @binding(3) var interface_band_mask: texture_storage_2d<r8uint, read>;
@group(0) @binding(4) var<storage, read> cell_offsets: array<u32>;

@compute @workgroup_size(8, 8, 1)
fn mark_as_remove(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.xy;
    let dim = textureDimensions(interface_band_mask);
    let idx_1d = idx.x + dim.x * idx.y;
    
    let mask = textureLoad(interface_band_mask, idx).r;
    let num_particles_in_cell = i32(cell_particle_counts[idx_1d]);

    for (var i = 0; i < num_particles_in_cell; i++) {
        let p_idx = cell_offsets[idx_1d] + u32(i);
        let level = sorted_particles[p_idx].level;
        if i < 4 && mask == 1 && abs(level) < 1.0{
            alive_particles_mask[p_idx] = 1;
        } else {
            alive_particles_mask[p_idx] = 0;
        }
    }
}