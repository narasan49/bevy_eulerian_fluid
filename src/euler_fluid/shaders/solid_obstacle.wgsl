#define_import_path bevy_fluid::solid_obstacle

const SHAPE_CIRCLE: u32 = 0;
const SHAPE_RECTANGLE: u32 = 1;
const SHAPE_TRIANGLE: u32 = 4;

struct Circle {
    radius: f32,
}

struct Rectangle {
    half_size: vec2<f32>,
}

struct ShapeVariant {
    shape: u32,
    values: array<f32, 6>,
}

fn get_circle(variant: ShapeVariant) -> Circle {
    return Circle(variant.values[0]);
}

fn get_rectangle(variant: ShapeVariant) -> Rectangle {
    return Rectangle(vec2<f32>(variant.values[0], variant.values[1]));
}

struct SolidObstacle {
    entity_id: u32,
    shape: ShapeVariant,
    transform: mat4x4<f32>,
    inverse_transform: mat4x4<f32>,
    linear_velocity: vec2<f32>,
    angular_velocity: f32,
}