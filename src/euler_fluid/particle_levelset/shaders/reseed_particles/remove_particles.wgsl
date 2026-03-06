#import bevy_fluid::particle_levelset::particle::Particle;

@group(0) @binding(0) var<storage, read> alive_particles_mask: array<u32>;
@group(0) @binding(1) var<storage, read> alive_particles_mask_scan: array<u32>;
@group(0) @binding(2) var<storage, read> sorted_particles: array<Particle>;
@group(0) @binding(3) var<storage, read_write> particles: array<Particle>;

@compute @workgroup_size(256, 1, 1)
fn remove_particles(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let i = global_invocation_id.x;
    
    if alive_particles_mask[i] == 1 {
        particles[alive_particles_mask_scan[i]] = sorted_particles[i];
    }
}