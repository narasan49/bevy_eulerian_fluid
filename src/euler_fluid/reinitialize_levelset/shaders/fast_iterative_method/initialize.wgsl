@group(0) @binding(0) var levelset_air: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var phi: texture_storage_2d<r32float, write>;
@group(0) @binding(2) var label: texture_storage_2d<r8uint, write>;

const LARGE_FLOAT: f32 = 1e30;
const LABEL_NONE: u32 = 0;
const LABEL_SOURCE: u32 = 1;
const LABEL_ACTIVE: u32 = 2;

@compute @workgroup_size(8, 8, 1)
fn initialize(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = vec2i(global_invocation_id.xy);
    let dim = vec2i(textureDimensions(levelset_air));

    let neighbors = array<vec2<i32>, 4>(
        idx + vec2i(-1, 0),
        idx + vec2i(1, 0),
        idx + vec2i(0, -1),
        idx + vec2i(0, 1),
    );

    let level = textureLoad(levelset_air, idx).r;

    for (var i = 0; i < 4; i++) {
        let idx_nb = neighbors[i];
        if (all(vec2i(0) <= idx_nb) && all(idx_nb < dim)) {
            let level_nb = textureLoad(levelset_air, idx_nb).r;
            if (level * level_nb <= 0.0) {
                textureStore(phi, idx, vec4f(level, 0.0, 0.0, 0.0));
                textureStore(label, idx, vec4u(LABEL_SOURCE, 0, 0, 0));
                return;
            }
        }
    }

    textureStore(phi, idx, vec4f(sign(level) * LARGE_FLOAT, 0.0, 0.0, 0.0));
    textureStore(label, idx, vec4u(LABEL_NONE, 0, 0, 0));
}