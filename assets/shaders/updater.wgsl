@group(0) @binding(0) var<uniform> size: u32;
@group(0) @binding(1) var<storage, read_write> idxs: array<u32>;
@group(0) @binding(2) var<storage, read_write> src: array<i32>;
@group(0) @binding(3) var<storage, read_write> dst: array<i32>;

struct ComputeInputs {
    @builtin(local_invocation_id) id: vec3<u32>,
    @builtin(local_invocation_index) idx: u32,
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
    @builtin(num_workgroups) dims: vec3<u32>,
}

@compute @workgroup_size(256)
fn main(ctx: ComputeInputs) {
    var k = ctx.gid.x;
    while (k < size) {
        let idx = idxs[k];
        dst[idx] = src[k];
        k += 256u;
    }
}