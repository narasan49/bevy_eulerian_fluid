#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::coordinate::{left, right, bottom, top};
#import bevy_fluid::area_fraction::area_fractions;

#ifdef REVERSE
@group(0) @binding(0) var p0: texture_storage_2d<r32float, write>;
@group(0) @binding(1) var p1: texture_storage_2d<r32float, read>;
#else
@group(0) @binding(0) var p0: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var p1: texture_storage_2d<r32float, write>;
#endif
@group(0) @binding(2) var div: texture_storage_2d<r32float, read>;
@group(0) @binding(3) var levelset_air0: texture_storage_2d<r32float, read>;
@group(0) @binding(4) var levelset_solid: texture_storage_2d<r32float, read>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(8, 8, 1)
#ifdef REVERSE
fn jacobi_iteration_reverse(
    @builtin(global_invocation_id) invocation_id: vec3<u32>
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let p = update_pressure(p1, x);

    textureStore(p0, x, vec4<f32>(p, 0.0, 0.0, 0.0));
}
#else
fn jacobi_iteration(
    @builtin(global_invocation_id) invocation_id: vec3<u32>
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let p = update_pressure(p0, x);

    textureStore(p1, x, vec4<f32>(p, 0.0, 0.0, 0.0));
}
#endif

fn update_pressure(
    p: texture_storage_2d<r32float, read>,
    x: vec2<i32>,
) -> f32 {
    let level_air_ij = textureLoad(levelset_air0, x).r;
    if (level_air_ij >= 0.0) {
        return 0.0;
    }
    let f = area_fractions(levelset_solid, x); // 0: solid, 1: non-solid
    let fully_solid = f.iminusj == 0.0 && f.iplusj == 0.0 && f.ijminus == 0.0 && f.ijplus == 0.0;

    if (fully_solid) {
        let p_ij = textureLoad(p, x).r;
        return p_ij;
    }

    let x_top = top(x);
    let x_right = right(x);
    let x_bottom = bottom(x);
    let x_left = left(x);

    let p_iminusj = textureLoad(p, x_left).r;
    let p_iplusj = textureLoad(p, x_right).r;
    let p_ijminus = textureLoad(p, x_bottom).r;
    let p_ijplus = textureLoad(p, x_top).r;

    let level_air_iminusj = textureLoad(levelset_air0, x_left).r;
    let level_air_iplusj = textureLoad(levelset_air0, x_right).r;
    let level_air_ijminus = textureLoad(levelset_air0, x_bottom).r;
    let level_air_ijplus = textureLoad(levelset_air0, x_top).r;
    let f_fluid_iminusj = min(0.0, level_air_iminusj / level_air_ij);
    let f_fluid_iplusj = min(0.0, level_air_iplusj / level_air_ij);
    let f_fluid_ijminus = min(0.0, level_air_ijminus / level_air_ij);
    let f_fluid_ijplus = min(0.0, level_air_ijplus / level_air_ij);

    let coef = f.iminusj * (1.0 - f_fluid_iminusj)
        + f.iplusj * (1.0 - f_fluid_iplusj)
        + f.ijminus * (1.0 - f_fluid_ijminus)
        + f.ijplus * (1.0 - f_fluid_ijplus);

    if (abs(coef) < 1.0e-6) {
        return 0.0;
    }
    let div_ij = textureLoad(div, x).r;
    let factor = constants.dx * constants.rho / constants.dt;

    let dp00 = f.iminusj * step(0.0, f_fluid_iminusj) * p_iminusj;
    let dp10 = f.iplusj * step(0.0, f_fluid_iplusj) * p_iplusj;
    let dp01 = f.ijminus * step(0.0, f_fluid_ijminus) * p_ijminus;
    let dp11 = f.ijplus * step(0.0, f_fluid_ijplus) * p_ijplus;

    return (dp00 + dp10 + dp01 + dp11 - factor * div_ij) / coef;
}
