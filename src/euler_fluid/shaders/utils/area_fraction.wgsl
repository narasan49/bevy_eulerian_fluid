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
    x_lb: vec2<i32>,
) -> AreaFractions {
    let levelset_solid_ij = textureLoad(levelset_solid, x_lb).r;
    let levelset_solid_iplusj = textureLoad(levelset_solid, x_lb + vec2<i32>(1, 0)).r;
    let levelset_solid_ijplus = textureLoad(levelset_solid, x_lb + vec2<i32>(0, 1)).r;
    let levelset_solid_iplusjplus = textureLoad(levelset_solid, x_lb + vec2<i32>(1, 1)).r;

    return AreaFractions(
        area_fraction(levelset_solid_ij, levelset_solid_ijplus),
        area_fraction(levelset_solid_iplusj, levelset_solid_iplusjplus),
        area_fraction(levelset_solid_ij, levelset_solid_iplusj),
        area_fraction(levelset_solid_ijplus, levelset_solid_iplusjplus),
    );
}

struct FluidFractions {
    iminusj: f32,
    iplusj: f32,
    ijminus: f32,
    ijplus: f32,
}

fn fluid_fraction(
    levelset_air_adjacent: f32,
    levelset_air_ij: f32,
) -> f32 {
    if (levelset_air_adjacent < 0.0 && levelset_air_ij < 0.0) {
        return 1.0;
    }
    if (levelset_air_adjacent >= 0.0 && levelset_air_ij >= 0.0) {
        return 0.0;
    }
    return levelset_air_ij / (levelset_air_ij - levelset_air_adjacent);
}

fn fluid_fractions(
    levelset_air: texture_storage_2d<r32float, read_write>,
    x: vec2<i32>,
) -> FluidFractions {
    let levelset_air_ij = textureLoad(levelset_air, x).r;
    let levelset_air_iminusj = textureLoad(levelset_air, x - vec2<i32>(1, 0)).r;
    let levelset_air_iplusj = textureLoad(levelset_air, x + vec2<i32>(1, 0)).r;
    let levelset_air_ijminus = textureLoad(levelset_air, x - vec2<i32>(0, 1)).r;
    let levelset_air_ijplus = textureLoad(levelset_air, x + vec2<i32>(0, 1)).r;
    return FluidFractions(
        fluid_fraction(levelset_air_iminusj, levelset_air_ij),
        fluid_fraction(levelset_air_iplusj, levelset_air_ij),
        fluid_fraction(levelset_air_ijminus, levelset_air_ij),
        fluid_fraction(levelset_air_ijplus, levelset_air_ij),
    );
}