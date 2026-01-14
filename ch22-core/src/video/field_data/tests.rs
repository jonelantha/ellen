use super::Field;
use crate::video::{FieldLine, VideoRegisters, field_line_flags::*};

struct LineDataSlices<'a> {
    flags: u8,
    char_data: &'a [u8],
    crtc_registers: &'a [u8],
    back_porch: u8,
    crtc_ula_control_and_palette: &'a [u8],
    cursor_char: u8,
}

fn get_line_data_slices(line: &FieldLine) -> LineDataSlices<'_> {
    let raw_data = line.get_raw_data();
    LineDataSlices {
        flags: raw_data[0],                                // flags
        char_data: &raw_data[1..101],                      // char data
        crtc_registers: &raw_data[101..102],               // crtc registers
        back_porch: raw_data[102],                         // back porch
        crtc_ula_control_and_palette: &raw_data[103..112], // crtc ula control & palette
        cursor_char: raw_data[112],                        // cursor char
    }
}

#[cfg(test)]
mod field_data_tests {
    use super::*;

    #[test]
    fn test_writes_registers_to_line() {
        let mut field = Field::default();
        let line_index = 5;
        let crtc_start = 0x1234;
        let raster = 0x1A;
        let video_registers = VideoRegisters {
            crtc_r0_horizontal_total: 0x80,
            crtc_r1_horizontal_displayed: 0x12,
            crtc_r2_horizontal_sync_position: 0x13,
            crtc_r3_sync_width: 0x14,
            ula_control: 0x20,
            ula_palette: 0x01_23_45_67_89_AB_CD_EF,
            ..VideoRegisters::default()
        };

        field.snapshot_scanline(
            line_index,
            crtc_start,
            raster,
            raster,
            0,
            0,
            &video_registers,
            |_| &[],
        );

        let data_slices = get_line_data_slices(&field.lines[line_index]);

        assert_eq!(data_slices.crtc_registers, [0x12]);
        assert_eq!(data_slices.back_porch, 0x6a);
        assert_eq!(
            data_slices.crtc_ula_control_and_palette,
            [0x20, 0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01]
        );
        assert_eq!(data_slices.cursor_char, 0x00);
    }

    #[test]
    fn test_display_flags_logic() {
        let line_index = 7;

        let test_cases = [
            // start_address, r1, r8, ula_control, raster_even, raster_odd, expected_flags
            // ula hires cases (ula_control: 0x00)
            (0x1000, 0x00, 0x00, 0x00, 1, 1, DISPLAYED), // zero length
            (0x1000, 0x14, 0x00, 0x00, 8, 8, DISPLAYED), // raster > 8
            (0x1000, 0x14, 0x30, 0x00, 1, 1, DISPLAYED), // screen delay no output
            (0x1000, 0x14, 0x00, 0x00, 7, 7, DISPLAYED | HAS_BYTES), // non-zero length & raster in range
            (0x2000, 0x10, 0x00, 0x00, 1, 1, DISPLAYED | INVALID_RANGE), // teletext region
            (0x1FFF, 0x08, 0x00, 0x00, 1, 1, DISPLAYED | INVALID_RANGE), // mixed: hires -> teletext
            (0x3FFF, 0x04, 0x00, 0x00, 1, 1, DISPLAYED | INVALID_RANGE), // mixed: teletext -> hires
            // ula teletext cases (ula_control: 0x02)
            (0x2000, 0x00, 0x00, 0x02, 1, 1, DISPLAYED), // zero length
            (0x2000, 0x14, 0x00, 0x02, 15, 15, DISPLAYED | HAS_BYTES), // non-zero length & raster > 8
            (0x1000, 0x10, 0x00, 0x02, 1, 1, DISPLAYED | INVALID_RANGE), // hires region
            (0x3FFF, 0x08, 0x00, 0x02, 1, 1, DISPLAYED | INVALID_RANGE), // mixed: teletext -> hires
            (0x1FFF, 0x04, 0x00, 0x02, 1, 1, DISPLAYED | INVALID_RANGE), // mixed: hires -> teletext
        ];

        for (crtc_start, r1, r8, ula_control, raster_even, raster_odd, expected_flags) in test_cases
        {
            let mut field = Field::default();

            let video_registers = VideoRegisters {
                crtc_r1_horizontal_displayed: r1,
                crtc_r8_interlace_and_skew: r8,
                ula_control,
                ..VideoRegisters::default()
            };

            field.snapshot_scanline(
                line_index,
                crtc_start,
                raster_even,
                raster_odd,
                0, // ic32 latch value
                0, // field counter
                &video_registers,
                |_| &[],
            );

            let data_slices = get_line_data_slices(&field.lines[line_index]);

            assert_eq!(
                data_slices.flags, expected_flags,
                "Failed for crtc_start=0x{:04x}, length={}, r8=0x{:02x}, ula_control=0x{:02x}, raster_even=0x{:02x}, raster_odd=0x{:02x}",
                crtc_start, r1, r8, ula_control, raster_even, raster_odd
            );
        }
    }

