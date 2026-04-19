#import bevy_sprite::mesh2d_vertex_output::VertexOutput;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let grid_spacing = 10.0;
    let line_width = 2.0;
    let uv = fract(mesh.world_position / grid_spacing);
    var color = vec3f(1.0);
    if any(uv.xy < vec2f(0.5 * line_width / grid_spacing)) {
        color = vec3f(0.0);
    }
    return vec4f(vec3f(color), 1.0);
}
