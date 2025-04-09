#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::coordinate::{interp2d_center, interp2d_edge_x, interp2d_edge_y, runge_kutta};

@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var levelset_air0: texture_storage_2d<r32float, read_write>;
@group(1) @binding(1) var levelset_air1: texture_storage_2d<r32float, read_write>;

@group(2) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(8, 8, 1)
fn advect_levelset(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let dt = constants.dt;
    let size = textureDimensions(levelset_air0);
    var x_new = runge_kutta(u0, v0, vec2<f32>(x), dt);
    if (x_new.x > f32(size.x) - 1.0) {
        x_new.x = f32(size.x) - 1.0;
    }
    if (x_new.y > f32(size.y) - 1.0) {
        x_new.y = f32(size.y) - 1.0;
    }
    if (x_new.x < 0.0) {
        x_new.x = 0.0;
    }
    if (x_new.y < 0.0) {
        x_new.y = 0.0;
    }
    let new_level = interp2d_center(levelset_air0, x_new);
    textureStore(levelset_air1, x, vec4<f32>(new_level, 0.0, 0.0, 0.0));
}