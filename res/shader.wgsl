

struct VertexInput {
    @location(0) position: vec2<f32>,
    @builtin(vertex_index) idx: u32,
};


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>(input.position, 1.0, 1.0);


    if ((input.idx & 1u) == 0u) {
        out.tex_coords[0] = 0.0;
    } else {
        out.tex_coords[0] = 1.0;
    }
    if ((input.idx & 3u) < 2u) {
        out.tex_coords[1] = 0.0;
    } else {
        out.tex_coords[1] = 1.0;
    }
    return out;
}


@group(0) @binding(0)
var s_diffuse: sampler;

@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {


    let result = in.tex_coords - vec2<f32>(0.5, 0.5);
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if (result.x * result.x + result.y * result.y > 0.25) {
        discard;
    }



    return object_color;
}
