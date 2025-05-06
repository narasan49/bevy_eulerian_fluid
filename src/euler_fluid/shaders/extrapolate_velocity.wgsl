#import bevy_fluid::levelset_utils::snap_to_free_surface;
#import bevy_fluid::area_fraction::{area_fraction, fluid_fraction};

@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var levelset_air0: texture_storage_2d<r32float, read_write>;
@group(1) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>;

@compute @workgroup_size(1, 64, 1)
fn extrapolate_u(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let level_minus = textureLoad(levelset_air0, idx).r;
    let level_plus = textureLoad(levelset_air0, idx + vec2<i32>(0, 1)).r;
    let fraction = fluid_fraction(level_minus, level_plus);
    if (fraction > 0.0) {
        return;
    }

    let level_solid_bottom = textureLoad(levelset_solid, idx).r;
    let level_solid_top = textureLoad(levelset_solid, idx + vec2<i32>(0, 1)).r;
    let solid_fraction = area_fraction(level_solid_bottom, level_solid_top);
    if (solid_fraction < 1.0) {
        return;
    }
    let level = calculate_level(levelset_air0, idx);

    let surface = snap_to_surface(vec2<f32>(idx) + vec2<f32>(-0.5, 0.0), level.value + 0.5, level.gradient);
    let surface_floor = vec2<i32>(i32(surface.x), i32(surface.y));
    // i = 0, 1, -1
    // j = 0, 1, -1
    for (var i = 0; i < 3; i += 1) {
        for (var j = 0; j < 3; j += 1) {
            var offset = vec2<i32>(i , j);
            if (i == 2) {
                offset = vec2<i32>(-1, j);
            } else if (j == 2) {
                offset = vec2<i32>(i, -1);
            }
            let level_air_surface_minus = textureLoad(levelset_air0, surface_floor + offset).r;
            let level_air_surface_plus = textureLoad(levelset_air0, surface_floor + offset + vec2<i32>(0, 1)).r;
            let fraction = fluid_fraction(level_air_surface_minus, level_air_surface_plus);
            if (fraction == 1.0) {
                let velocity = textureLoad(u0, surface_floor + offset).r;
                textureStore(u0, idx, vec4<f32>(velocity, 0.0, 0.0, 0.0));
                return;
            }
        }
    }
}

@compute @workgroup_size(64, 1, 1)
fn extrapolate_v(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let level_minus = textureLoad(levelset_air0, idx).r;
    let level_plus = textureLoad(levelset_air0, idx + vec2<i32>(1, 0)).r;
    let fraction = fluid_fraction(level_minus, level_plus);
    if (fraction > 0.0) {
        return;
    }
    let level_solid_left = textureLoad(levelset_solid, idx).r;
    let level_solid_right = textureLoad(levelset_solid, idx + vec2<i32>(1, 0)).r;
    let solid_fraction = area_fraction(level_solid_left, level_solid_right);
    if (solid_fraction < 1.0) {
        return;
    }
    let level = calculate_level(levelset_air0, idx);

    let surface = snap_to_surface(vec2<f32>(idx) + vec2<f32>(0.0, -0.5), level.value + 0.5, level.gradient);
    let surface_floor = vec2<i32>(i32(surface.x), i32(surface.y));
    // i = 0, 1, -1
    // j = 0, 1, -1
    for (var i = 0; i < 3; i += 1) {
        for (var j = 0; j < 3; j += 1) {
            var offset = vec2<i32>(i , j);
            if (i == 2) {
                offset = vec2<i32>(-1, j);
            } else if (j == 2) {
                offset = vec2<i32>(i, -1);
            }
            let level_air_surface_minus = textureLoad(levelset_air0, surface_floor + offset).r;
            let level_air_surface_plus = textureLoad(levelset_air0, surface_floor + offset + vec2<i32>(1, 0)).r;
            let fraction = fluid_fraction(level_air_surface_minus, level_air_surface_plus);
            if (fraction == 1.0) {
                let velocity = textureLoad(v0, surface_floor + offset).r;
                textureStore(v0, idx, vec4<f32>(velocity, 0.0, 0.0, 0.0));
                return;
            }
        }
    }
}

struct Level {
    value: f32,
    gradient: vec2<f32>,
}

fn calculate_level(
    levelset: texture_storage_2d<r32float, read_write>,
    idx: vec2<i32>,
) -> Level {
    let level_air_iminusjminus = textureLoad(levelset_air0, idx).r;
    let level_air_iminusjplus = textureLoad(levelset_air0, idx + vec2<i32>(0, 1)).r;
    let level_air_iplusjminus = textureLoad(levelset_air0, idx + vec2<i32>(1, 0)).r;
    let level_air_iplusjplus = textureLoad(levelset_air0, idx + vec2<i32>(1, 1)).r;

    var level_center = 0.25 * (level_air_iminusjminus + level_air_iplusjminus + level_air_iminusjplus + level_air_iplusjplus);
    var level_gradient = vec2<f32>(
        0.5 * (level_air_iplusjplus - level_air_iminusjplus + level_air_iplusjminus - level_air_iminusjminus),
        0.5 * (level_air_iplusjplus - level_air_iplusjminus + level_air_iminusjplus - level_air_iminusjminus),
    );
    if (!all(level_gradient == vec2<f32>(0.0))) {
        level_gradient = normalize(level_gradient);
    }

    return Level(level_center, level_gradient);
}

fn snap_to_surface(
    origin: vec2<f32>,
    level: f32,
    level_gradient: vec2<f32>,
) -> vec2<f32> {
    return origin - level * level_gradient;
}