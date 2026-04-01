@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var in_is_u_valid: texture_storage_2d<r32sint, read>;
@group(0) @binding(2) var out_is_u_valid: texture_storage_2d<r32sint, write>;

@compute @workgroup_size(1, 64, 1)
fn extrapolate_u(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let idx = vec2<i32>(invocation_id.xy);
    let dim = vec2<i32>(textureDimensions(in_is_u_valid));
    let is_valid = textureLoad(in_is_u_valid, idx).r;
    if (is_valid == 1) {
        textureStore(out_is_u_valid, idx, vec4<i32>(1, 0, 0, 0));
    } else {
        var count = 0;
        var new_u = 0.0;

        let right = idx + vec2<i32>(1, 0);
        if all(right < dim) {
            let is_valid_right = textureLoad(in_is_u_valid, right).r;
            if is_valid_right == 1 {
                new_u += textureLoad(u0, right).r;
                count += 1;
            }

        }
        
        let left = idx + vec2<i32>(-1, 0);
        if all(left > vec2<i32>(0)) {
            let is_valid_left = textureLoad(in_is_u_valid, left).r;
            if is_valid_left == 1 {
                new_u += textureLoad(u0, left).r;
                count += 1;
            }
        }
        
        let top = idx + vec2<i32>(0, 1);
        if all(top < dim) {
            let is_valid_top = textureLoad(in_is_u_valid, top).r;
            if is_valid_top == 1 {
                new_u += textureLoad(u0, top).r;
                count += 1;
            }
        }
        
        let bottom = idx + vec2<i32>(0, -1);
        if all(bottom > vec2<i32>(0)) {
            let is_valid_bottom = textureLoad(in_is_u_valid, bottom).r;
            if is_valid_bottom == 1 {
                new_u += textureLoad(u0, bottom).r;
                count += 1;
            }
        }

        if count > 0 {
            new_u /= f32(count);
            textureStore(u0, idx, vec4<f32>(new_u, 0, 0, 0));
            textureStore(out_is_u_valid, idx, vec4<i32>(1, 0, 0, 0));
        } else {
            textureStore(out_is_u_valid, idx, vec4<i32>(0, 0, 0, 0));
        }
    }
}