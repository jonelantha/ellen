// bindings

@group(0) @binding(0) var<storage, read> field : FieldBuf;
@group(0) @binding(1) var<storage, read_write> line_metrics : LineMetricsBuf;
@group(0) @binding(2) var<storage, read_write> frame_metrics : FrameMetricsBuf;

// field buffer

const BYTES_PER_LINE = 122u;

struct FieldBuf {
    bytes: array<u32>,
};

fn get_field_line_byte(line: u32, offset: u32) -> u32 {
    let idx = line * BYTES_PER_LINE + offset;

    let word = field.bytes[idx >> 2u];
    return (word >> ((idx & 0x3u) * 8u)) & 0xffu;
}

// metrics buffers

struct LineMetrics {
    bm_left: u32,
    width: u32,
}

struct LineMetricsBuf {
    lines: array<LineMetrics>,
};

struct FrameMetricsBuf {
    num_lines: u32,
    bm_display_origin_x: u32,
    bm_display_origin_y: u32,
};

// constants

const CANVAS_WIDTH = 640u;
const CANVAS_HEIGHT = 512u;

// shaders

@compute @workgroup_size(1)
fn metrics_main() {
    let num_lines = arrayLength(&field.bytes) * 4u / BYTES_PER_LINE;

    var bm_min_y: u32 = 0xffffffffu;
    var bm_max_y: u32 = 0u;
    var bm_min_x: u32 = 0xffffffffu;
    var bm_max_x: u32 = 0u;

    for (var line = 0u; line < num_lines; line++) {
        let line_type = get_field_line_byte(line, 0u);
        if line_type == 0u { continue; };

        let r0_horizontal_total = get_field_line_byte(line, 104u);
        let r1_horizontal_displayed = get_field_line_byte(line, 105u);
        let r2_horizontal_sync_pos = get_field_line_byte(line, 106u);
        let r3_sync_width = get_field_line_byte(line, 107u);
        let video_ula_control_reg = get_field_line_byte(line, 113u);

        let bm_left = calc_line_bm_left(
            r0_horizontal_total,
            r2_horizontal_sync_pos,
            r3_sync_width,
            video_ula_control_reg,
        );

        let width = calc_line_width(
            r1_horizontal_displayed,
            video_ula_control_reg,
        );

        line_metrics.lines[line] = LineMetrics(bm_left, width);

        bm_min_x = min(bm_min_x, bm_left);
        bm_max_x = max(bm_max_x, bm_left + width);
        bm_min_y = min(bm_min_y, line);
        bm_max_y = max(bm_max_y, line + 1);
    }
    
    frame_metrics.num_lines = num_lines;
    if bm_min_y != 0xffffffffu {
        frame_metrics.bm_display_origin_x = calc_display_origin(11 * 16, /* char x char width */ bm_min_x, bm_max_x, CANVAS_WIDTH);
        frame_metrics.bm_display_origin_y = calc_display_origin(31 * 2, bm_min_y * 2, bm_max_y * 2, CANVAS_HEIGHT);
    }
}

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

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4f {
    let display_x = u32(input.crt.x);
    let display_y = u32(input.crt.y);

    let line = (frame_metrics.bm_display_origin_y + display_y) / 2;
    
    if line >= frame_metrics.num_lines {
        return vec4f(0.0, 0.0, 1.0, 1.0);
    }

    let line_type = get_field_line_byte(line, 0u);
    if line_type == 0u {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }

    if line_type != 1u {
        // cursor could be visible
        return vec4f(0.0, 1.0, 0.0, 1.0);
    }

    let video_ula_control_reg = get_field_line_byte(line, 113u);
    
    if calc_teletext_enabled(video_ula_control_reg) {
        return vec4f(0.0, 1.0, 1.0, 1.0);
    }
    
    let r1_horizontal_displayed = get_field_line_byte(line, 105u);
    let is_high_freq = calc_is_high_freq(video_ula_control_reg);
    let bm_line_left = line_metrics.lines[line].bm_left;
    
    let char_index_and_pixel = get_char_index_and_pixel(
        display_x,
        r1_horizontal_displayed,
        bm_line_left,
        is_high_freq
    );
    
    if char_index_and_pixel.char_index < 0 {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }

    let byte = get_field_line_byte(line, 1u + u32(char_index_and_pixel.char_index));

    let num_cols = calc_num_colours(video_ula_control_reg);

    if num_cols == 4u && !is_high_freq {
        let palette_idx = extract_palette_index_4col(byte, char_index_and_pixel.pixel / 4u);
        
        let flash = (video_ula_control_reg & 1u) != 0u;

        let colour_idx = get_colour_index_from_palette_index(line, palette_idx, flash);

        return colour_idx_to_rgb(colour_idx);
    } else {
        if byte > 0 {
            return vec4f(0.0, 1.0, 0.0, 1.0);
        } else {
            return vec4f(1.0, 0.0, 0.0, 1.0);
        }
    }
}

