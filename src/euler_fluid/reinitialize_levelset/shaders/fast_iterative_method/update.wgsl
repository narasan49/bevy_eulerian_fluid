
@group(0) @binding(0) var labels: texture_storage_2d<r8uint, read_write>;
@group(0) @binding(1) var phi: texture_storage_2d<r32float, read_write>;
// @group(0) @binding(2) var phi_sign: texture_storage_2d<r8sint, read_write>;


const LABEL_NONE: u32 = 0;
const LABEL_SOURCE: u32 = 1;
const LABEL_ACTIVE: u32 = 2;

const EPSIRON: f32 = 1e-6;
const LARGE_FLOAT: f32 = 1e30;
const SQRT2: f32 = sqrt(2.0);

@compute @workgroup_size(8, 8, 1)
fn update(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = vec2i(global_invocation_id.xy);

    let label = textureLoad(labels, idx).r;
    if label != LABEL_ACTIVE {
        return;
    }
    
    var p = textureLoad(phi, idx).r;
    var q = solve_quadratic_2d(phi, idx) * sign(p);
    textureStore(phi, idx, vec4f(q, 0.0, 0.0, 0.0));
    if abs(p - q) > EPSIRON {
        return;
    }

    let neighbors = array<vec2i, 4>(
        idx + vec2i(-1, 0),
        idx + vec2i(1, 0),
        idx + vec2i(0, -1),
        idx + vec2i(0, 1),
    );
    let dim = vec2i(textureDimensions(phi));

    textureStore(labels, idx, vec4u(LABEL_NONE, 0, 0, 0));
    for (var i = 0; i < 4; i++) {
        let idx_nb = neighbors[i];
        if (all(vec2i(0) <= idx_nb) && all(idx_nb < dim)) {
            let label_nb = textureLoad(labels, idx_nb).r;
            if label_nb != LABEL_ACTIVE && label_nb != LABEL_SOURCE {
                let p_nb = abs(textureLoad(phi, idx_nb).r);
                let q_nb = solve_quadratic_2d(phi, idx_nb);
                if p_nb > q_nb {
                    textureStore(phi, idx_nb, vec4f(q_nb * sign(p), 0.0, 0.0, 0.0));
                    // ToDo: Double buffering labels
                    textureStore(labels, idx_nb, vec4u(LABEL_ACTIVE, 0, 0, 0));
                }
            }
        }
    }
}

fn solve_quadratic_2d(
    phi: texture_storage_2d<r32float, read_write>,
    idx: vec2i,
) -> f32 {
    let phi_xmin = min(abs_get_phi(phi, idx + vec2i(-1, 0)), abs_get_phi(phi, idx + vec2i(1, 0)));
    let phi_ymin = min(abs_get_phi(phi, idx + vec2i(0, -1)), abs_get_phi(phi, idx + vec2i(0, 1)));
    
    let d = phi_xmin - phi_ymin;
    if d > SQRT2 {
        return phi_ymin + 1.0;
    } else if d < -SQRT2 {
        return phi_xmin + 1.0;
    } else {
        return 0.5 * (phi_xmin + phi_ymin + sqrt(2.0 - d * d));
    }
}

fn abs_get_phi(
    phi: texture_storage_2d<r32float, read_write>,
    idx: vec2i,
) -> f32 {
    let dim = vec2i(textureDimensions(phi));
    if any(idx < vec2i(0)) || any(dim <= idx) {
        return LARGE_FLOAT;
    }

    return abs(textureLoad(phi, idx).r);
}