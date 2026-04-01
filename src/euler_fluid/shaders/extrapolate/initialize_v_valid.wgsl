#import bevy_fluid::area_fraction::area_fraction;

@group(0) @binding(0) var is_v_valid: texture_storage_2d<r32sint, write>;
@group(0) @binding(1) var levelset_air: texture_storage_2d<r32float, read>;

@compute @workgroup_size(64, 1, 1)
fn initialize_v_valid(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(invocation_id.xy);

    let level_centers = array<f32, 6>(
        textureLoad(levelset_air, idx + vec2<i32>(-1, -1)).r,
        textureLoad(levelset_air, idx + vec2<i32>(-1, 0)).r,
        textureLoad(levelset_air, idx + vec2<i32>(0, -1)).r,
        textureLoad(levelset_air, idx + vec2<i32>(0, 0)).r,
        textureLoad(levelset_air, idx + vec2<i32>(1, -1)).r,
        textureLoad(levelset_air, idx + vec2<i32>(1, 0)).r,
    );

    let level_air_vertex_minus = 0.25 * (level_centers[0] + level_centers[1] + level_centers[2] + level_centers[3]);
    let level_air_vertex_plus = 0.25 * (level_centers[2] + level_centers[3] + level_centers[4] + level_centers[5]);
    let area_fraction = area_fraction(level_air_vertex_minus, level_air_vertex_plus);

    if area_fraction > 0.0 {
        textureStore(is_v_valid, idx, vec4<i32>(0, 0, 0, 0));
    } else {
        textureStore(is_v_valid, idx, vec4<i32>(1, 0, 0, 0));
    }
}