#import bevy_pbr::{
    mesh_view_bindings::globals,
    mesh_functions,
    mesh_view_bindings,
    view_transformations,
}
@group(2) @binding(0) var<uniform> t: i32;
@group(2) @binding(1) var<uniform> origin: vec3f;
@group(2) @binding(2) var<uniform> x_axis: vec3f;
@group(2) @binding(3) var<uniform> y_axis: vec3f;
@group(2) @binding(4) var<uniform> inputbounds: vec4<i32>;

// linear map f s.t. f(-1.0) = lb and f(1.0) = ub
// assumes |x| <= 1.0
fn f2i(x: f32, bounds: vec2<i32>) -> i32 {
    let lb = bounds.x;
    let ub = bounds.y;

    if lb == ub {
        return lb;
    }

    let a = f32(u32(ub - lb));
    let b = i64(lb) + i64(ub);

    let ax = (x * a + f32(b & 1)) * 0.5;

    let y = i64(round(ax)) + (b >> 1);
    return i32(y);
}

struct Vertex {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

@vertex
fn vertex(
    v: Vertex,
) -> VertexOut {
    var out: VertexOut;
    let tri = v.vertex_index / 3;
    let vert = v.vertex_index % 3;

    var posdir: vec4f;

    let k = (vert + tri) % 4;

    if vert == 0 {
        posdir = vec4(origin, 1);
        out.uv = vec2f(0,0);
    } else if k == 0 {
        posdir = vec4(x_axis, 0);
        out.uv = vec2f(1,0);
    } else if k == 1 {
        posdir = vec4(y_axis, 0);
        out.uv = vec2f(0,1);
    } else if k == 2 {
        posdir = vec4(-x_axis, 0);
        out.uv = vec2f(-1,0);
    } else/*if k == 3 */ {
        posdir = vec4(-y_axis, 0);
        out.uv = vec2f(0,-1);
    }
    out.clip_pos = mesh_view_bindings::view.clip_from_world * posdir;
    out.builtin_position = out.clip_pos;
    return out;
}

struct VertexOut {
    @builtin(position) builtin_position: vec4f,
    @location(0) @interpolate(perspective) clip_pos: vec4f,
    @location(1) @interpolate(perspective) uv: vec2f,
}


@fragment
fn fragment(
    v: VertexOut,
) ->  @location(0) vec4f {
    let near = 0.1;
    let xy = near * v.uv / v.clip_pos.z;

    let x = xy.x;
    let y = xy.y;


    if all(abs(xy) <= vec2(1.0)) {
        discard;
    }

    if all(abs(xy) <= vec2(2.0)) {
        var n: u32 = 0;
        var k: u32 = 0;
        var m: u32 = 0;
        if abs(y) <= 1.0 {
            n = u32(f2i(y, inputbounds.yw));
            k = u32(32.0 * (abs(x) - 1.0));
            m = countLeadingZeros(u32(inputbounds.w - inputbounds.y));
        } else if abs(x) <= 1.0 {
            n = u32(f2i(x, inputbounds.xz));
            k = u32(32.0 * (abs(y) - 1.0));
            m = countLeadingZeros(u32(inputbounds.z - inputbounds.x));
        }

        let bit = (n >> k) & 1;

        let zero = vec4f(0.01,0.01,0.01,1.0);
        let one  = vec4f(0.1,0.1,0.1,1.0);

        if k + 7 < 32 - m {        
            return zero + 0.1 * one;
        } else if bit != 0 {
            return one;
        } else {
            return zero;
        }
    }

    discard;
}

