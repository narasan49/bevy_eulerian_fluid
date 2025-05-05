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
    let level_solid = levelset_solid_grid_center(levelset_solid, x);
    var levelset_sign = 1.0;
    if (level < 0.0 && level_solid >= 0.0) {
        levelset_sign = -1.0;
    }

    var level_air = sdf * levelset_sign;
    textureStore(levelset_air0, x, vec4<f32>(level_air, 0.0, 0.0, 0.0));
}

fn levelset_solid_grid_center(
    levelset_solid: texture_storage_2d<r32float, read_write>,
    x: vec2<i32>,
) -> f32 {
    let levelset_solid_iminusjminus = textureLoad(levelset_solid, x).r;
    let levelset_solid_iplusjminus = textureLoad(levelset_solid, x + vec2<i32>(1, 0)).r;
    let levelset_solid_iminusjplus = textureLoad(levelset_solid, x + vec2<i32>(0, 1)).r;
    let levelset_solid_iplusjplus = textureLoad(levelset_solid, x + vec2<i32>(1, 1)).r;
    return 
        (levelset_solid_iminusjminus + levelset_solid_iplusjminus +
        levelset_solid_iminusjplus + levelset_solid_iplusjplus) / 4.0;
}