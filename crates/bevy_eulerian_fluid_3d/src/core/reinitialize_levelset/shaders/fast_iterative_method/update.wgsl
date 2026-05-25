#import euler_fluid_3d::workgroup_shape::WG_SIZE

@group(0) @binding(0) var labels: texture_storage_3d<r32uint, read_write>;
@group(0) @binding(1) var phi: texture_storage_3d<r32float, read_write>;


const LABEL_NONE: u32 = 0;
const LABEL_SOURCE: u32 = 1;
const LABEL_ACTIVE: u32 = 2;

const EPSIRON: f32 = 1e-6;
const LARGE_FLOAT: f32 = 1e30;
const SQRT2: f32 = sqrt(2.0);

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn update(
    @builtin(global_invocation_id) gid: vec3u,
) {
    let idx = vec3i(gid);

    let label = textureLoad(labels, idx).r;
    if label != LABEL_ACTIVE {
        return;
    }
    
    var p = textureLoad(phi, idx).r;
    var q = solve_quadratic_3d(phi, idx) * sign(p);
    textureStore(phi, idx, vec4f(q, 0.0, 0.0, 0.0));
    if abs(p - q) > EPSIRON {
        return;
    }

    let neighbors = array<vec3i, 6>(
        idx + vec3i(-1, 0, 0),
        idx + vec3i(1, 0, 0),
        idx + vec3i(0, -1, 0),
        idx + vec3i(0, 1, 0),
        idx + vec3i(0, 0, -1),
        idx + vec3i(0, 0, 1),
    );
    let dim = vec3i(textureDimensions(phi));

    textureStore(labels, idx, vec4u(LABEL_NONE, 0, 0, 0));
    for (var i = 0; i < 6; i++) {
        let idx_nb = neighbors[i];
        if (all(vec3i(0) <= idx_nb) && all(idx_nb < dim)) {
            let label_nb = textureLoad(labels, idx_nb).r;
            if label_nb != LABEL_ACTIVE && label_nb != LABEL_SOURCE {
                let p_nb = abs(textureLoad(phi, idx_nb).r);
                let q_nb = solve_quadratic_3d(phi, idx_nb);
                if p_nb > q_nb {
                    textureStore(phi, idx_nb, vec4f(q_nb * sign(p), 0.0, 0.0, 0.0));
                    textureStore(labels, idx_nb, vec4u(LABEL_ACTIVE, 0, 0, 0));
                }
            }
        }
    }
}

fn solve_quadratic_3d(
    phi: texture_storage_3d<r32float, read_write>,
    idx: vec3i,
) -> f32 {
    let phi_xmin = min(abs_get_phi(phi, idx + vec3i(-1, 0, 0)), abs_get_phi(phi, idx + vec3i(1, 0, 0)));
    let phi_ymin = min(abs_get_phi(phi, idx + vec3i(0, -1, 0)), abs_get_phi(phi, idx + vec3i(0, 1, 0)));
    let phi_zmin = min(abs_get_phi(phi, idx + vec3i(0, 0, -1)), abs_get_phi(phi, idx + vec3i(0, 0, 1)));
    
    var phi_sorted = vec3f(phi_xmin, phi_ymin, phi_zmin);
    if phi_sorted.x > phi_sorted.y {
        let tmp = phi_sorted.x;
        phi_sorted.x = phi_sorted.y;
        phi_sorted.y = tmp;
    }
    if phi_sorted.x > phi_sorted.z {
        let tmp = phi_sorted.x;
        phi_sorted.x = phi_sorted.z;
        phi_sorted.z = tmp;
    }
    if phi_sorted.y > phi_sorted.z {
        let tmp = phi_sorted.y;
        phi_sorted.y = phi_sorted.z;
        phi_sorted.z = tmp;
    }

    let d0 = phi_sorted.z - phi_sorted.x;
    let d1 = phi_sorted.y - phi_sorted.x;
    if d0 < 1.0 {
        let phi_sum = phi_sorted.x + phi_sorted.y + phi_sorted.z;
        let phi_sq_sum = phi_sorted.x * phi_sorted.x + phi_sorted.y * phi_sorted.y + phi_sorted.z * phi_sorted.z;
        return 1.0 / 6.0 * (2.0 * phi_sum + sqrt(4.0 * phi_sum * phi_sum - 12.0 * (phi_sq_sum - 1.0)));
    } else if d1 < 1.0 {
        return 0.5 * (phi_sorted.x + phi_sorted.y + sqrt(2.0 - d1 * d1));
    } else {
        return phi_sorted.x + 1.0;
    }
}

fn abs_get_phi(
    phi: texture_storage_3d<r32float, read_write>,
    idx: vec3i,
) -> f32 {
    let dim = vec3i(textureDimensions(phi));
    if any(idx < vec3i(0)) || any(dim <= idx) {
        return LARGE_FLOAT;
    }

    return abs(textureLoad(phi, idx).r);
}