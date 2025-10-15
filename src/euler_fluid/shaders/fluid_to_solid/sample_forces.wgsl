#import bevy_fluid::area_fraction::area_fractions;
#import bevy_fluid::fluid_to_solid::fixed_point_conversion::{f32_to_i32};

const MAX_SOLIDS: u32 = 256;

@group(0) @binding(0) var<storage, read_write> bins_x: array<atomic<i32>>;
@group(0) @binding(1) var<storage, read_write> bins_y: array<atomic<i32>>;

@group(1) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>;

@group(2) @binding(1) var p1: texture_storage_2d<r32float, read_write>;

@group(3) @binding(2) var solid_id: texture_storage_2d<r32sint, read_write>;

// @compute @workgroup_size(MAX_SOLIDS, 8, 1)
@compute @workgroup_size(8, 8, 1)
fn sample_forces_to_solid(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(global_invocation_id.xy);

    let f = area_fractions(levelset_solid, idx);
    if (f.iminusj == 0.0 && f.iplusj == 0.0 && f.ijminus == 0.0 && f.ijplus == 0.0) {
        // Fully solid
        return;
    }
    
    let solid_id = textureLoad(solid_id, idx).r;
    if (solid_id == -1) {
        return;
    }

    let p = textureLoad(p1, idx).r;
    let force = -vec2<f32>(f.iplusj - f.iminusj, f.ijplus - f.ijminus) * p;
    atomicAdd(&bins_x[solid_id], f32_to_i32(force.x));
    atomicAdd(&bins_y[solid_id], f32_to_i32(force.y));

}