// bindings

@group(0) @binding(0) var<storage, read> field : FieldBuf;
@group(0) @binding(1) var<storage, read_write> frame_metrics : FrameMetricsBuf;

// field buffer

const DWORDS_PER_LINE = 29;

const FLAG_DISPLAYED = 0x0001;
const FLAG_HAS_BYTES = 0x0002;
const FLAG_INVALID_RANGE = 0x0004;
const FLAG_CURSOR_RASTER_EVEN = 0x0010;
const FLAG_CURSOR_RASTER_ODD = 0x0020;

const FLAG_ULA_FLASH = 0x0100;
const FLAG_ULA_TELETEXT = 0x0200;
const FLAG_ULA_HIGH_FREQ = 0x1000;
const FLAG_ULA_PIXEL_SEL_MASK = 0x1c00;
const FLAG_ULA_HIGH_FREQ_80 = 0x1c00;
const FLAG_ULA_HIGH_FREQ_40 = 0x1800;
const FLAG_ULA_HIGH_FREQ_20 = 0x1400;
const FLAG_ULA_HIGH_FREQ_10 = 0x1000;
const FLAG_ULA_LOW_FREQ_80 = 0x0c00;
const FLAG_ULA_LOW_FREQ_40 = 0x0800;
const FLAG_ULA_LOW_FREQ_20 = 0x0400;
const FLAG_ULA_LOW_FREQ_10 = 0x0000;
const FLAG_ULA_CURSOR_SEGMENT_3_4 = 0x2000;
const FLAG_ULA_CURSOR_SEGMENT_2 = 0x4000;
const FLAG_ULA_CURSOR_SEGMENT_1 = 0x8000;
const OFFSET_FLAGS_AND_METRICS = 0;
const OFFSET_CURSOR = 1;
const OFFSET_PALETTE_START = 2;
const OFFSET_DATA_START = 4;
struct FieldBuf {
    bytes: array<u32>,
};

struct FlagsAndMetrics {
    flags: u32,
    total_chars: u32,
    back_porch: u32,
};

fn get_line_dword(line: u32, offset: u32) -> u32 {
    return field.bytes[line * DWORDS_PER_LINE + offset];
}

fn get_line_flags_and_metrics(line: u32) -> FlagsAndMetrics {
    let dword = get_line_dword(line, OFFSET_FLAGS_AND_METRICS);

    return FlagsAndMetrics(
        dword & 0xffff,
        extract_byte(dword, 2),
        extract_byte(dword, 3)
    );
}

fn get_field_line_palette(line: u32, index: u32) -> u32 {
    let palette_dword = get_line_dword(line, OFFSET_PALETTE_START + (index >> 3));

    return extract_nibble(palette_dword, index & 0x07);
}

fn get_field_line_char(line: u32, offset: u32) -> u32 {
    let dword = get_line_dword(line, OFFSET_DATA_START + (offset >> 2));

    return extract_byte(dword, offset & 0x03);
}

// metrics buffers

struct FrameMetricsBuf {
    num_lines: u32,
    bm_display_origin_x: u32,
    bm_display_origin_y: u32,
};

// constants

const CANVAS_WIDTH = 640;
const CANVAS_HEIGHT = 512;

// shaders

