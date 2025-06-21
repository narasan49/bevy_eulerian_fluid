#define_import_path bevy_fluid::fluid_to_solid::fixed_point_conversion

// Converts a fixed-point i32 to a floating-point f32.
fn i32_to_f32(i32_value: i32) -> f32 {
    return f32(i32_value) * 0.0000001;
}

// Converts a floating-point f32 to a fixed-point i32.
fn f32_to_i32(f32_value: f32) -> i32 {
    return i32(f32_value * 10000000.0);
}