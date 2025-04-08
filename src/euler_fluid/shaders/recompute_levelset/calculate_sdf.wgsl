@group(0) @binding(0) var levelset_air0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var levelset_air1: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var levelset_solid: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var seeds_x: texture_storage_2d<r32float, read_write>;
@group(1) @binding(1) var seeds_y: texture_storage_2d<r32float, read_write>;

fn get_seed(x: vec2<i32>) -> vec2<f32> {
    return vec2<f32>(textureLoad(seeds_x, x).r, textureLoad(seeds_y, x).r);
}

@compute @workgroup_size(8, 8, 1)
fn calculate_sdf(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let sdf = distance(get_seed(x), vec2<f32>(x));
    let level = textureLoad(levelset_air1, x).r;
    var levelset_sign = 1.0;
    if (level < 0.0) {
        levelset_sign = -1.0;
    }

    var level_air = sdf * levelset_sign;

    let level_solid = textureLoad(levelset_solid, x).r;
    // levelset_solid < 0.0 -> solid
    // levelset_solid >= 0.0 -> air or fluid
    // levelset_air < 0.0 -> fluid or solid
    // levelset_air >= 0.0 -> air
    if (level_solid < 0.0) {
        if (level_air < 0.0) {
            level_air = max(level_air, level_solid);
        } else {
            level_air = level_solid;
        }
    } else {
        if (level_air < 0.0) {
            // noop
        } else {
            level_air = min(level_air, level_solid);
        }
    }
    textureStore(levelset_air0, x, vec4<f32>(level_air, 0.0, 0.0, 0.0));
}