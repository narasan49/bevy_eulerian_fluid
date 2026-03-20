use bevy::{math::Vec2, render::render_resource::ShaderType};

pub(crate) const MAX_PARTICLES_PER_CELL: usize = 16;

#[derive(Clone, ShaderType)]
pub struct Particle {
    position: Vec2,
    radius: f32,
    sign: f32,
    escaped: u32,
}

impl Particle {
    pub const ZERO: Particle = Particle {
        position: Vec2::ZERO,
        radius: 0.0,
        sign: 0.0,
        escaped: 0,
    };
}
