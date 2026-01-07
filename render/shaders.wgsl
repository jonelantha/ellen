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

const CANVAS_WIDTH = 640u;
const CANVAS_HEIGHT = 512u;

const TELETEXT_FRAME_WIDTH = 480u;
const TELETEXT_FRAME_HEIGHT = 500u;
const NON_TELETEXT_FRAME_WIDTH = 640u;
const NON_TELETEXT_FRAME_HEIGHT = 512u;

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

fn get_field_line_u32(line: u32, offset: u32) -> u32 {
    return (get_field_line_byte(line, offset) << 0u) |
           (get_field_line_byte(line, offset + 1u) << 8u) |
           (get_field_line_byte(line, offset + 2u) << 16u) |
           (get_field_line_byte(line, offset + 3u) << 24u);
}

fn calc_teletext_enabled(video_ula_control_reg: u32) -> bool {
    return (video_ula_control_reg & 0x02u) != 0u;
}

fn calc_is_high_freq(video_ula_control_reg: u32) -> bool {
    return (video_ula_control_reg & 0x10u) != 0u;
}

fn calc_num_colours(video_ula_control_reg: u32) -> u32 {
    let idx = (video_ula_control_reg & 0x1cu) >> 2u;
    let lookup = array<u32, 8>(
        16u,
        4u,
        2u,
        0u,
        0u,
        16u,
        4u,
        2u,
    );
    return lookup[idx];
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
    let sync_end = r2_horizontal_sync_pos + h_sync_width;
    if sync_end >= r0_horizontal_total + 1 {
        return 0u;
    } else {
        return r0_horizontal_total + 1 - sync_end;
    }
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

fn get_render_start(first: i32, total_displayed: u32, max_displayed: u32) -> i32 {
    if total_displayed <= max_displayed {
        if first < 0 {
            return 0;
        } else if first + i32(total_displayed) > i32(max_displayed) {
            return i32(max_displayed) - i32(total_displayed);
        }
    }

    return first;
}

fn centring_offset(canvas_size: u32, frame_size: u32) -> i32 {
    return (i32(canvas_size) - i32(frame_size)) / 2;
}

fn calc_metric_x_offset() -> i32 {
    let teletext = (metrics.flags & METRIC_FLAG_HAS_TELETEXT) != 0u;
    let frame_width = select(NON_TELETEXT_FRAME_WIDTH, TELETEXT_FRAME_WIDTH, teletext);
    
    let h_sync_char_width = select(16, 12, teletext);

    let h_sync_left_border: i32 = 11 * h_sync_char_width;
    let min_left = i32(metrics.min_left);
    let displayed_width = metrics.max_right - metrics.min_left;
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
    let frame_height = select(NON_TELETEXT_FRAME_HEIGHT, TELETEXT_FRAME_HEIGHT, teletext);
    
    let v_sync_top_border = 31;
    let top = i32(metrics.top);
    let render_start = get_render_start(
        (top - v_sync_top_border) * 2,
        (metrics.bottom - metrics.top) * 2,
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
    
    for (var line = 0u; line < metrics.num_lines; line++) {
        let line_type = get_field_line_byte(line, 0u);
        if line_type != 0u {
            let r0_horizontal_total = get_field_line_byte(line, 104u);
            let r1_horizontal_displayed = get_field_line_byte(line, 105u);
            let r2_horizontal_sync_pos = get_field_line_byte(line, 106u);
            let r3_sync_width = get_field_line_byte(line, 107u);
            let video_ula_control_reg = get_field_line_byte(line, 113u);
            let is_teletext = calc_teletext_enabled(video_ula_control_reg);

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

            if metrics.flags == 0u { // metrics not set yet
                metrics.top = line;

                metrics.min_left = displayed_left;
                metrics.max_right = displayed_right;
            } else {
                metrics.min_left = min(metrics.min_left, displayed_left);
                metrics.max_right = max(metrics.max_right, displayed_right);
            }
            metrics.bottom = line + 1;

            if is_teletext {
                metrics.flags |= METRIC_FLAG_HAS_TELETEXT;
            } else {
                metrics.flags |= METRIC_FLAG_HAS_HIRES;
            }
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
 * fragment - helpers
 */

fn extract_palette_index_4col(byte_val: u32, pixel_in_byte: u32) -> u32 {
    switch pixel_in_byte {
        case 0u: {
            // bits 7,5,3,1
            return ((byte_val & 0x80u) >> 4u) |
                   ((byte_val & 0x20u) >> 3u) |
                   ((byte_val & 0x08u) >> 2u) |
                   ((byte_val & 0x02u) >> 1u);
        }
        case 1u: {
            // bits 6,4,2,0
            return ((byte_val & 0x40u) >> 3u) |
                   ((byte_val & 0x10u) >> 2u) |
                   ((byte_val & 0x04u) >> 1u) |
                   (byte_val & 0x01u);
        }
        case 2u: {
            // bits 5,3,1,H (H=1)
            return ((byte_val & 0x20u) >> 2u) |
                   ((byte_val & 0x08u) >> 1u) |
                   ((byte_val & 0x02u) >> 0u) |
                   1u;
        }
        default: { // 3u
            // bits 4,2,0,H (H=1)
            return ((byte_val & 0x10u) >> 1u) |
                   ((byte_val & 0x04u) >> 0u) |
                   ((byte_val & 0x01u) << 1u) |
                   1u;
        }
    }
}

fn get_color_from_paletteindex(palette_lo: u32, palette_hi: u32, palette_idx: u32, flash: bool) -> vec3f {
    let palette_val = select(
        (palette_lo >> (palette_idx * 4u)) & 0x0fu,
        (palette_hi >> ((palette_idx - 8u) * 4u)) & 0x0fu,
        palette_idx >= 8u
    );
    
    var color_idx = palette_val;
    
    // Handle flashing colors (values >= 8)
    if color_idx > 7u {
        color_idx &= 7u;
        // If flash bit is set (bit 0 of control register), invert
        if flash {
            color_idx ^= 7u;
        }
    }

    return color_to_rgb(color_idx);
}

fn color_to_rgb(color_idx: u32) -> vec3f {
    switch color_idx {
        case 0u: { return vec3f(0.0, 0.0, 0.0); }      // Black
        case 1u: { return vec3f(1.0, 0.0, 0.0); }      // Red
        case 2u: { return vec3f(0.0, 1.0, 0.0); }      // Green
        case 3u: { return vec3f(1.0, 1.0, 0.0); }      // Yellow
        case 4u: { return vec3f(0.0, 0.0, 1.0); }      // Blue
        case 5u: { return vec3f(1.0, 0.0, 1.0); }      // Magenta
        case 6u: { return vec3f(0.0, 1.0, 1.0); }      // Cyan
        default: { return vec3f(1.0, 1.0, 1.0); }      // White
    }
}

/**
 * fragment - render
 */

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4f {
    
    if metrics.flags != METRIC_FLAG_HAS_HIRES {
        return vec4f(0.5, 0.0, 0.0, 1.0);
    }

    let y = (i32(input.crt.y) - metrics.y_offset) / 2;
    if y < 0 || y >= i32(metrics.num_lines) {
        return vec4f(0.0, 0.0, 1.0, 1.0);
    }

    let line= u32(y);
    let line_type = get_field_line_byte(line, 0u);
    if line_type == 0u {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }

    if line_type != 1u {
        // cursor could be visible
        return vec4f(0.0, 1.0, 0.0, 1.0);
    }

    let byte_index_and_pixel = get_byte_index_and_pixel(line, u32(input.crt.x));
    if byte_index_and_pixel.byte_index < 0 {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }

    let video_ula_control_reg = get_field_line_byte(line, 113u);
    let num_cols = calc_num_colours(video_ula_control_reg);
    let is_high_freq = calc_is_high_freq(video_ula_control_reg);
        
    let byte = get_field_line_byte(line, u32(byte_index_and_pixel.byte_index) + 1u);
    
    if num_cols == 4u && !is_high_freq {
        
        // Extract palette index for this pixel
        let palette_idx = extract_palette_index_4col(byte, byte_index_and_pixel.pixel / 4u);
        
        // Pack into two u32 values for 16 palette entries of 4 bits each
        let palette_lo = get_field_line_u32(line, 114u);
        let palette_hi = get_field_line_u32(line, 118u);
        
        let flash = (video_ula_control_reg & 1u) != 0u;

        // Get color from palette
        let rgb = get_color_from_paletteindex(palette_lo, palette_hi, palette_idx, flash);
        
        return vec4f(rgb, 1.0);
    } else {
        if byte > 0 {
            return vec4f(0.0, 1.0, 0.0, 1.0);
        } else {
            return vec4f(1.0, 0.0, 0.0, 1.0);
        }
    }
}

struct ByteIndexAndPixel {
    byte_index: i32,     // negative values indicate failure
    pixel: u32,
}

fn get_byte_index_and_pixel(line: u32, crt_x: u32) -> ByteIndexAndPixel {
    let r0_horizontal_total = get_field_line_byte(line, 104u);
    let r1_horizontal_displayed = get_field_line_byte(line, 105u);
    let r2_horizontal_sync_pos = get_field_line_byte(line, 106u);
    let r3_sync_width = get_field_line_byte(line, 107u);
    let video_ula_control_reg = get_field_line_byte(line, 113u);

    let left = calc_base_line_offset(
        r0_horizontal_total,
        r2_horizontal_sync_pos,
        r3_sync_width,
        video_ula_control_reg,
    );

    let x = i32(crt_x) - metrics.x_offset - i32(left);

    if x < 0 {
        return ByteIndexAndPixel(-1, 0u);
    }
    let char_width = select(16u, 8u, calc_is_high_freq(video_ula_control_reg));
    
    let byte_index = u32(x) / char_width;
    let pixel = u32(x) % char_width;
    
    if byte_index >= r1_horizontal_displayed {
        return ByteIndexAndPixel(-2, 0u);
    }

    return ByteIndexAndPixel(i32(byte_index), pixel);
}
