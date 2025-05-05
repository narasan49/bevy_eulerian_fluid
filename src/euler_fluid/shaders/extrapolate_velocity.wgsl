#import bevy_fluid::levelset_utils::snap_to_free_surface;

@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var levelset_air0: texture_storage_2d<r32float, read_write>;

@compute @workgroup_size(1, 64, 1)
fn extrapolate_u(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let level_air_minus = textureLoad(levelset_air0, idx - vec2<i32>(1, 0)).r;
    let level_air_plus = textureLoad(levelset_air0, idx).r;
    let level_air = 0.5 * (level_air_minus + level_air_plus);
    if (level_air < 0.0) {
        return;
    }
    let level_air_iminusjminus = textureLoad(levelset_air0, idx + vec2<i32>(-1, -1)).r;
    let level_air_iminusjplus = textureLoad(levelset_air0, idx + vec2<i32>(-1, 1)).r;
    let level_air_ijminus = textureLoad(levelset_air0, idx + vec2<i32>(0, -1)).r;
    let level_air_ijplus = textureLoad(levelset_air0, idx + vec2<i32>(0, 1)).r;
    var level_gradient = vec2<f32>(
        0.25 * (level_air_ijminus - level_air_iminusjminus + level_air_ijplus - level_air_iminusjplus),
        level_air_ijplus - level_air_ijminus,
    );
    if (!all(level_gradient == vec2<f32>(0.0))) {
        level_gradient = normalize(level_gradient);
    }

    let surface = vec2<f32>(idx) - (level_air + 0.5) * level_gradient;
    let surface_floor = vec2<i32>(i32(surface.x), i32(surface.y));
    let search_half_width = 2;
    for (var i = -search_half_width; i < search_half_width+1; i += 1) {
        for (var j = -search_half_width; j < search_half_width+1; j += 1) {
            let offset = vec2<i32>(i, j);
            let level_air_surface_minus = textureLoad(levelset_air0, surface_floor + offset - vec2<i32>(1, 0)).r;
            let level_air_surface_plus = textureLoad(levelset_air0, surface_floor + offset).r;
            let level_air_surface = 0.5 * (level_air_surface_minus + level_air_surface_plus);
            if (level_air_surface < 0.0) {
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
    let level_air_minus = textureLoad(levelset_air0, idx - vec2<i32>(0, 1)).r;
    let level_air_plus = textureLoad(levelset_air0, idx).r;
    let level_air = 0.5 * (level_air_minus + level_air_plus);
    if (level_air < 0.0) {
        return;
    }
    let level_air_iminusjminus = textureLoad(levelset_air0, idx + vec2<i32>(-1, -1)).r;
    let level_air_iminusj = textureLoad(levelset_air0, idx + vec2<i32>(-1, 0)).r;
    let level_air_iplusjminus = textureLoad(levelset_air0, idx + vec2<i32>(1, -1)).r;
    let level_air_iplusj = textureLoad(levelset_air0, idx + vec2<i32>(1, 0)).r;
    var level_gradient = vec2<f32>(
        0.25 * (level_air_iplusjminus - level_air_iminusjminus + level_air_iplusj - level_air_iminusj),
        level_air_plus - level_air_minus,
    );
    if (!all(level_gradient == vec2<f32>(0.0))) {
        level_gradient = normalize(level_gradient);
    }

    let surface = vec2<f32>(idx) - (level_air + 0.5) * level_gradient;
    let surface_floor = vec2<i32>(i32(surface.x), i32(surface.y));
    let search_half_width = 2;
    for (var i = -search_half_width; i < search_half_width+1; i += 1) {
        for (var j = -search_half_width; j < search_half_width+1; j += 1) {
            let offset = vec2<i32>(i, j);
            let level_air_surface_minus = textureLoad(levelset_air0, surface_floor + offset - vec2<i32>(0, 1)).r;
            let level_air_surface_plus = textureLoad(levelset_air0, surface_floor + offset).r;
            let level_air_surface = 0.5 * (level_air_surface_minus + level_air_surface_plus);
            if (level_air_surface < 0.0) {
                let velocity = textureLoad(v0, surface_floor + offset).r;
                textureStore(v0, idx, vec4<f32>(velocity, 0.0, 0.0, 0.0));
                return;
            }
        }
    }
}