#import bevy_fluid::particle_levelset::constants::BAND_WIDTH;

@group(0) @binding(0) var levelset_air: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var interface_band_mask: texture_storage_2d<r32uint, write>;

@compute @workgroup_size(8, 8, 1)
fn update_interface_band_mask(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.xy;

    let level = textureLoad(levelset_air, idx).r;
    if (abs(level) < BAND_WIDTH) {
        textureStore(interface_band_mask, idx, vec4<u32>(1, 0, 0, 0));
    } else {
        textureStore(interface_band_mask, idx, vec4<u32>(0, 0, 0, 0));
    }
}