#define_import_path bevy_fluid::levelset_utils

fn project_onto_surface(
    levelset_solid: texture_storage_2d<r32float, read>,
    x: vec2<f32>,
    levelset_index: vec2<i32>,
    levelset_offset: f32,
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
    } else {
        level_gradient = vec2<f32>(0.0, 0.0);
    }
    let level = (levelset_ij + levelset_iplusj + levelset_ijplus + levelset_iplusjplus) / 4.0;
    let x_surface = round(x - (level + levelset_offset) * level_gradient);
    return x_surface;
}

fn snap_to_free_surface(
    levelset_air: texture_storage_2d<r32float, read_write>,
    x: vec2<i32>,
) -> vec2<f32> {
    let levelset_air_iminusj = textureLoad(levelset_air, x - vec2<i32>(1, 0)).r;
    let levelset_air_iplusj = textureLoad(levelset_air, x + vec2<i32>(1, 0)).r;
    let levelset_air_ijminus = textureLoad(levelset_air, x - vec2<i32>(0, 1)).r;
    let levelset_air_ijplus = textureLoad(levelset_air, x + vec2<i32>(0, 1)).r;

    var level_gradient = vec2<f32>(
        0.5 * (levelset_air_iplusj - levelset_air_iminusj),
        0.5 * (levelset_air_ijplus - levelset_air_ijminus),
    );
    if (level_gradient.x != 0.0 || level_gradient.y != 0.0) {
        level_gradient = normalize(level_gradient);
    } else {
        level_gradient = vec2<f32>(0.0, 0.0);
    }
    let level = textureLoad(levelset_air, x).r;
    let x_surface = round(vec2<f32>(x) - level * level_gradient);
    return x_surface;
}
