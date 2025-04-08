#import bevy_fluid::fluid_uniform::SimulationUniform;

@group(0) @binding(0) var levelset: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var grid_label: texture_storage_2d<r32uint, read_write>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@compute
@workgroup_size(8, 8, 1)
fn initialize_grid_center(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
    textureStore(grid_label, x, vec4<u32>(1, 0, 0, 0));
    let dim = textureDimensions(levelset);

    let zero_contour_height  = f32(dim.y) - constants.initial_fluid_level * f32(dim.y);
    let value = zero_contour_height - f32(global_id.y);
    textureStore(levelset, x, vec4<f32>(value, 0.0, 0.0, 0.0));
}