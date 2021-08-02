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
struct Uniforms {
    model_view_projection: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[stage(vertex)]]
fn main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = in.tex_coord;
    out.normal = in.normal;
    out.pos = uniforms.model_view_projection * vec4<f32>(in.pos, 1.0);
    return out;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.normal, 1.0);
}
