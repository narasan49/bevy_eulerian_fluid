#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::area_fraction::area_fraction;

// The number of texture_storage binding for WebGPU is limited to 8.
// So solve_velocity_u and solve_velocity_v have different bindings for u0, u1 and v0, v1.
@group(0) @binding(0) var u0: texture_storage_2d<r32float, write>;
@group(0) @binding(1) var u1: texture_storage_2d<r32float, read>;
@group(0) @binding(2) var u_solid: texture_storage_2d<r32float, read>;
@group(0) @binding(3) var p0: texture_storage_2d<r32float, read>;
@group(0) @binding(4) var levelset_air0: texture_storage_2d<r32float, read>;
@group(0) @binding(5) var area_fraction_solid: texture_storage_2d<rgba32float, read>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(1, 64, 1)
fn solve_velocity_u(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let factor = constants.dt / (constants.dx * constants.rho);
    let x = vec2<i32>(invocation_id.xy);
//  if (any(x == vec2<i32>(0)) || any(x == vec2<i32>(textureDimensions(u0)) - 1)) {
//      textureStore(u0, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
//      return;
//  }

    let fraction = textureLoad(area_fraction_solid, x).x;
    if (fraction == 0.0) {
        textureStore(u0, x, textureLoad(u_solid, x));
        return;
    }

    var p_ij = textureLoad(p0, x).r;
    var p_iminusj = textureLoad(p0, x - vec2<i32>(1, 0)).r;

    let level_plus = textureLoad(levelset_air0, x).r;
    let level_minus = textureLoad(levelset_air0, x - vec2<i32>(1, 0)).r;
    if (level_minus >= 0.0 && level_plus < 0.0) {
        p_iminusj = level_minus / level_plus * p_ij;
    } else if (level_minus < 0.0 && level_plus >= 0.0) {
        p_ij = level_plus / level_minus * p_iminusj;
    } else if (level_minus >= 0.0 && level_plus >= 0.0) {
        textureStore(u0, x, vec4f(0.0));
        return;
    }

    let u = textureLoad(u1, x);
    let du = vec4<f32>(factor * (p_ij - p_iminusj), 0.0, 0.0, 0.0);
    textureStore(u0, x, u - du);
}
