@group(0) @binding(0) var t_biffuse: texture_2d<f32>;
@group(0) @binding(1) var s_biffuse: sampler;

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_cords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_main: u32,
) -> VertexOutput{
    var out: VertexOutput;

    let x = f32(1 - i32(in_vertex_main)) * 0.5;
    let y = f32(i32(in_vertex_main & 1u) * 2) * 0.5;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);

    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32>{
    let texture_color = textureSample(t_biffuse, s_biffuse, in.tex_cords);

    return texture_color * in.color;
}