// ula control helpers

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
    return select(16u, 8u, calc_is_high_freq(video_ula_control_reg));
}

// line metric calcs

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

fn calc_line_bm_left(
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
    return back_porch * calc_h_pixels_per_char(video_ula_control_reg);
}

fn calc_line_width(
    r1_horizontal_displayed: u32,
    video_ula_control_reg: u32,
) -> u32 {
    return r1_horizontal_displayed * calc_h_pixels_per_char(video_ula_control_reg);
}

// screen metric calcs

fn calc_display_origin(bm_default_origin: u32, bm_min: u32, bm_max: u32, canvas_size: u32) -> u32 {
    if bm_max - bm_min <= canvas_size {
        if bm_min < bm_default_origin {
            return bm_min;
        } else if bm_max > bm_default_origin + canvas_size {
            return bm_max - canvas_size;
        }
    }

    return bm_default_origin;
}

// char and pixel calcs

struct ByteIndexAndPixel {
    char_index: i32,     // negative values indicate failure
    pixel: u32,
}

fn get_char_index_and_pixel(
    display_x: u32,
    r1_horizontal_displayed: u32,
    bm_line_left: u32,
    is_high_freq: bool
) -> ByteIndexAndPixel {
    let bm_x = frame_metrics.bm_display_origin_x + display_x;

    if bm_x < bm_line_left {
        return ByteIndexAndPixel(-1, 0u);
    }

    let data_x = bm_x - bm_line_left;

    let char_width = select(16u, 8u, is_high_freq);
    
    let char_index = data_x / char_width;
    let pixel = data_x % char_width;
    
    if char_index >= r1_horizontal_displayed {
        return ByteIndexAndPixel(-2, 0u);
    }

    return ByteIndexAndPixel(i32(char_index), pixel);
}

// palette

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

fn get_colour_index_from_palette_index(line: u32, palette_idx: u32, flash: bool) -> u32 {
    let byte_offset = 114u + (palette_idx >> 1u);
    let byte_val = get_field_line_byte(line, byte_offset);
    
    var colour_idx: u32;
    if (palette_idx & 1u) == 0u {
        colour_idx = byte_val & 0x0fu;
    } else {
        colour_idx = (byte_val >> 4u) & 0x0fu;
    }

    if colour_idx > 7u {
        colour_idx &= 7u;

        if flash { colour_idx ^= 7u; }
    }

    return colour_idx;
}

fn colour_idx_to_rgb(colour_idx: u32) -> vec4f {
    switch colour_idx {
        case 0u: { return vec4f(0.0, 0.0, 0.0, 1.0); }      // Black
        case 1u: { return vec4f(1.0, 0.0, 0.0, 1.0); }      // Red
        case 2u: { return vec4f(0.0, 1.0, 0.0, 1.0); }      // Green
        case 3u: { return vec4f(1.0, 1.0, 0.0, 1.0); }      // Yellow
        case 4u: { return vec4f(0.0, 0.0, 1.0, 1.0); }      // Blue
        case 5u: { return vec4f(1.0, 0.0, 1.0, 1.0); }      // Magenta
        case 6u: { return vec4f(0.0, 1.0, 1.0, 1.0); }      // Cyan
        default: { return vec4f(1.0, 1.0, 1.0, 1.0); }      // White
    }
}