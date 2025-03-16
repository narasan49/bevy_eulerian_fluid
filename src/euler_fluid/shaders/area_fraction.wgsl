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