@compute @workgroup_size(1)
fn metrics_main() {
    let num_lines = arrayLength(&field.bytes) / DWORDS_PER_LINE;

    var bm_min_y = 0xffffffffu;
    var bm_max_y = 0u;
    var bm_min_x = 0xffffffffu;
    var bm_max_x = 0u;

    for (var line = 0u; line < num_lines; line++) {
        let flags_and_metrics = get_line_flags_and_metrics(line);

        let flags = flags_and_metrics.flags;
        
        if (flags & FLAG_DISPLAYED) == 0 { continue; };

        let char_shift = get_char_shift(flags);

        let bm_left = flags_and_metrics.back_porch << char_shift;
        let width = flags_and_metrics.total_chars << char_shift;

        bm_min_x = min(bm_min_x, bm_left);
        bm_max_x = max(bm_max_x, bm_left + width);
        bm_min_y = min(bm_min_y, line);
        bm_max_y = max(bm_max_y, line + 1);
    }
    
    frame_metrics.num_lines = num_lines;
    if bm_min_y != 0xffffffff {
        frame_metrics.bm_display_origin_x = calc_display_origin(176, bm_min_x, bm_max_x, CANVAS_WIDTH);
        frame_metrics.bm_display_origin_y = calc_display_origin(62, bm_min_y * 2, bm_max_y * 2, CANVAS_HEIGHT);
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
    let odd_line = ((frame_metrics.bm_display_origin_y + display_y) & 1) != 0;
    
    if line >= frame_metrics.num_lines {
        return vec4f(0.0, 0.0, 1.0, 1.0);
    }

    let flags_and_metrics = get_line_flags_and_metrics(line);

    let flags = flags_and_metrics.flags;
    
    if ((flags & FLAG_DISPLAYED) == 0) {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }

    if ((flags & FLAG_ULA_TELETEXT) != 0) {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }

    let char_index_and_pixel = get_char_index_and_pixel(display_x, flags_and_metrics);
    
    if char_index_and_pixel.out_of_bounds {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }

    var colour_index = get_hires_colour_index(flags, line, char_index_and_pixel);

    if is_cursor(flags, line, odd_line, char_index_and_pixel.char_index) {
        colour_index ^= 7;
    }

    return colour_index_to_rgb(colour_index);
}

fn get_hires_colour_index(flags: u32, line: u32, char_index_and_pixel: ByteIndexAndPixel) -> u32 {
    if ((flags & FLAG_HAS_BYTES) == 0) {
        return 0;
    } else {
        let byte = get_field_line_char(line,  char_index_and_pixel.char_index);

        let palette_index = extract_palette_index(byte, char_index_and_pixel.pixel);

        let flash = (flags & FLAG_ULA_FLASH) != 0;

        return get_colour_index_from_palette_index(line, palette_index, flash);
    }
}

// ula control helpers

fn get_char_shift(flags: u32) -> u32 {
    return select(4u, 3u, (flags & FLAG_ULA_HIGH_FREQ) != 0);
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
    out_of_bounds: bool,
    char_index: u32,
    pixel: u32,
}

fn get_char_index_and_pixel(
    display_x: u32,
    flags_and_metrics: FlagsAndMetrics
) -> ByteIndexAndPixel {
    let char_shift = get_char_shift(flags_and_metrics.flags);

    let bm_x = frame_metrics.bm_display_origin_x + display_x;

    let bm_line_left = flags_and_metrics.back_porch << char_shift;
    
    if bm_x < bm_line_left {
        return ByteIndexAndPixel(true, 1, 0);
    }

    let logical_x = bm_x - bm_line_left;

    let char_index = logical_x >> char_shift;
    if char_index >= flags_and_metrics.total_chars {
        return ByteIndexAndPixel(true, 2, 0);
    }

    let pixel = pixel_in_char(logical_x, flags_and_metrics.flags);

    return ByteIndexAndPixel(false, char_index, pixel);
}

fn pixel_in_char(x: u32, flags: u32) -> u32 {
    // bits = bits in mask
    // px/byte = 2^bits
    // depth = bits/pixel = 8 / px/byte
    // 2^(num rh ignored bits) = px width
    switch flags & FLAG_ULA_PIXEL_SEL_MASK {                    // bits | px/byte | depth | px width | modes
        case FLAG_ULA_HIGH_FREQ_10: { return 0; }               // 0    | 1       | ?     | 8        | ?
        case FLAG_ULA_HIGH_FREQ_20: { return (x & 0x04) >> 2; } // 1    | 2       | 4     | 4        | 2
        case FLAG_ULA_HIGH_FREQ_40: { return (x & 0x06) >> 1; } // 2    | 4       | 2     | 2        | 1
        case FLAG_ULA_HIGH_FREQ_80: { return x & 0x07; }        // 3    | 8       | 1     | 1        | 0,3
        case FLAG_ULA_LOW_FREQ_10:  { return (x & 0x08) >> 3; } // 1    | 2       | 4     | 8        | '8'
        case FLAG_ULA_LOW_FREQ_20:  { return (x & 0x0c) >> 2; } // 2    | 4       | 2     | 4        | 5
        case FLAG_ULA_LOW_FREQ_40:  { return (x & 0x0e) >> 1; } // 3    | 8       | 1     | 2        | 4,6
        default: /*LOW_FREQ_80*/    { return x & 0x0f; }        // 4    | ?       | ?     | ?        | N/A
    }
}

// palette

fn extract_palette_index(byte_val: u32, index: u32) -> u32 {
    switch index {
        case 0: { // bits 7,5,3,1
            return ((byte_val & 0x80) >> 4) |
                   ((byte_val & 0x20) >> 3) |
                   ((byte_val & 0x08) >> 2) |
                   ((byte_val & 0x02) >> 1);
        }
        case 1: { // bits 6,4,2,0
            return ((byte_val & 0x40) >> 3) |
                   ((byte_val & 0x10) >> 2) |
                   ((byte_val & 0x04) >> 1) |
                   (byte_val & 0x01);
        }
        case 2: { // bits 5,3,1,H (H=1)
            return ((byte_val & 0x20) >> 2) |
                   ((byte_val & 0x08) >> 1) |
                   (byte_val & 0x02) |
                   1;
        }
        case 3: { // bits 4,2,0,H (H=1)
            return ((byte_val & 0x10) >> 1) |
                   (byte_val & 0x04) |
                   ((byte_val & 0x01) << 1) |
                   1;
        }
        case 4: { //bits 3,1,H,H (H=1)
            return (byte_val & 0x08) |
                   ((byte_val & 0x02) << 1) |
                   3;
        }
        case 5: { // bits 2,0,H,H (H=1)
            return ((byte_val & 0x04) << 1) |
                   ((byte_val & 0x01) << 2) |
                   3;
        }
        case 6: { // bits 1,H,H,H (H=1)
            return ((byte_val & 0x02) << 2) |
                   7;
        }
        default: { // 7, bits 0,H,H,H (H=1)
            return ((byte_val & 0x01) << 3) |
                   7;
        }
    }
}

fn get_colour_index_from_palette_index(line: u32, index: u32, flash: bool) -> u32 {
    var colour_index = get_field_line_palette(line, index);
    
    if colour_index > 7 {
        colour_index &= 7;

        if flash { colour_index ^= 7; }
    }

    return colour_index;
}

fn colour_index_to_rgb(colour_index: u32) -> vec4f {
    switch colour_index {
        case 0: { return vec4f(0.0, 0.0, 0.0, 1.0); }   // Black
        case 1: { return vec4f(1.0, 0.0, 0.0, 1.0); }   // Red
        case 2: { return vec4f(0.0, 1.0, 0.0, 1.0); }   // Green
        case 3: { return vec4f(1.0, 1.0, 0.0, 1.0); }   // Yellow
        case 4: { return vec4f(0.0, 0.0, 1.0, 1.0); }   // Blue
        case 5: { return vec4f(1.0, 0.0, 1.0, 1.0); }   // Magenta
        case 6: { return vec4f(0.0, 1.0, 1.0, 1.0); }   // Cyan
        default: { return vec4f(1.0, 1.0, 1.0, 1.0); }  // White
    }
}

// cursor

fn is_cursor(flags: u32, line: u32, odd_line: bool, char_x: u32) -> bool {
    if !odd_line && (flags & FLAG_CURSOR_RASTER_EVEN) == 0 { return false; }

    if odd_line && (flags & FLAG_CURSOR_RASTER_ODD) == 0 { return false; }

    let cursor_start= extract_byte(get_line_dword(line, OFFSET_CURSOR), 0);

    if char_x < cursor_start {
        return false;
    } else if char_x < cursor_start + 1 {
        return (flags & FLAG_ULA_CURSOR_SEGMENT_1) != 0;
    } else if char_x < cursor_start + 2 {
        return (flags & FLAG_ULA_CURSOR_SEGMENT_2) != 0;
    } else if char_x < cursor_start + 4 {
        return (flags & FLAG_ULA_CURSOR_SEGMENT_3_4) != 0;
    } else {
        return false;
    }
}

// helpers

fn extract_byte(value: u32, byte_index: u32) -> u32 {
    return (value >> (byte_index * 8)) & 0xff;
}

fn extract_nibble(value: u32, nibble_index: u32) -> u32 {
    return (value >> (nibble_index * 4)) & 0x0f;
}

// direct renderer

@group(1) @binding(0) var<storage, read> direct_buffer : array<u32>;

@fragment
fn direct_fragment_main(input: VertexOutput) -> @location(0) vec4f {
    let x = u32(input.crt.x);
    let y = u32(input.crt.y);
    
    let source_index = y * 640 + x;
    let colour_index = extract_byte(direct_buffer[source_index >> 2], source_index & 0x03);
    
    return colour_index_to_rgb(colour_index);
}
