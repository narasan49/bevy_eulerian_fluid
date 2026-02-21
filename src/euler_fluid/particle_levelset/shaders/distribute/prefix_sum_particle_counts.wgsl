// Implements a section 39.2.4 of 
// https://developer.nvidia.com/gpugems/gpugems3/part-vi-gpu-computing/chapter-39-parallel-prefix-sum-scan-cuda

// cell_offsets must be padded with 0 to be a multiple of BLOCK_SIZE.
@group(0) @binding(0) var<storage, read_write> cell_offsets: array<u32>;
@group(0) @binding(1) var<storage, read_write> sums: array<u32>;

// BLOCK_SIZE should be a power of 2.
const BLOCK_SIZE: u32 = 512;
const NUM_THREADS_PER_WORKGROUP: u32 = BLOCK_SIZE / 2;

// the per-dimension limit [1024, 1024, 64] and the total invocation limit 768
const MAX_CHUNKS_X: u32 = 1024;
const MAX_CHUNKS_Y: u32 = 512 / BLOCK_SIZE;


var<workgroup> cell_offsets_local: array<u32, BLOCK_SIZE>;

@compute @workgroup_size(NUM_THREADS_PER_WORKGROUP, 1, 1)
fn prefix_sum_particle_counts_per_workgroup(
    @builtin(workgroup_id) workgroup_id : vec3<u32>,
    @builtin(local_invocation_index) local_invocation_index: u32,
) {
    let workgroup_idx = workgroup_id.x;
    let idx = i32(local_invocation_index);
    let depth = i32(log2(f32(BLOCK_SIZE)));
    let offset = i32(workgroup_idx * BLOCK_SIZE);

    cell_offsets_local[2 * idx] = cell_offsets[offset + 2 * idx];
    cell_offsets_local[2 * idx + 1] = cell_offsets[offset + 2 * idx + 1];
    workgroupBarrier();

    // Up-sweep
    for (var d = 0; d < depth; d++) {
        let width = 1 << u32(depth - d - 1);
        workgroupBarrier();
        if (idx < width) {
            let scale = 1 << u32(d);
            let left_idx = scale * (2 * idx + 1) - 1;
            let right_idx = scale * (2 * idx + 2) - 1;

            cell_offsets_local[right_idx] += cell_offsets_local[left_idx];
        }
    }

    workgroupBarrier();

    // Clear the last elements in the workgroup;
    if (local_invocation_index == 0) {
        // let workgroup_last_idx = (workgroup_idx + 1) * NUM_THREADS_PER_WORKGROUP - 1;
        let last_idx = i32(BLOCK_SIZE) - 1;
        sums[workgroup_idx] = cell_offsets_local[last_idx];
        cell_offsets_local[last_idx] = 0;
    }
    
    // Down-sweep
    for (var d = 0; d < depth; d++) {
        let width = 1 << u32(d);
        workgroupBarrier();
        if (idx < width) {
            let scale = 1 << u32(depth - d - 1);
            let left_idx = scale * (2 * idx + 1) - 1;
            let right_idx = scale * (2 * idx + 2) - 1;

            let temp = cell_offsets_local[left_idx];
            cell_offsets_local[left_idx] = cell_offsets_local[right_idx];
            cell_offsets_local[right_idx] += temp;
        }
    }

    workgroupBarrier();
    cell_offsets[offset + 2 * idx] = cell_offsets_local[2 * idx];
    cell_offsets[offset + 2 * idx + 1] = cell_offsets_local[2 * idx + 1];
}

var<workgroup> sums_local: array<u32, MAX_CHUNKS_X * MAX_CHUNKS_Y>;

@compute @workgroup_size(MAX_CHUNKS_X/2, MAX_CHUNKS_Y, 1)
fn prefix_sum_local_scans(
    @builtin(local_invocation_index) local_invocation_index: u32,
) {
    let idx = i32(local_invocation_index);
    let n = i32(arrayLength(&sums));
    let depth = i32(log2(f32(n)));

    let is_active = idx <= n / 2;

    if is_active {
        sums_local[2 * idx] = sums[2 * idx];
        sums_local[2 * idx + 1] = sums[2 * idx + 1];
    }
    workgroupBarrier();

    // Up-sweep
    for (var d = 0; d < depth; d++) {
        let width = 1 << u32(depth - d - 1);
        workgroupBarrier();
        if (idx < width) {
            let scale = 1 << u32(d);
            let left_idx = scale * (2 * idx + 1) - 1;
            let right_idx = scale * (2 * idx + 2) - 1;

            sums_local[right_idx] += sums_local[left_idx];
        }
    }
    workgroupBarrier();

    if (local_invocation_index == 0) {
        let workgroup_last_idx = n - 1;
        sums_local[workgroup_last_idx] = 0;
    }

    // Down-sweep
    for (var d = 0; d < depth; d++) {
        let width = 1 << u32(d);
        workgroupBarrier();
        if (idx < width) {
            let scale = 1 << u32(depth - d - 1);
            let left_idx = scale * (2 * idx + 1) - 1;
            let right_idx = scale * (2 * idx + 2) - 1;

            let temp = sums_local[left_idx];
            sums_local[left_idx] = sums_local[right_idx];
            sums_local[right_idx] += temp;
        }
    }

    workgroupBarrier();
    if is_active {
        sums[2 * idx] = sums_local[2 * idx];
        sums[2 * idx + 1] = sums_local[2 * idx + 1];
    }
}

@compute @workgroup_size(BLOCK_SIZE, 1, 1)
fn add_scanned_block_sums(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let idx = global_invocation_id.x;
    let block_idx = idx / BLOCK_SIZE;

    cell_offsets[idx] += sums[block_idx];
}
