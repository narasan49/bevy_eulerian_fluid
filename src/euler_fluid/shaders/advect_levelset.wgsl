#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::coordinate::{interp2d_center, runge_kutta};

@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var levelset_air0: texture_storage_2d<r32float, read_write>;
@group(1) @binding(1) var levelset_air1: texture_storage_2d<r32float, read_write>;

@group(2) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(8, 8, 1)
fn advect_levelset(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(invocation_id.xy);
    let x = vec2<f32>(idx);

    let dt = constants.dt;
    let size = textureDimensions(levelset_air0);
    var x_new = runge_kutta(u0, v0, x, dt);
    x_new = clamp(x_new, vec2<f32>(0.0), vec2<f32>(size) - vec2<f32>(1.0));

    let new_level = interp2d_center(levelset_air0, x_new);
    textureStore(levelset_air1, idx, vec4<f32>(new_level, 0.0, 0.0, 0.0));
}
