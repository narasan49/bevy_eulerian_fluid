#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::coordinate::{interp2d_center, interp2d_edge_x, interp2d_edge_y, runge_kutta};

@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var u1: texture_storage_2d<r32float, read_write>;
@group(0) @binding(3) var v1: texture_storage_2d<r32float, read_write>;

@group(1) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>;

@group(2) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(1, 64, 1)
fn advect_u(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let levelset_x = interp2d_center(levelset_solid, vec2<f32>(x) + vec2<f32>(0.0, 0.5));
    if (levelset_x <= 0.0) {
        textureStore(u1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        return;
    }
    let backtraced_x: vec2<f32> = runge_kutta(u0, v0, vec2<f32>(x), constants.dt);
    let dim = vec2<f32>(textureDimensions(u0));
    if (backtraced_x.x < 0.0 || backtraced_x.x > dim.x - 1.0 || backtraced_x.y < 0.0 || backtraced_x.y > dim.y - 1.0) {
        textureStore(u1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    } else {
        let level = interp2d_center(levelset_solid, backtraced_x + vec2<f32>(0.0, 0.5));
        if (level < 0.0) {
            let levelset_index = vec2<i32>(backtraced_x + vec2<f32>(0.0, 0.5));
            let corrected_backtraced_x = project_onto_surface(levelset_solid, backtraced_x, level, levelset_index);

            let backtraced_u: f32 = interp2d_edge_x(u0, corrected_backtraced_x);
            if (backtraced_x.x < 0.0 || backtraced_x.x > dim.x - 1.0 || backtraced_x.y < 0.0 || backtraced_x.y > dim.y - 1.0) {
                textureStore(u1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
            } else {
                textureStore(u1, x, vec4<f32>(backtraced_u, 0.0, 0.0, 0.0));
            }
        } else {
            let backtraced_u: f32 = interp2d_edge_x(u0, backtraced_x);
            textureStore(u1, x, vec4<f32>(backtraced_u, 0.0, 0.0, 0.0));
        }
    }
}

@compute @workgroup_size(64, 1, 1)
fn advect_v(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let levelset_x = interp2d_center(levelset_solid, vec2<f32>(x) + vec2<f32>(0.5, 0.0));
    if (levelset_x <= 0.0) {
        textureStore(v1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        return;
    }
    let backtraced_x: vec2<f32> = runge_kutta(u0, v0, vec2<f32>(x), constants.dt);
    let dim = vec2<f32>(textureDimensions(v0));
    if (backtraced_x.x < 0.0 || backtraced_x.x > dim.x - 1.0 || backtraced_x.y < 0.0 || backtraced_x.y > dim.y - 1.0) {
        textureStore(v1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    } else {
        let level = interp2d_center(levelset_solid, backtraced_x + vec2<f32>(0.5, 0.0));
        if (level < 0.0) {
            let levelset_index = vec2<i32>(backtraced_x + vec2<f32>(0.5, 0.0));
            let corrected_backtraced_x = project_onto_surface(levelset_solid, backtraced_x, level, levelset_index);

            let backtraced_v: f32 = interp2d_edge_y(v0, corrected_backtraced_x);
            if (backtraced_x.x < 0.0 || backtraced_x.x > dim.x - 1.0 || backtraced_x.y < 0.0 || backtraced_x.y > dim.y - 1.0) {
                textureStore(v1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
            } else {
                textureStore(v1, x, vec4<f32>(backtraced_v, 0.0, 0.0, 0.0));
            }
        } else {
            let backtraced_v: f32 = interp2d_edge_y(v0, backtraced_x);
            textureStore(v1, x, vec4<f32>(backtraced_v, 0.0, 0.0, 0.0));
        }
    }
}

fn project_onto_surface(
    levelset_solid: texture_storage_2d<r32float, read_write>,
    backtraced_x: vec2<f32>,
    level: f32,
    levelset_index: vec2<i32>,
) -> vec2<f32> {
    let levelset_ij = textureLoad(levelset_solid, levelset_index).r;
    let levelset_iplusj = textureLoad(levelset_solid, levelset_index + vec2<i32>(1, 0)).r;
    let levelset_ijplus = textureLoad(levelset_solid, levelset_index + vec2<i32>(0, 1)).r;
    let levelset_iplusjplus = textureLoad(levelset_solid, levelset_index + vec2<i32>(1, 1)).r;
    
    var level_gradient = vec2<f32>(
        0.5 * (levelset_iplusj - levelset_ij + levelset_iplusjplus - levelset_ijplus),
        0.5 * (levelset_ijplus - levelset_ij + levelset_iplusjplus - levelset_iplusj),
    );
    if (level_gradient.x != 0.0 || level_gradient.y != 0.0) {
        level_gradient = normalize(level_gradient);
    }
    let corrected_backtraced_x = round(backtraced_x + level * level_gradient);
    return corrected_backtraced_x;
}