struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) crt: vec2<f32>,
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
    output.crt = vec2f((1.0 + position2d.x) * 320.0, (1.0 - position2d.y) * 256.0);

    return output;
}

const BYTES_PER_ROW = 122u;
const MAX_ROWS = 320u;

struct FieldBuf {
    bytes: array<u32>, // BYTES_PER_ROW * MAX_ROWS / 4u
};

@group(0) @binding(0) var<storage, read> field : FieldBuf;

fn load_field_byte(idx: u32) -> u32 {
    let word = field.bytes[idx >> 2u];
    return (word >> ((idx & 3u) * 8u)) & 0xffu;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4f {
    let y = u32(input.crt.y);

    if (y < MAX_ROWS) {
        let byte = load_field_byte(y * BYTES_PER_ROW + 1u + u32(input.crt.x / 640.0 * 100.0));
        if (byte > 0) {
            return vec4f(0.0, 1.0, 0.0, 1.0);
        } else {
            return vec4f(1.0, 0.0, 0.0, 1.0);
        }
    } else {
        return vec4f(0.0, 0.0, 1.0, 1.0);
    }
}
