#import euler_fluid_3d::fluid_uniform::FluidUniform
#import euler_fluid_3d::area_fraction::area_fraction
#import euler_fluid_3d::workgroup_shape::WG_SIZE

struct Force {
    force: vec3f,
    position: vec3f,
}

@group(0) @binding(0) var u1: texture_storage_3d<r32float, read_write>;
@group(0) @binding(1) var v1: texture_storage_3d<r32float, read_write>;
@group(0) @binding(2) var w1: texture_storage_3d<r32float, read_write>;
@group(0) @binding(3) var levelset_air0: texture_storage_3d<r32float, read>;
@group(0) @binding(4) var<storage, read> forces: array<Force>;
@group(0) @binding(5) var area_fraction_solid: texture_storage_3d<rgba32float, read>;

@group(1) @binding(0) var<uniform> constants: FluidUniform;

@compute @workgroup_size(WG_SIZE.x, WG_SIZE.y, WG_SIZE.z)
fn apply_forces(
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let idx = vec3i(gid);
    let dim = vec3i(textureDimensions(area_fraction_solid));
    if any(idx >= dim) {
        return;
    }

    let f_air_minus = area_fraction(levelset_air0, idx);
    let f_solid_minus = textureLoad(area_fraction_solid, gid);

    var net_force = constants.gravity;
    var n = arrayLength(&forces);
    loop {
        if (n == 0) {
            break;
        }
        n = n - 1u;
        let f = forces[n];
        net_force = net_force + f.force * gaussian_3d(vec3f(idx), f.position, 10.0);
    }
    
    if f_air_minus.x == 1.0 || f_solid_minus.x == 0.0 {
        textureStore(u1, idx, vec4f(0.0));
    } else {
        let u_value = textureLoad(u1, idx).r;
        textureStore(u1, idx, vec4f(u_value + net_force.x * constants.dt / constants.dx, 0.0, 0.0, 0.0));
    }
    
    if f_air_minus.y == 1.0 || f_solid_minus.y == 0.0 {
        textureStore(v1, idx, vec4f(0.0));
    } else {
        let v_value = textureLoad(v1, idx).r;
        textureStore(v1, idx, vec4f(v_value + net_force.y * constants.dt / constants.dx, 0.0, 0.0, 0.0));
    }
    
    if f_air_minus.z == 1.0 || f_solid_minus.z == 0.0 {
        textureStore(w1, idx, vec4f(0.0));
    } else {
        let w_value = textureLoad(w1, idx).r;
        textureStore(w1, idx, vec4f(w_value + net_force.z * constants.dt / constants.dx, 0.0, 0.0, 0.0));
    }
}

fn gaussian_3d(x: vec3f, x0: vec3f, sigma: f32) -> f32 {
    let b = -1.0 / (2.0 * sigma * sigma);
    return exp(b * dot(x - x0, x - x0));
}
