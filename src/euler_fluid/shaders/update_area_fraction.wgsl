#import bevy_fluid::area_fraction::area_fractions

@group(0) @binding(0) var levelset_solid: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var area_fraction_solid: texture_storage_2d<rgba32float, write>;

@compute @workgroup_size(8, 8, 1)
fn update_area_fraction(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = vec2i(global_invocation_id.xy);
    let f = area_fractions(levelset_solid, idx);

    textureStore(area_fraction_solid, idx, vec4<f32>(f.iminusj, f.iplusj, f.ijminus, f.ijplus));
}