#import bevy_fluid::area_fraction::area_fractions;
#import bevy_fluid::fluid_to_solid::fixed_point_conversion::{f32_to_i32};
#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::solid_obstacle::{SolidObstacle, center_of_mass};

const MAX_SOLIDS: u32 = 256;

@group(0) @binding(0) var<storage, read_write> bins_force_x: array<atomic<i32>>;
@group(0) @binding(1) var<storage, read_write> bins_force_y: array<atomic<i32>>;
@group(0) @binding(2) var<storage, read_write> bins_torque: array<atomic<i32>>;
@group(0) @binding(3) var levelset_solid: texture_storage_2d<r32float, read_write>;
@group(0) @binding(4) var solid_id: texture_storage_2d<r32sint, read>;
@group(0) @binding(5) var p1: texture_storage_2d<r32float, read>;

@group(1) @binding(0) var<storage, read> obstacles: array<SolidObstacle>;

@group(2) @binding(0) var<uniform> simulation_uniform: SimulationUniform;

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

    let obstacle = obstacles[solid_id];
    let center = center_of_mass(obstacle);
    // idx to world position
    let uv = vec2<f32>(idx) / simulation_uniform.size;
    let xy = vec2<f32>(uv.x - 0.5, -uv.y + 0.5) * simulation_uniform.size;
    let x = (simulation_uniform.fluid_transform * vec4<f32>(xy, 0.0, 1.0)).xy;

    let r = x - center;
    let torque = r.x * force.y - r.y * force.x;

    atomicAdd(&bins_force_x[solid_id], f32_to_i32(force.x));
    atomicAdd(&bins_force_y[solid_id], f32_to_i32(force.y));
    atomicAdd(&bins_torque[solid_id], f32_to_i32(torque));
}