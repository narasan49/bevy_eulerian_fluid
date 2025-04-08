#import bevy_fluid::fluid_uniform::SimulationUniform;

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
    let levelset_x = levelset_at(levelset_solid, vec2<f32>(x) + vec2<f32>(0.0, 0.5));
    if (levelset_x <= 0.0) {
        textureStore(u1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        return;
    }
    let backtraced_x: vec2<f32> = runge_kutta(u0, v0, x, constants.dt);
    let dim = vec2<f32>(textureDimensions(u0));
    if (backtraced_x.x < 0.0 || backtraced_x.x > dim.x - 1.0 || backtraced_x.y < 0.0 || backtraced_x.y > dim.y - 1.0) {
        textureStore(u1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    } else {
        let level = levelset_at(levelset_solid, backtraced_x + vec2<f32>(0.0, 0.5));
        if (level < 0.0) {
            let levelset_index = vec2<i32>(backtraced_x + vec2<f32>(0.0, 0.5));
            let corrected_backtraced_x = project_onto_surface(levelset_solid, backtraced_x, level, levelset_index);

            let backtraced_u: f32 = u_at(u0, corrected_backtraced_x);
            if (backtraced_x.x < 0.0 || backtraced_x.x > dim.x - 1.0 || backtraced_x.y < 0.0 || backtraced_x.y > dim.y - 1.0) {
                textureStore(u1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
            } else {
                textureStore(u1, x, vec4<f32>(backtraced_u, 0.0, 0.0, 0.0));
            }
        } else {
            let backtraced_u: f32 = u_at(u0, backtraced_x);
            textureStore(u1, x, vec4<f32>(backtraced_u, 0.0, 0.0, 0.0));
        }
    }
}

@compute @workgroup_size(64, 1, 1)
fn advect_v(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let levelset_x = levelset_at(levelset_solid, vec2<f32>(x) + vec2<f32>(0.5, 0.0));
    if (levelset_x <= 0.0) {
        textureStore(v1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        return;
    }
    let backtraced_x: vec2<f32> = runge_kutta(u0, v0, x, constants.dt);
    let dim = vec2<f32>(textureDimensions(v0));
    if (backtraced_x.x < 0.0 || backtraced_x.x > dim.x - 1.0 || backtraced_x.y < 0.0 || backtraced_x.y > dim.y - 1.0) {
        textureStore(v1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    } else {
        let level = levelset_at(levelset_solid, backtraced_x + vec2<f32>(0.5, 0.0));
        if (level < 0.0) {
            let levelset_index = vec2<i32>(backtraced_x + vec2<f32>(0.5, 0.0));
            let corrected_backtraced_x = project_onto_surface(levelset_solid, backtraced_x, level, levelset_index);

            let backtraced_v: f32 = v_at(v0, corrected_backtraced_x);
            if (backtraced_x.x < 0.0 || backtraced_x.x > dim.x - 1.0 || backtraced_x.y < 0.0 || backtraced_x.y > dim.y - 1.0) {
                textureStore(v1, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
            } else {
                textureStore(v1, x, vec4<f32>(backtraced_v, 0.0, 0.0, 0.0));
            }
        } else {
            let backtraced_v: f32 = v_at(v0, backtraced_x);
            textureStore(v1, x, vec4<f32>(backtraced_v, 0.0, 0.0, 0.0));
        }
    }
}

fn runge_kutta(
    u: texture_storage_2d<r32float, read_write>,
    v: texture_storage_2d<r32float, read_write>,
    x: vec2<i32>,
    dt: f32,
) -> vec2<f32> {
    let velocity = vec2<f32>(u_at(u, vec2<f32>(x)), v_at(v, vec2<f32>(x)));
    let x_mid = vec2<f32>(x) - vec2<f32>(0.5 * dt) * velocity;
    let velocity_mid = vec2<f32>(u_at(u, x_mid), v_at(v, x_mid));

    return vec2<f32>(x) - dt * velocity_mid;
}

fn u_at(
    u: texture_storage_2d<r32float, read_write>,
    x: vec2<f32>,
) -> f32 {
    let i = i32(round(x.x));
    let j = i32(floor(x.y));
    let fract_i = x.x + 0.5 - f32(i);
    let fract_j = x.y - f32(j);
    let u00 = textureLoad(u, vec2<i32>(i, j)).r;
    let u10 = textureLoad(u, vec2<i32>(i + 1, j)).r;
    let u01 = textureLoad(u, vec2<i32>(i, j + 1)).r;
    let u11 = textureLoad(u, vec2<i32>(i + 1, j + 1)).r;

    return mix(mix(u00, u10, fract_i), mix(u01, u11, fract_i), fract_j);
}

fn v_at(
    v: texture_storage_2d<r32float, read_write>,
    x: vec2<f32>,
) -> f32 {
    let i = i32(floor(x.x));
    let j = i32(round(x.y));
    let fract_i = x.x - f32(i);
    let fract_j = x.y + 0.5 - f32(j);
    let v00 = textureLoad(v, vec2<i32>(i, j)).r;
    let v10 = textureLoad(v, vec2<i32>(i + 1, j)).r;
    let v01 = textureLoad(v, vec2<i32>(i, j + 1)).r;
    let v11 = textureLoad(v, vec2<i32>(i + 1, j + 1)).r;

    return mix(mix(v00, v10, fract_i), mix(v01, v11, fract_i), fract_j);
}

fn levelset_at(
    levelset: texture_storage_2d<r32float, read_write>,
    x: vec2<f32>,
) -> f32 {
    let i = i32(floor(x.x));
    let j = i32(floor(x.y));
    let fract_i = x.x - f32(i);
    let fract_j = x.y - f32(j);
    let levelset00 = textureLoad(levelset, vec2<i32>(i, j)).r;
    let levelset10 = textureLoad(levelset, vec2<i32>(i + 1, j)).r;
    let levelset01 = textureLoad(levelset, vec2<i32>(i, j + 1)).r;
    let levelset11 = textureLoad(levelset, vec2<i32>(i + 1, j + 1)).r;

    return mix(mix(levelset00, levelset10, fract_i), mix(levelset01, levelset11, fract_i), fract_j);
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