/**
 * bindings
 */

@group(0) @binding(0) var<storage, read> field : FieldBuf;
@group(0) @binding(1) var<storage, read_write> metrics : MetricsBuf;

/**
 * field buffer
 */

const BYTES_PER_LINE = 122u;

struct FieldBuf {
    bytes: array<u32>,
};

fn get_field_line_byte(line: u32, offset: u32) -> u32 {
    let idx = line * BYTES_PER_LINE + offset;

    let word = field.bytes[idx >> 2u];
    return (word >> ((idx & 0x3u) * 8u)) & 0xffu;
}

/**
 * metrics buffer
 */

struct MetricsBuf {
    num_lines: u32,
    flags: u32,
    top: u32,
    bottom: u32,
};

const METRIC_FLAG_VISIBLE = 0x01;

/**
 * metrics calculation - compute shader
 */

@compute @workgroup_size(1)
fn metrics_main() {
    metrics.num_lines = arrayLength(&field.bytes) * 4u / BYTES_PER_LINE;
    metrics.flags = 0u;

    for (var line = 0u; line < metrics.num_lines; line++) {
        let visible = get_field_line_byte(line, 0u);
        if visible != 0u {
            if (metrics.flags & METRIC_FLAG_VISIBLE) == 0u { // metrics not set yet
                metrics.flags = metrics.flags | METRIC_FLAG_VISIBLE;
                metrics.top = line;
            }

            metrics.bottom = line + 1;
        }
    }
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
    let y = u32(input.crt.y) + metrics.top;

    if (metrics.flags & METRIC_FLAG_VISIBLE) == 0u {
        return vec4f(0.5, 0.5, 0.5, 1.0);
    } else if y < metrics.num_lines {
        let byte = get_field_line_byte(y, 1u + u32(input.crt.x / 640.0 * 100.0));
        if byte > 0 {
            return vec4f(0.0, 1.0, 0.0, 1.0);
        } else {
            return vec4f(1.0, 0.0, 0.0, 1.0);
        }
    } else {
        return vec4f(0.0, 0.0, 1.0, 1.0);
    }
}
