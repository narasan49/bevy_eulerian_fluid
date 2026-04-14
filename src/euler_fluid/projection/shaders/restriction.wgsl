@group(0) @binding(0) var r: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var levelset_air: texture_storage_2d<r32float, read>;
@group(0) @binding(2) var area_fraction_solid: texture_storage_2d<rgba32float, read>;

@group(0) @binding(3) var b_low: texture_storage_2d<r32float, write>;
@group(0) @binding(4) var levelset_air_low: texture_storage_2d<r32float, write>;
@group(0) @binding(5) var area_fraction_solid_low: texture_storage_2d<rgba32float, write>;
@group(0) @binding(6) var x_low: texture_storage_2d<r32float, write>;

@compute @workgroup_size(8, 8, 1)
fn restriction(
    @builtin(global_invocation_id) global_invocation_id: vec3u,
) {
    let idx = global_invocation_id.xy;
    let dim = textureDimensions(b_low);
    if any(idx >= dim) {
        return;
    }

    let b_r = 0.25 * (textureLoad(r, 2 * idx).r
        + textureLoad(r, 2 * idx + vec2u(0, 1)).r
        + textureLoad(r, 2 * idx + vec2u(1, 0)).r
        + textureLoad(r, 2 * idx + vec2u(1, 1)).r);
        
    var phi0 = textureLoad(levelset_air, 2 * idx).r;
    let phi1 = textureLoad(levelset_air, 2 * idx + vec2u(0, 1)).r;
    let phi2 = textureLoad(levelset_air, 2 * idx + vec2u(1, 0)).r;
    let phi3 = textureLoad(levelset_air, 2 * idx + vec2u(1, 1)).r;

    var phi_r = phi0;
    if abs(phi1) < abs(phi_r) {
        phi_r = phi1;
    }
    if abs(phi2) < abs(phi_r) {
        phi_r = phi2;
    }
    if abs(phi3) < abs(phi_r) {
        phi_r = phi3;
    }
    // let phi_r = 0.25 * (phi0 + phi1 + phi2 + phi3);

    let fractions = array<vec4f, 4>(
        textureLoad(area_fraction_solid, 2 * idx),
        textureLoad(area_fraction_solid, 2 * idx + vec2u(1, 0)),
        textureLoad(area_fraction_solid, 2 * idx + vec2u(0, 1)),
        textureLoad(area_fraction_solid, 2 * idx + vec2u(1, 1)),
    );

    let f_r = vec4f(
        0.5 * (fractions[0].x + fractions[2].x),
        0.5 * (fractions[1].y + fractions[3].y),
        0.5 * (fractions[0].z + fractions[1].z),
        0.5 * (fractions[2].w + fractions[3].w),
    );

    textureStore(b_low, idx, vec4f(b_r, vec3f(0)));
    textureStore(levelset_air_low, idx, vec4f(phi_r, vec3f(0)));
    textureStore(area_fraction_solid_low, idx, f_r);
    textureStore(x_low, idx, vec4f(0));
}