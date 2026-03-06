@group(0) @binding(0) var<storage, read> alive_particles_mask: array<u32>;
@group(0) @binding(1) var<storage, read> alive_particles_mask_scan: array<u32>;
@group(0) @binding(2) var<storage, read_write> particle_count: u32;

@compute @workgroup_size(1)
fn update_particles_count() {
    let last1 = arrayLength(&alive_particles_mask);
    let last2 = arrayLength(&alive_particles_mask_scan);
    particle_count = alive_particles_mask[last1 - 1] + alive_particles_mask_scan[last2 - 1];
}