    #[test]
    fn test_interlace_flags_logic() {
        let line_index = 7;

        let test_cases = [
            // r8, expected_flags
            (0, 0),
            (0x03, INTERLACE_VIDEO_AND_SYNC),
        ];

        for (crtc_r8_interlace_and_skew, expected_flags) in test_cases {
            let mut field = Field::default();

            let video_registers = VideoRegisters {
                crtc_r8_interlace_and_skew,
                ..VideoRegisters::default()
            };

            field.snapshot_scanline(
                line_index,
                0x1000, // crtc start
                0,      // raster_even
                0,      // raster_odd
                0,      // ic32 latch value
                0,      // field counter
                &video_registers,
                |_| &[],
            );

            let data_slices = get_line_data_slices(&field.lines[line_index]);

            assert_eq!(
                data_slices.flags & INTERLACE_VIDEO_AND_SYNC,
                expected_flags,
                "Failed for crtc_r8_interlace_and_skew=0x{:02x}",
                crtc_r8_interlace_and_skew
            );
        }
    }

    #[test]
    fn test_cursor_flags_logic() {
        let line_index = 7;

        let test_cases = [
            // raster, r8, r10, r11, r14, r15, expected_cursor_char, expected_cursor_even, expected_cursor_odd
            // Raster tests
            (1, 0, 0, 0, 0, 0, 0, false, false), // rasters not in range
            (5, 0, 3, 7, 0x00, 0x00, 0, false, false), // rasters in range but cursor before display area
            (5, 0, 3, 7, 0x10, 0x0A, 10, true, true), // both rasters in range, cursor at relative position 10
            (3, 0, 3, 7, 0x10, 0x14, 20, true, true), // even raster at start of range, cursor at position 20
            (7, 0, 3, 7, 0x10, 0x1E, 30, true, false), // even raster at end of range, odd raster out of range
            (2, 0, 3, 7, 0x10, 0x19, 25, false, true), // only odd raster (=3) in range [3..7], even (=2) out
            // Cursor delay tests (r8 bits 6-7)
            (5, 0xC0, 3, 7, 0x10, 0x0A, 0, false, false), // cursor_delay == 3 (disabled)
            (5, 0x00, 3, 7, 0x10, 0x0A, 10, true, true),  // cursor_delay == 0
            (5, 0x40, 3, 7, 0x10, 0x0A, 11, true, true), // cursor_delay == 1 (cursor_char = delay + rel_address)
            (5, 0x80, 3, 7, 0x10, 0x0A, 12, true, true), // cursor_delay == 2
            // Address range tests
            (5, 0, 3, 7, 0x10, 0x05, 5, true, true), // cursor at relative position 5
            (5, 0, 3, 7, 0x10, 0x50, 0, false, false), // cursor at position 0x50 (r1=0x50, so exactly at boundary - outside)
            (5, 0, 3, 7, 0x10, 0x4F, 0x4F, true, true), // cursor at position 0x4F (last valid position)
            (5, 0, 3, 7, 0x00, 0xFF, 0, false, false), // cursor_address = 0x00FF < 0x1000 (before display)
            (5, 0, 3, 7, 0x10, 0x20, 0x20, true, true), // cursor at 0x1020, relative position 0x20
        ];

        for (raster, r8, r10, r11, r14, r15, expected_cursor_char, expected_even, expected_odd) in
            test_cases
        {
            let mut field = Field::default();

            let video_registers = VideoRegisters {
                crtc_r1_horizontal_displayed: 0x50,
                crtc_r8_interlace_and_skew: r8,
                crtc_r10_cursor_start_raster: r10,
                crtc_r11_cursor_end_raster: r11,
                crtc_r14_cursor_h: r14,
                crtc_r15_cursor_l: r15,
                ..VideoRegisters::default()
            };

            field.snapshot_scanline(
                line_index,
                0x1000, // crtc start
                raster,
                raster + 1,
                0, // ic32 latch value
                0, // field counter
                &video_registers,
                |_| &[],
            );

            let data_slices = get_line_data_slices(&field.lines[line_index]);

            let msg = format!(
                "raster=0x{:02x}, r8=0x{:02x}, r10=0x{:02x}, r11=0x{:02x}, r14=0x{:02x}, r15=0x{:02x}",
                raster, r8, r10, r11, r14, r15
            );

            assert_eq!(
                data_slices.cursor_char, expected_cursor_char,
                "cursor char: {}",
                msg
            );
            assert_eq!(
                data_slices.flags & CURSOR_RASTER_EVEN != 0,
                expected_even,
                "cursor even: {}",
                msg
            );
            assert_eq!(
                data_slices.flags & CURSOR_RASTER_ODD != 0,
                expected_odd,
                "cursor odd: {}",
                msg
            );
        }
    }

