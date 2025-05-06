#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::coordinate::{left, right, bottom, top};
#import bevy_fluid::area_fraction::area_fractions;

@group(0) @binding(0) var<uniform> constants: SimulationUniform;

@group(1) @binding(0) var p0: texture_storage_2d<r32float, read_write>;
@group(1) @binding(1) var p1: texture_storage_2d<r32float, read_write>;

@group(2) @binding(0) var div: texture_storage_2d<r32float, read_write>;

@group(3) @binding(0) var levelset_air0: texture_storage_2d<r32float, read_write>;
@group(3) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>;

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
    let f = area_fractions(levelset_solid, x); // 0: solid, 1: non-solid
    let f_fluid = area_fractions(levelset_air0, x); // 0: fluid, 1: non-fluid

    let fully_solid = f.iminusj == 0.0 && f.iplusj == 0.0 && f.ijminus == 0.0 && f.ijplus == 0.0;
    let fully_non_fluid = f_fluid.iminusj == 1.0 && f_fluid.iplusj == 1.0 && f_fluid.ijminus == 1.0 && f_fluid.ijplus == 1.0;

    if (fully_solid || fully_non_fluid) {
        return 0.0;
    }

    let x_top = top(x);
    let x_right = right(x);
    let x_bottom = bottom(x);
    let x_left = left(x);

    let p_iminusj = textureLoad(p, x_left).r;
    let p_iplusj = textureLoad(p, x_right).r;
    let p_ijminus = textureLoad(p, x_bottom).r;
    let p_ijplus = textureLoad(p, x_top).r;

    let coef = f.iminusj * (1.0 - f_fluid.iminusj)
        + f.iplusj * (1.0 - f_fluid.iplusj)
        + f.ijminus * (1.0 - f_fluid.ijminus)
        + f.ijplus * (1.0 - f_fluid.ijplus);
    
    if (abs(coef) < 1.0e-6) {
        return 0.0;
    }
    let div_ij = textureLoad(div, x).r;
    let factor = constants.dx * constants.rho / constants.dt;
    
    let dp00 = f.iminusj * (1.0 - f_fluid.iminusj) * p_iminusj;
    let dp10 = f.iplusj * (1.0 - f_fluid.iplusj) * p_iplusj;
    let dp01 = f.ijminus * (1.0 - f_fluid.ijminus) * p_ijminus;
    let dp11 = f.ijplus * (1.0 - f_fluid.ijplus) * p_ijplus;
    
    return (dp00 + dp10 + dp01 + dp11 - factor * div_ij) / coef;
}