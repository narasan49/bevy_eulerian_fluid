#define_import_path bevy_fluid::area_fraction

struct AreaFractions {
    iminusj: f32,
    iplusj: f32,
    ijminus: f32,
    ijplus: f32,
}

fn area_fraction(level0: f32, level1: f32) -> f32 {
    return clamp(max(level0, level1) / abs(level0 - level1), 0.0, 1.0);
}

fn area_fractions(
    levelset_solid: texture_storage_2d<r32float, read_write>,
    idx: vec2<i32>,
) -> AreaFractions {
    let offsets = array<vec2<i32>, 9>(
        vec2<i32>(-1, -1),
        vec2<i32>(0, -1),
        vec2<i32>(1, -1),
        vec2<i32>(-1, 0),
        vec2<i32>(0, 0),
        vec2<i32>(1, 0),
        vec2<i32>(-1, 1),
        vec2<i32>(0, 1),
        vec2<i32>(1, 1),
    );
    let level_centers = array<f32, 9>(
        textureLoad(levelset_solid, idx + offsets[0]).r,
        textureLoad(levelset_solid, idx + offsets[1]).r,
        textureLoad(levelset_solid, idx + offsets[2]).r,
        textureLoad(levelset_solid, idx + offsets[3]).r,
        textureLoad(levelset_solid, idx + offsets[4]).r,
        textureLoad(levelset_solid, idx + offsets[5]).r,
        textureLoad(levelset_solid, idx + offsets[6]).r,
        textureLoad(levelset_solid, idx + offsets[7]).r,
        textureLoad(levelset_solid, idx + offsets[8]).r,
    );

    // levelset[i-0.5, j-0.5]
    let level_vertex_iminusjminus = 0.25 * (level_centers[0] + level_centers[1] + level_centers[3] + level_centers[4]);
    // levelset[i+0.5, j-0.5]
    let level_vertex_iplusjminus = 0.25 * (level_centers[1] + level_centers[2] + level_centers[4] + level_centers[5]);
    // levelset[i-0.5, j+0.5]
    let level_vertex_iminusjplus = 0.25 * (level_centers[3] + level_centers[4] + level_centers[6] + level_centers[7]);
    // levelset[i+0.5, j+0.5]
    let level_vertex_iplusjplus = 0.25 * (level_centers[4] + level_centers[5] + level_centers[7] + level_centers[8]);
    
    return AreaFractions(
        area_fraction(level_vertex_iminusjminus, level_vertex_iminusjplus),
        area_fraction(level_vertex_iplusjminus, level_vertex_iplusjplus),
        area_fraction(level_vertex_iminusjminus, level_vertex_iplusjminus),
        area_fraction(level_vertex_iminusjplus, level_vertex_iplusjplus),
    );
}