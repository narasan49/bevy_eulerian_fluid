#define_import_path bevy_fluid::fluid_to_solid::fixed_point_conversion
const FIXED_POINT_SCALE: f32 = 0.1;
const FIXED_POINT_INV_SCALE: f32 = 1.0 / FIXED_POINT_SCALE;

// Converts a fixed-point i32 to a floating-point f32.
fn i32_to_f32(i32_value: i32) -> f32 {
    return f32(i32_value) * FIXED_POINT_SCALE;
}

// Converts a floating-point f32 to a fixed-point i32.
fn f32_to_i32(f32_value: f32) -> i32 {
    return i32(f32_value * FIXED_POINT_INV_SCALE);
}