use super::Field;
use crate::video::{FieldLine, VideoRegisters, field_line_flags::*};

struct LineDataSlices<'a> {
    flags: u8,
    char_data: &'a [u8],
    crtc_counters: &'a [u8],
    crtc_registers: &'a [u8],
    crtc_ula_control_and_palette: &'a [u8],
}

fn get_line_data_slices(line: &FieldLine) -> LineDataSlices<'_> {
    let raw_data = line.get_raw_data();
    LineDataSlices {
        flags: raw_data[0],                             // flags
        char_data: &raw_data[1..101],                   // char data
        crtc_counters: &raw_data[101..103],             // crtc counters
        crtc_registers: &raw_data[103..110],            // crtc registers
        crtc_ula_control_and_palette: &raw_data[110..], // crtc ula control & palette
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
            crtc_r8_interlace_and_skew: 0x15,
            crtc_r14_cursor_h: 0x18,
            crtc_r15_cursor_l: 0x19,
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

        assert_eq!(data_slices.crtc_counters, [0x34, 0x12]);
        assert_eq!(
            data_slices.crtc_registers,
            [0x80, 0x12, 0x13, 0x14, 0x15, 0x18, 0x19]
        );
        assert_eq!(
            data_slices.crtc_ula_control_and_palette,
            [0x20, 0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01]
        );
    }

    #[test]
    fn test_flags_logic() {
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
                0,
                0,
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