    #[test]
    fn test_cursor_blink_modes() {
        let line_index = 7;

        // Test cases: (r10_blink_mode_bits, visible_field_counter_ranges)
        let test_cases: &[(u8, Vec<std::ops::Range<u8>>)] = &[
            (0x00, vec![0..64]),                         // solid
            (0x20, vec![]),                              // off
            (0x40, vec![8..16, 24..32, 40..48, 56..64]), // fast blink
            (0x60, vec![16..32, 48..64]),                // slow blink
        ];

        for (r10_blink_mode_bits, visible_ranges) in test_cases {
            // Test all 255 field_counter values
            for field_counter in 0..=255u8 {
                let mut field = Field::default();

                let video_registers = VideoRegisters {
                    crtc_r1_horizontal_displayed: 0x50,
                    crtc_r8_interlace_and_skew: 0,
                    crtc_r10_cursor_start_raster: *r10_blink_mode_bits,
                    crtc_r11_cursor_end_raster: 7,
                    crtc_r14_cursor_h: 0x10,
                    crtc_r15_cursor_l: 0x0F,
                    ..VideoRegisters::default()
                };

                field.snapshot_scanline(
                    line_index,
                    0x1000, // crtc start
                    5,      // raster even
                    6,      // raster odd
                    0,      // ic32 latch value
                    field_counter,
                    &video_registers,
                    |_| &[],
                );

                let data_slices = get_line_data_slices(&field.lines[line_index]);

                // pattern repeats so get expected value using field_counter % 64
                let expected_visible = visible_ranges
                    .iter()
                    .any(|range| range.contains(&(field_counter % 64)));

                let is_visible =
                    (data_slices.flags & (CURSOR_RASTER_EVEN | CURSOR_RASTER_ODD)) != 0;

                assert_eq!(
                    is_visible, expected_visible,
                    "Blink mode r10=0x{:02x} failed for field_counter={}",
                    r10_blink_mode_bits, field_counter
                );
            }
        }
    }

    #[test]
    fn test_back_porch_logic() {
        let line_index = 7;

        let test_cases = [
            // r0, r2, r3, expected_back_porch
            // basic cases
            (0x7F, 0x60, 0x02, 0x1E),
            (0x7F, 0x63, 0x28, 0x15),
            (0x63, 0x4F, 0x05, 0x10),
            // sync width variations
            (0x80, 0x60, 0x0F, 0x12),
            (0x80, 0x60, 0x08, 0x19),
            (0x80, 0x60, 0xF2, 0x1F), // only lower nibble r3 used
            (0x7F, 0x70, 0xA5, 0x0B), // only lower nibble r3 used
            (0x63, 0x50, 0x3F, 0x05), // only lower nibble r3 used
            (0x80, 0x60, 0x00, 0x20), // r3 & 0x0f = 0, uses min of 1
            // edge cases
            (0x63, 0x5A, 0x0A, 0x00), // back porch = 0
            (0xFE, 0xF0, 0x0E, 0x01), // large r0
            (0xFF, 0x80, 0x00, 0x7F), // largest r0, r3 = 0 uses min of 1
            (0x01, 0x00, 0x00, 0x01), // small r0, r3 = 0 uses min of 1
            (0x00, 0x00, 0x00, 0x00), // smallest r0, r3 = 0 uses min of 1
            (0x40, 0x50, 0x05, 0x00), // r2 + r3 > r0 + 1
            (0x10, 0x20, 0x08, 0x00), // r2 + r3 > r0 + 1
            (0x00, 0x01, 0x01, 0x00), // r2 + r3 > r0 + 1
            (0xFF, 0xFF, 0x00, 0x00), // r3 = 0
            (0xFE, 0x00, 0x00, 0xFE), // r3 = 0
        ];

        for (
            crtc_r0_horizontal_total,
            crtc_r2_horizontal_sync_position,
            crtc_r3_sync_width,
            expected_back_porch,
        ) in test_cases
        {
            let mut field = Field::default();

            let video_registers = VideoRegisters {
                crtc_r0_horizontal_total,
                crtc_r2_horizontal_sync_position,
                crtc_r3_sync_width,
                ..VideoRegisters::default()
            };

            field.snapshot_scanline(
                line_index,
                0x1000, // crtc start
                0,      // raster_even
                0,      // raster_odd
                0,      // ic32 latch value
                0,      // field counter
                &video_registers,
                |_| &[],
            );

            let data_slices = get_line_data_slices(&field.lines[line_index]);

            assert_eq!(
                data_slices.back_porch, expected_back_porch,
                "Failed for crtc_r0_horizontal_total=0x{:02x}, crtc_r2_horizontal_sync_position=0x{:02x}, crtc_r3_sync_width=0x{:02x}",
                crtc_r0_horizontal_total, crtc_r2_horizontal_sync_position, crtc_r3_sync_width
            );
        }
    }

