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

struct FieldBuf {
    bytes: array<u32>,
};

fn load_field_byte(idx: u32) -> u32 {
    let word = field.bytes[idx >> 2u];
    return (word >> ((idx & 3u) * 8u)) & 0xffu;
}

@group(0) @binding(0) var<storage, read> field : FieldBuf;

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4f {
    let uv_int = vec2u(u32(input.uv.x * 640.0), u32(input.uv.y * 512.0));

    if (uv_int.y < 320u) {
        let byte = load_field_byte(uv_int.y * 122u + 1u + u32(f32(uv_int.x) / 640.0 * 100.0));
        if (byte > 0) {
            return vec4f(0.0, 1.0, 0.0, 1.0);
        } else {
            return vec4f(1.0, 0.0, 0.0, 1.0);
        }
    } else {
        return vec4f(0.0, 0.0, 1.0, 1.0);
    }
}
