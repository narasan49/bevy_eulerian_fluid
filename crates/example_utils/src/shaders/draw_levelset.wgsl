#import bevy_sprite::mesh2d_vertex_output::VertexOutput;

@group(2) @binding(0) var levelset_texture: texture_2d<f32>;
@group(2) @binding(1) var levelset_sampler: sampler;
@group(2) @binding(2) var<uniform> base_color: vec3<f32>;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    var v = textureSample(levelset_texture, levelset_sampler, mesh.uv).r;
    if abs(v) < 0.5 {
        return vec4<f32>(vec3f(0.0), 1.0);
    }
    v = step(v, 0.0);
    return vec4<f32>(v * base_color, v * 0.5);
}
