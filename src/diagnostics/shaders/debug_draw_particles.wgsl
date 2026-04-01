#import bevy_render::view::uv_to_ndc;
#import bevy_sprite::mesh2d_functions
#import bevy_sprite::mesh2d_view_bindings::globals
#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::particle_levelset::particle::Particle;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) color: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<storage, read> particles: array<Particle>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<storage, read> particles_count: u32;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> fluid_size: vec2<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> color: vec4<f32>;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let idx = mesh2d_functions::get_tag(vertex.instance_index);
    if idx < particles_count {
        let particle_position = particles[idx].position;

        let world_from_local = mesh2d_functions::get_world_from_local(vertex.instance_index);

        let fluid_uv = particle_position / fluid_size;
        let ndc = uv_to_ndc(fluid_uv);
        let half_size = 0.5 * fluid_size;
        let world_position = world_from_local * vec4<f32>(ndc * half_size, 0.0, 1.0);

        out.world_position = vec4<f32>(vertex.position.xy + world_position.xy, 3.0, 1.0);
        out.clip_position = mesh2d_functions::mesh2d_position_world_to_clip(out.world_position);
        out.color = color;
    }
    return out;
}

@fragment
fn fragment(
    mesh: VertexOutput
) -> @location(0) vec4<f32> {
    return mesh.color;
}
