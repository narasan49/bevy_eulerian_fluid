@group(0) @binding(0) var data: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var<storage, read_write> partial_sums: array<f32>;
@group(0) @binding(2) var<storage, read_write> sum: f32;

const SIZE_X = 16;
const SIZE_Y = 16;
const WG_SIZE = SIZE_X * SIZE_Y;
const WG_SIZE_2 = 512;
var<workgroup> workgroup_sum: array<f32, WG_SIZE>;

@compute @workgroup_size(SIZE_X, SIZE_Y, 1)
fn partial_reduction(
    @builtin(global_invocation_id) global_invocation_id: vec3u,
    @builtin(local_invocation_index) lid: u32,
    @builtin(num_workgroups) num_workgroups: vec3u,
    @builtin(workgroup_id) workgroup_id: vec3u,
) {
    let idx = global_invocation_id.xy;
    let dim = textureDimensions(data);
    if all(idx < dim) {
        let data_sample = textureLoad(data, idx).r;
        if data_sample < 0.0 {
            workgroup_sum[lid] = 1.0;
        } else {
            workgroup_sum[lid] = 0.0;
        }
    }

    workgroupBarrier();

    var stride: u32 = WG_SIZE / 2;
    loop {
        if lid < stride {
            workgroup_sum[lid] += workgroup_sum[lid + stride];
        }
        if stride == 1 {
            break;
        }
        stride /= 2;

        workgroupBarrier();
    }

    if lid == 0 {
        let wid = workgroup_id.x + workgroup_id.y * num_workgroups.x;
        partial_sums[wid] = workgroup_sum[0];
    }
}

var<workgroup> partials2: array<f32, WG_SIZE_2>;

@compute @workgroup_size(WG_SIZE_2, 1, 1)
fn reduction(
    @builtin(local_invocation_index) lid: u32,
) {
    let n = arrayLength(&partial_sums);

    var tmp_sum = 0.0;
    for (var i = lid; i < n; i += WG_SIZE_2) {
        tmp_sum += partial_sums[i];
    }
    partials2[lid] = tmp_sum;

    workgroupBarrier();

    for (var stride: u32 = WG_SIZE_2 / 2; stride > 0; stride /= 2) {
        if lid < stride {
            partials2[lid] += partials2[lid + stride];
        }
        workgroupBarrier();
    }

    if lid == 0 {
        sum = partials2[0];
    }
}