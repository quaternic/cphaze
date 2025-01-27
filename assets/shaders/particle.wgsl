#import bevy_pbr::{
    mesh_view_bindings::globals,
    mesh_bindings::mesh,
    mesh_functions,
    mesh_view_bindings,
}
@group(2) @binding(0) var<uniform> t: i32;
@group(2) @binding(1) var<uniform> color: vec4f;
@group(2) @binding(2) var<uniform> xy_bounds: vec4<i32>;
@group(2) @binding(3) var<uniform> z_scale: f32;

struct Vertex {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
    @location(0) x: i32,
    @location(1) y: i32,
    @location(2) z: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

// linear map f s.t. f(lb) = -1.0 and f(ub) = 1.0
fn int_map(x: i32, lb: i32, ub: i32) -> f32 {
    // f(x) = (2x - (lb + ub)) / (ub-lb)
    // i32::MIN + 1 == -i32::MAX
    // min(2x - lb - ub) = 2 * i32::MIN - 2 * i32::MAX = -2 * u32::MAX
    // max(2x - lb - ub) = 2 * i32::MAX - 2 * i32::MIN =  2 * u32::MAX

    if lb == ub {
        if x == lb {
            return 0.0;
        } else if x < lb {
            return -2.0;
        } else {
            return 2.0;
        }
    }

    let y = i64(x) + i64(x) - i64(lb) - i64(ub);
    return f32(y) / f32(u32(ub - lb));
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let x = int_map(vertex.x, xy_bounds.x, xy_bounds.z);
    let y = int_map(vertex.y, xy_bounds.y, xy_bounds.w);
    var z = vertex.z * z_scale;

    // world position
    let wp = vec4f(x, y, z, 1.0);
    out.clip_position = mesh_view_bindings::view.clip_from_world * wp;

    // distance to camera, to scale the brightness
    let d = wp.xyz - mesh_view_bindings::view.world_position;
    out.color = color * inverseSqrt(dot(d,d));
    return out;
}

struct FragIn {
    @builtin(position) p: vec4f,
    @location(0) color: vec4f,
}

@fragment
fn fragment(
    @location(0) color: vec4f,
) -> @location(0) vec4f {
    return color;
}
