@group(0) @binding(0) var x: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var x_low: texture_storage_2d<r32float, read>;
@group(0) @binding(2) var levelset_air: texture_storage_2d<r32float, read>;

@compute @workgroup_size(8, 8, 1)
fn prolongation(
    @builtin(global_invocation_id) global_invocation_id: vec3u,
) {
    let idx = global_invocation_id.xy;
    let dim = textureDimensions(x_low);
    if any(idx >= dim) {
        return;
    }

    let correction = textureLoad(x_low, idx);
    let offsets = array<vec2u, 4>(
        vec2u(0, 0),
        vec2u(1, 0),
        vec2u(0, 1),
        vec2u(1, 1),
    );
    let x0 = textureLoad(x_low, idx);
    let x1 = textureLoad(x_low, idx + offsets[1]);
    let x2 = textureLoad(x_low, idx + offsets[2]);
    let x3 = textureLoad(x_low, idx + offsets[3]);

    for (var i = 0u; i < 4; i++) {
        let fine_idx = 2 * idx + offsets[i];
        let level = textureLoad(levelset_air, fine_idx).r;
        if level < 0.0 {
            textureStore(x, fine_idx, correction + textureLoad(x, fine_idx));
        }
    }
}