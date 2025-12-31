struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(@builtin(vertex_index) VertexIndex : u32) -> VertexOutput {
    var positions = array<vec2f, 3>(
        vec2f(-1.0, -1.0),
        vec2f(3.0, -1.0),
        vec2f(-1.0, 3.0),
    );

    var output: VertexOutput;
    let position2d: vec2<f32> = positions[VertexIndex];
    output.position = vec4f(position2d, 0.0, 1.0);
    output.uv = vec2f(position2d.x * 0.5 + 0.5, 1.0 - (position2d.y * 0.5 + 0.5));

    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4f {
    let uv_int = vec2u(u32(input.uv.x * 640.0), u32(input.uv.y * 512.0));
    if (uv_int.x == 0u || uv_int.y == 0u) {
        return vec4f(0.0, 0.0, 1.0, 1.0);
    }
    if (uv_int.x == 639u || uv_int.y == 511u) {
        return vec4f(0.0, 1.0, 0.0, 1.0);
    }
    return vec4f(1.0, 0.0, 0.0, 1.0);
}
