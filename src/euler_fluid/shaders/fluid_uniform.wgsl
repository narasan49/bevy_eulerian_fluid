#define_import_path bevy_fluid::fluid_uniform

struct SimulationUniform {
    dx: f32,
    dt: f32,
    rho: f32,
    gravity: vec2<f32>,
    initial_fluid_level: f32,
    fluid_transform: mat4x4<f32>,
}