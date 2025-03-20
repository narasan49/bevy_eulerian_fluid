#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::coordinate::{left, right, bottom, top};
#import bevy_fluid::area_fraction::area_fractions;

@group(0) @binding(0) var<uniform> constants: SimulationUniform;

@group(1) @binding(0) var p0: texture_storage_2d<r32float, read_write>;
@group(1) @binding(1) var p1: texture_storage_2d<r32float, read_write>;

@group(2) @binding(0) var div: texture_storage_2d<r32float, read_write>;

@group(3) @binding(1) var levelset_air: texture_storage_2d<r32float, read_write>; // nx, ny
@group(3) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>; // nx + 1, ny + 1

@compute @workgroup_size(8, 8, 1)
fn jacobi_iteration(
    @builtin(global_invocation_id) invocation_id: vec3<u32>
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let p = update_pressure(p0, x);
    
    textureStore(p1, x, vec4<f32>(p, 0.0, 0.0, 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn jacobi_iteration_reverse(
    @builtin(global_invocation_id) invocation_id: vec3<u32>
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let p = update_pressure(p1, x);

    textureStore(p0, x, vec4<f32>(p, 0.0, 0.0, 0.0));
}

fn update_pressure(
    p: texture_storage_2d<r32float, read_write>,
    x: vec2<i32>,
) -> f32 {
    let levelset_air_ij = textureLoad(levelset_air, x).r;
    if (levelset_air_ij >= 0.0) {
        return 0.0;
    }
    
    let x_top = top(x);
    let x_right = right(x);
    let x_bottom = bottom(x);
    let x_left = left(x);

    let f = area_fractions(levelset_solid, x);

    let p_iminusj = textureLoad(p, x_left).r;
    let p_iplusj = textureLoad(p, x_right).r;
    let p_ijminus = textureLoad(p, x_bottom).r;
    let p_ijplus = textureLoad(p, x_top).r;

    let levelset_air_iminusj = textureLoad(levelset_air, x_left).r;
    let levelset_air_iplusj = textureLoad(levelset_air, x_right).r;
    let levelset_air_ijminus = textureLoad(levelset_air, x_bottom).r;
    let levelset_air_ijplus = textureLoad(levelset_air, x_top).r;

    let coef = f.iminusj * (1.0 - max(0.0, levelset_air_iminusj) / levelset_air_ij) 
        + f.iplusj * (1.0 - max(0.0, levelset_air_iplusj) / levelset_air_ij)
        + f.ijminus * (1.0 - max(0.0, levelset_air_ijminus) / levelset_air_ij)
        + f.ijplus * (1.0 - max(0.0, levelset_air_ijplus) / levelset_air_ij);
    
    if (abs(coef) < 1.0e-6) {
        return 0.0;
    }
    let div_ij = textureLoad(div, x).r;
    let factor = constants.dx * constants.rho / constants.dt;
    
    let dp00 = step(0.0, levelset_air_iminusj / levelset_air_ij) * f.iminusj * p_iminusj;
    let dp10 = step(0.0, levelset_air_iplusj / levelset_air_ij) * f.iplusj * p_iplusj;
    let dp01 = step(0.0, levelset_air_ijminus / levelset_air_ij) * f.ijminus * p_ijminus;
    let dp11 = step(0.0, levelset_air_ijplus / levelset_air_ij) * f.ijplus * p_ijplus;
    
    return (dp00 + dp10 + dp01 + dp11 - factor * div_ij) / coef;
}