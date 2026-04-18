@group(0) @binding(0) var labels_in: texture_storage_2d<r32uint, read>;
@group(0) @binding(1) var labels_out: texture_storage_2d<r32uint, write>;

const LABEL_NONE: u32 = 0;
const LABEL_SOURCE: u32 = 1;
const LABEL_ACTIVE: u32 = 2;

@compute @workgroup_size(8, 8, 1)
fn initialize_active_label(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = vec2i(global_invocation_id.xy);
    let dim = vec2i(textureDimensions(labels_in));

    let neighbors = array<vec2<i32>, 4>(
        idx + vec2i(-1, 0),
        idx + vec2i(1, 0),
        idx + vec2i(0, -1),
        idx + vec2i(0, 1),
    );

    let label = textureLoad(labels_in, idx).r;
    if label == LABEL_SOURCE {
        textureStore(labels_out, idx, vec4u(LABEL_SOURCE, 0, 0, 0));
        return;
    }

    for (var i = 0; i < 4; i++) {
        let idx_nb = neighbors[i];
        if (all(vec2i(0) <= idx_nb) && all(idx_nb < dim)) {
            let label_nb = textureLoad(labels_in, idx_nb).r;
            if (label_nb == LABEL_SOURCE) {
                textureStore(labels_out, idx, vec4u(LABEL_ACTIVE, 0, 0, 0));
                return;
            }
        }
    }

    textureStore(labels_out, idx, vec4u(LABEL_NONE, 0, 0, 0));
}