#define_import_path euler_fluid_3d::fluid_uniform

struct FluidUniform {
    dx: f32,
    dt: f32,
    rho: f32,
    gravity: vec3<f32>,
    fluid_transform: mat4x4<f32>,
    size: vec3<f32>,
}