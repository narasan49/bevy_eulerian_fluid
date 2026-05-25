use bevy::{ecs::resource::Resource, math::UVec3, shader::ShaderDefVal};

#[derive(Resource)]
pub enum WorkgroupShape {
    Size8x8x4,
}

impl WorkgroupShape {
    pub fn shader_def(&self) -> ShaderDefVal {
        match self {
            WorkgroupShape::Size8x8x4 => "WG8X8".into(),
        }
    }

    pub fn workgroup_size(&self) -> UVec3 {
        match self {
            WorkgroupShape::Size8x8x4 => UVec3::new(8, 8, 4),
        }
    }
}

pub fn workgroup_size_center(size: UVec3, workgroup_shape: &WorkgroupShape) -> UVec3 {
    (size + workgroup_shape.workgroup_size() - 1) / workgroup_shape.workgroup_size()
}

pub fn workgroup_size_x(size: UVec3, workgroup_shape: &WorkgroupShape) -> UVec3 {
    (size + UVec3::X + workgroup_shape.workgroup_size() - 1) / workgroup_shape.workgroup_size()
}

pub fn workgroup_size_y(size: UVec3, workgroup_shape: &WorkgroupShape) -> UVec3 {
    (size + UVec3::Y + workgroup_shape.workgroup_size() - 1) / workgroup_shape.workgroup_size()
}

pub fn workgroup_size_z(size: UVec3, workgroup_shape: &WorkgroupShape) -> UVec3 {
    (size + UVec3::Z + workgroup_shape.workgroup_size() - 1) / workgroup_shape.workgroup_size()
}

pub fn workgroup_size_xyz(size: UVec3, workgroup_shape: &WorkgroupShape) -> UVec3 {
    (size + UVec3::ONE + workgroup_shape.workgroup_size() - 1) / workgroup_shape.workgroup_size()
}
