#import bevy_sprite::mesh2d_vertex_output::VertexOutput;

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var u_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var u_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var v_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var v_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let u_dimf = vec2f(textureDimensions(u_tex));
    let u_delta = vec2f(1.0) / u_dimf;
    let u_vec = array<f32, 4>(
        textureSample(u_tex, u_sampler, mesh.uv + u_delta * vec2f(0.0, -1.0)).r,
        textureSample(u_tex, u_sampler, mesh.uv + u_delta * vec2f(1.0, -1.0)).r,
        textureSample(u_tex, u_sampler, mesh.uv + u_delta * vec2f(0.0, 1.0)).r,
        textureSample(u_tex, u_sampler, mesh.uv + u_delta * vec2f(1.0, 1.0)).r,
    );
    let du_dy = 0.25 * (u_vec[2] + u_vec[3] - u_vec[0] - u_vec[1]);

    let v_dimf = vec2f(textureDimensions(v_tex));
    let v_delta = vec2f(1.0) / v_dimf;
    let v_vec = array<f32, 4>(
        textureSample(v_tex, v_sampler, mesh.uv + v_delta * vec2f(-1.0, 0.0)).r,
        textureSample(v_tex, v_sampler, mesh.uv + v_delta * vec2f(-1.0, 1.0)).r,
        textureSample(v_tex, v_sampler, mesh.uv + v_delta * vec2f(1.0, 0.0)).r,
        textureSample(v_tex, v_sampler, mesh.uv + v_delta * vec2f(1.0, 1.0)).r,
    );
    let dv_dx = 0.25 * (v_vec[2] + v_vec[3] - v_vec[0] - v_vec[1]);

    let vorticity = 0.1 * (dv_dx - du_dy) + 0.5;
    return vec4<f32>(vec3f(vorticity), 1.0);
}
