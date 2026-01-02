/**
 * bindings
 */

@group(0) @binding(0) var<storage, read> field : FieldBuf;
@group(0) @binding(1) var<storage, read_write> metrics : MetricsBuf;

/**
 * field buffer
 */

const BYTES_PER_ROW = 122u;

struct FieldBuf {
    bytes: array<u32>,
};

fn get_field_row_byte(row: u32, offset: u32) -> u32 {
    let idx = row * BYTES_PER_ROW + offset;

    let word = field.bytes[idx >> 2u];
    return (word >> ((idx & 0x3u) * 8u)) & 0xffu;
}

/**
 * metrics buffer
 */

struct MetricsBuf {
    num_rows: u32,
    first_visible_line: u32,
};

/**
 * metrics calculation - compute shader
 */

@compute @workgroup_size(1)
fn metrics_main() {
    metrics.num_rows = arrayLength(&field.bytes) * 4u / BYTES_PER_ROW;

    var first_visible = metrics.num_rows;
    
    for (var row = 0u; row < metrics.num_rows; row++) {
        let first_byte = get_field_row_byte(row, 0u);
        if (first_byte != 0u) {
            first_visible = row;
            break;
        }
    }
    
    metrics.first_visible_line = first_visible;
}

/**
 * vertex / fragment shared
 */

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) crt: vec2<f32>,
}

/**
 * vertex
 */

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

/**
 * fragment - render
 */

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4f {
    let y = u32(input.crt.y) + metrics.first_visible_line;

    if (y < metrics.num_rows) {
        let byte = get_field_row_byte(y, 1u + u32(input.crt.x / 640.0 * 100.0));
        if (byte > 0) {
            return vec4f(0.0, 1.0, 0.0, 1.0);
        } else {
            return vec4f(1.0, 0.0, 0.0, 1.0);
        }
    } else {
        return vec4f(0.0, 0.0, 1.0, 1.0);
    }
}
