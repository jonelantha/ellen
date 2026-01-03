/**
 * bindings
 */

@group(0) @binding(0) var<storage, read> field : FieldBuf;
@group(0) @binding(1) var<storage, read_write> metrics : MetricsBuf;

/**
 * field buffer
 */

const BYTES_PER_LINE = 122u;
const TELETEXT_DELAY_OFFSET = 36u;

struct FieldBuf {
    bytes: array<u32>,
};

/**
 * field buffer helpers
 */

fn get_field_line_byte(line: u32, offset: u32) -> u32 {
    let idx = line * BYTES_PER_LINE + offset;

    let word = field.bytes[idx >> 2u];
    return (word >> ((idx & 0x3u) * 8u)) & 0xffu;
}

fn calcTeletextEnabled(videoULAControlReg: u32) -> bool {
    return (videoULAControlReg & 0x02u) != 0u;
}

fn calcIsHighFreq(videoULAControlReg: u32) -> bool {
    return (videoULAControlReg & 0x10u) != 0u;
}

fn calcHPixelsPerChar(videoULAControlReg: u32) -> u32 {
    if calcTeletextEnabled(videoULAControlReg) {
        return 12u;
    } else if calcIsHighFreq(videoULAControlReg) {
        return 8u;
    } else {
        return 16u;
    }
}

fn calcDisplayXOffset(isTeletext: bool) -> u32 {
    if isTeletext {
        return TELETEXT_DELAY_OFFSET;
    } else {
        return 0u;
    }
}

fn calcBackPorch(
    r0_horizontalTotal: u32,
    r2_horizontalSyncPos: u32,
    r3_syncWidth: u32,
) -> u32 {
    let hSyncWidth = r3_syncWidth & 0x0fu;
    return r0_horizontalTotal + 1u - (r2_horizontalSyncPos + hSyncWidth);
}

fn calcBaseLineOffset(
    r0_horizontalTotal: u32,
    r2_horizontalSyncPos: u32,
    r3_syncWidth: u32,
    videoULAControlReg: u32,
) -> u32 {
    let backPorch = calcBackPorch(
        r0_horizontalTotal,
        r2_horizontalSyncPos,
        r3_syncWidth,
    );
    let charWidth = calcHPixelsPerChar(videoULAControlReg);
    return backPorch * charWidth;
}

fn calcDisplayWidth(
    r1_horizontalDisplayed: u32,
    videoULAControlReg: u32,
) -> u32 {
    return r1_horizontalDisplayed * calcHPixelsPerChar(videoULAControlReg);
}

/**
 * metrics buffer
 */

struct MetricsBuf {
    num_lines: u32,
    flags: u32,
    top: u32,
    bottom: u32,
    min_left: u32,
    max_right: u32,
};

const METRIC_FLAG_HAS_TELETEXT      = 0x01;
const METRIC_FLAG_HAS_HIRES  = 0x02;

/**
 * metrics compute
 */

@compute @workgroup_size(1)
fn metrics_main() {
    metrics.num_lines = arrayLength(&field.bytes) * 4u / BYTES_PER_LINE;
    metrics.flags = 0u;
    metrics.min_left = 0xFFFFFFFFu;
    metrics.max_right = 0u;

    for (var line = 0u; line < metrics.num_lines; line++) {
        let visible = get_field_line_byte(line, 0u);
        let videoULAControlReg = get_field_line_byte(line, 113u);
        if visible != 0u {
            if metrics.flags == 0u { // metrics not set yet
                metrics.top = line;
            }
            let isTeletext = calcTeletextEnabled(videoULAControlReg);
            
            if isTeletext {
                metrics.flags |= METRIC_FLAG_HAS_TELETEXT;
            } else {
                metrics.flags |= METRIC_FLAG_HAS_HIRES;
            }

            metrics.bottom = line + 1;

            let r0_horizontalTotal = get_field_line_byte(line, 104u);
            let r1_horizontalDisplayed = get_field_line_byte(line, 105u);
            let r2_horizontalSyncPos = get_field_line_byte(line, 106u);
            let r3_syncWidth = get_field_line_byte(line, 107u);

            let left = calcBaseLineOffset(
                r0_horizontalTotal,
                r2_horizontalSyncPos,
                r3_syncWidth,
                videoULAControlReg,
            );
            let displayedLeft = left + calcDisplayXOffset(isTeletext);
            let displayedWidth = calcDisplayWidth(
                r1_horizontalDisplayed,
                videoULAControlReg,
            );
            let displayedRight = displayedLeft + displayedWidth;

            metrics.min_left = min(metrics.min_left, displayedLeft);
            metrics.max_right = max(metrics.max_right, displayedRight);
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

    if metrics.flags != METRIC_FLAG_HAS_HIRES {
        return vec4f(0.5, 0.0, 0.0, 1.0);
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
