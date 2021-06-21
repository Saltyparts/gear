struct VertexInput {
    [[location(0)]] pos: vec3<f32>;
    [[location(1)]] tex_coord: vec2<f32>;
    [[location(2)]] normal: vec3<f32>;
};

struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[builtin(position)]] pos: vec4<f32>;
};

[[block]]
struct Locals {
    view_pos: vec4<f32>;
    view_proj: mat4x4<f32>;
    pos: vec3<f32>;
};

[[group(0), binding(0)]]
var<uniform> locals: Locals;

[[stage(vertex)]]
fn main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = in.tex_coord;
    out.normal = in.normal;
    out.pos = locals.view_proj * vec4<f32>(in.pos + locals.pos, 1.0);
    return out;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.normal, 1.0);
}