    #[test]
    fn test_teletext_writing_char_data() {
        let mut field = Field::default();
        let char_data: Vec<u8> = (0..20).collect();

        field.snapshot_scanline(
            12,     // line index
            0x2000, // crtc start
            0,      // raster line even
            0,      // raster line odd
            0,      // ic32 latch value
            0,      // field counter
            &VideoRegisters {
                ula_control: 0x02,
                crtc_r1_horizontal_displayed: 0x10,
                ..VideoRegisters::default()
            },
            |range| match (range.start, range.end) {
                (0x3C00, 0x3C10) => &char_data,
                _ => panic!("Unexpected range: {:?}", range),
            },
        );

        let data_slices = get_line_data_slices(&field.lines[12]);

        assert_eq!(data_slices.char_data[..20], char_data);
    }

    #[test]
    fn test_teletext_writing_char_data_wrapping() {
        let mut field = Field::default();
        let char_data_region_1: Vec<u8> = (0..20).collect();
        let char_data_region_2: Vec<u8> = (50..60).collect();

        field.snapshot_scanline(
            12,     // line index
            0x27F0, // crtc start
            0,      // raster line even
            0,      // raster line odd
            0,      // ic32 latch value
            0,      // field counter
            &VideoRegisters {
                ula_control: 0x02,
                crtc_r1_horizontal_displayed: 0x50,
                ..VideoRegisters::default()
            },
            |range| match (range.start, range.end) {
                (0x3FF0, 0x4000) => &char_data_region_1,
                (0x7C00, 0x7C40) => &char_data_region_2,
                _ => panic!("Unexpected range: {:?}", range),
            },
        );

        let data_slices = get_line_data_slices(&field.lines[12]);

        assert_eq!(data_slices.char_data[..20], char_data_region_1);
        assert_eq!(data_slices.char_data[20..30], char_data_region_2);
    }

    #[test]
    fn test_hires_writing_char_data() {
        let mut field = Field::default();
        let char_data: Vec<u8> = (0..0x40).collect();

        field.snapshot_scanline(
            12,     // line index
            0x1000, // crtc start
            3,      // raster line even
            3,      // raster line odd
            0,      // ic32 latch value
            0,      // field counter
            &VideoRegisters {
                crtc_r1_horizontal_displayed: 0x10,
                ..VideoRegisters::default()
            },
            |range| match (range.start, range.end) {
                (0x4000, 0x4080) => &char_data,
                _ => panic!("Unexpected range: {:?}", range),
            },
        );

        let data_slices = get_line_data_slices(&field.lines[12]);

        assert_eq!(
            data_slices.char_data[..8],
            [0x03, 0x0B, 0x13, 0x1B, 0x23, 0x2B, 0x33, 0x3B] // every 8th byte starting from index 3
        );
    }

    #[test]
    fn test_hires_writing_char_data_wrapping() {
        let mut field = Field::default();
        let char_data_region_1: Vec<u8> = (0..0x40).collect();
        let char_data_region_2: Vec<u8> = (0x60..0x80).collect();

        field.snapshot_scanline(
            12,     // line index
            0x17F0, // crtc start
            5,      // raster odd line
            5,      // raster even line
            0,      // ic32 latch value
            0,      // field counter
            &VideoRegisters {
                crtc_r1_horizontal_displayed: 0x20,
                ..VideoRegisters::default()
            },
            |range| match (range.start, range.end) {
                (0x7F80, 0x8000) => &char_data_region_1,
                (0x0000, 0x0080) => &char_data_region_2,
                _ => panic!("Unexpected range: {:?}", range),
            },
        );

        let data_slices = get_line_data_slices(&field.lines[12]);

        assert_eq!(
            data_slices.char_data[..12],
            [
                0x05, 0x0D, 0x15, 0x1D, 0x25, 0x2D, 0x35, 0x3D, 0x65, 0x6D, 0x75, 0x7D
            ] // first 8 bytes: every 8th byte starting from index 5 from region 1
              // final 4 bytes: every 8th byte starting from index 5 from region 2
        );
    }
}
