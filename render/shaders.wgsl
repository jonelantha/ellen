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

const CANVAS_WIDTH: i32 = 640;
const CANVAS_HEIGHT: i32 = 512;

const TELETEXT_FRAME_WIDTH: i32 = 480;
const TELETEXT_FRAME_HEIGHT: i32 = 500;
const NON_TELETEXT_FRAME_WIDTH: i32 = 640;
const NON_TELETEXT_FRAME_HEIGHT: i32 = 512;

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

fn calc_teletext_enabled(video_ula_control_reg: u32) -> bool {
    return (video_ula_control_reg & 0x02u) != 0u;
}

fn calc_is_high_freq(video_ula_control_reg: u32) -> bool {
    return (video_ula_control_reg & 0x10u) != 0u;
}

fn calc_h_pixels_per_char(video_ula_control_reg: u32) -> u32 {
    if calc_teletext_enabled(video_ula_control_reg) {
        return 12u;
    } else if calc_is_high_freq(video_ula_control_reg) {
        return 8u;
    } else {
        return 16u;
    }
}

fn calc_display_x_offset(is_teletext: bool) -> u32 {
    if is_teletext {
        return TELETEXT_DELAY_OFFSET;
    } else {
        return 0u;
    }
}

fn calc_back_porch(
    r0_horizontal_total: u32,
    r2_horizontal_sync_pos: u32,
    r3_sync_width: u32,
) -> u32 {
    let h_sync_width = r3_sync_width & 0x0fu;
    return r0_horizontal_total + 1u - (r2_horizontal_sync_pos + h_sync_width);
}

fn calc_base_line_offset(
    r0_horizontal_total: u32,
    r2_horizontal_sync_pos: u32,
    r3_sync_width: u32,
    video_ula_control_reg: u32,
) -> u32 {
    let back_porch = calc_back_porch(
        r0_horizontal_total,
        r2_horizontal_sync_pos,
        r3_sync_width,
    );
    let char_width = calc_h_pixels_per_char(video_ula_control_reg);
    return back_porch * char_width;
}

fn calc_display_width(
    r1_horizontal_displayed: u32,
    video_ula_control_reg: u32,
) -> u32 {
    return r1_horizontal_displayed * calc_h_pixels_per_char(video_ula_control_reg);
}

fn get_render_start(first: i32, total_displayed: i32, max_displayed: i32) -> i32 {
    if total_displayed <= max_displayed {
        if first < 0 {
            return 0;
        } else if first + total_displayed > max_displayed {
            return max_displayed - total_displayed;
        }
    }

    return first;
}

fn centring_offset(canvas_size: i32, frame_size: i32) -> i32 {
    return (canvas_size - frame_size) / 2;
}

fn calc_metric_x_offset() -> i32 {
    let teletext = (metrics.flags & METRIC_FLAG_HAS_TELETEXT) != 0u;
    var frame_width: i32;
    if teletext {
        frame_width = TELETEXT_FRAME_WIDTH;
    } else {
        frame_width = NON_TELETEXT_FRAME_WIDTH;
    }

    var h_sync_char_width: i32;
    if teletext {
        h_sync_char_width = 12;
    } else {
        h_sync_char_width = 16;
    }

    let h_sync_left_border: i32 = 11 * h_sync_char_width;
    let min_left = i32(metrics.min_left);
    let displayed_width = i32(metrics.max_right - metrics.min_left);
    let render_start = get_render_start(
        min_left - h_sync_left_border,
        displayed_width,
        frame_width,
    );
    let centre = centring_offset(CANVAS_WIDTH, frame_width);

    return -min_left + render_start + centre;
}

fn calc_metric_y_offset() -> i32 {
    let teletext = (metrics.flags & METRIC_FLAG_HAS_TELETEXT) != 0u;
    var frame_height: i32;
    if teletext {
        frame_height = TELETEXT_FRAME_HEIGHT;
    } else {
        frame_height = NON_TELETEXT_FRAME_HEIGHT;
    }
    let v_sync_top_border = 31;
    let top = i32(metrics.top);
    let render_start = get_render_start(
        (top - v_sync_top_border) * 2,
        i32(metrics.bottom - metrics.top) * 2,
        frame_height,
    );
    let centre = centring_offset(CANVAS_HEIGHT, frame_height);

    return -top * 2 + render_start + centre;
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
    x_offset: i32,
    y_offset: i32,
};

const METRIC_FLAG_HAS_TELETEXT  = 0x01;
const METRIC_FLAG_HAS_HIRES     = 0x02;

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
        let video_ula_control_reg = get_field_line_byte(line, 113u);
        if visible != 0u {
            if metrics.flags == 0u { // metrics not set yet
                metrics.top = line;
            }
            let is_teletext = calc_teletext_enabled(video_ula_control_reg);
            
            if is_teletext {
                metrics.flags |= METRIC_FLAG_HAS_TELETEXT;
            } else {
                metrics.flags |= METRIC_FLAG_HAS_HIRES;
            }

            metrics.bottom = line + 1;

            let r0_horizontal_total = get_field_line_byte(line, 104u);
            let r1_horizontal_displayed = get_field_line_byte(line, 105u);
            let r2_horizontal_sync_pos = get_field_line_byte(line, 106u);
            let r3_sync_width = get_field_line_byte(line, 107u);

            let left = calc_base_line_offset(
                r0_horizontal_total,
                r2_horizontal_sync_pos,
                r3_sync_width,
                video_ula_control_reg,
            );
            let displayed_left = left + calc_display_x_offset(is_teletext);
            let displayed_width = calc_display_width(
                r1_horizontal_displayed,
                video_ula_control_reg,
            );
            let displayed_right = displayed_left + displayed_width;

            metrics.min_left = min(metrics.min_left, displayed_left);
            metrics.max_right = max(metrics.max_right, displayed_right);
        }
    }

    if metrics.flags != 0u {
        metrics.x_offset = calc_metric_x_offset();
        metrics.y_offset = calc_metric_y_offset();
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
