use crate::video::video_memory_access::VideoMemoryAccess;

#[cfg(test)]
mod is_crtc_range_hires {

    use super::*;

    #[test]
    fn test_ranges() {
        let test_cases = [
            (0x0000, 1, true),
            (0x0500, 5, true),
            (0x1000, 8, true),
            (0x1fff, 1, true),
            (0x2000, 1, false),  // Teletext
            (0x2500, 5, false),  // Teletext
            (0x3000, 8, false),  // Teletext
            (0x3fff, 1, false),  // Teletext
            (0x1ff8, 16, false), // Mixed
            (0x1ffc, 8, false),  // Mixed
            (0x1ffe, 4, false),  // Mixed
        ];

        for (crtc_start, length, expected) in test_cases {
            let result = VideoMemoryAccess::is_crtc_range_hires(crtc_start, length);
            assert_eq!(
                result, expected,
                "Failed for crtc_start=0x{:04x}, length={}",
                crtc_start, length
            );
        }
    }
}

#[cfg(test)]
mod is_crtc_range_telextext {

    use super::*;

    #[test]
    fn test_ranges() {
        let test_cases = [
            (0x0000, 1, false),  // HiRes
            (0x0500, 5, false),  // HiRes
            (0x1000, 8, false),  // HiRes
            (0x1fff, 1, false),  // HiRes
            (0x2000, 1, true),   // Teletext
            (0x2500, 5, true),   // Teletext
            (0x3000, 8, true),   // Teletext
            (0x3fff, 1, true),   // Teletext
            (0x1ff8, 16, false), // Mixed
            (0x1ffc, 8, false),  // Mixed
            (0x1ffe, 4, false),  // Mixed
        ];

        for (crtc_start, length, expected) in test_cases {
            let result = VideoMemoryAccess::is_crtc_range_telextext(crtc_start, length);
            assert_eq!(
                result, expected,
                "Failed for crtc_start=0x{:04x}, length={}",
                crtc_start, length
            );
        }
    }
}

#[cfg(test)]
mod test_translate_crtc_range {
    use super::*;

    #[test]
    fn test_translate_crtc_range() {
        let test_cases = [
            (0x0000, 1, 0x00, ((0x0000..0x0008), None)),
            // HiRes offset
            (0x0001, 1, 0x00, ((0x0008..0x0010), None)),
            // HiRes multi-byte
            (0x0100, 2, 0x00, ((0x0800..0x0810), None)),
            // HiRes end
            (0x0fff, 1, 0x00, ((0x7ff8..0x8000), None)),
            // HiRes wrap 0x00 start
            (0x1000, 1, 0x00, ((0x4000..0x4008), None)),
            // HiRes wrap 0x00 to start
            (0x1800, 1, 0x00, ((0x0000..0x0008), None)),
            // HiRes wrap 0x10 start
            (0x1000, 1, 0x10, ((0x6000..0x6008), None)),
            // HiRes wrap 0x10 to start
            (0x1400, 1, 0x10, ((0x0000..0x0008), None)),
            // HiRes wrap 0x20 start
            (0x1000, 1, 0x20, ((0x3000..0x3008), None)),
            // HiRes wrap 0x20 to start
            (0x1a00, 1, 0x20, ((0x0000..0x0008), None)),
            // HiRes wrap 0x30 start
            (0x1000, 1, 0x30, ((0x5800..0x5808), None)),
            // Wrap 0x30 to start
            (0x1500, 1, 0x30, ((0x0000..0x0008), None)),
            // Teletext start
            (0x2000, 1, 0x00, ((0x3c00..0x3c01), None)),
            // Teletext multi-byte
            (0x2100, 2, 0x00, ((0x3d00..0x3d02), None)),
            // Teletext 2nd half
            (0x2800, 1, 0x00, ((0x7c00..0x7c01), None)),
            // Teletext wrap back
            (0x3000, 1, 0x00, ((0x3c00..0x3c01), None)),
            // Teletext 2nd again
            (0x3800, 1, 0x00, ((0x7c00..0x7c01), None)),
            // Span HiRes to wrap
            (0x0ffe, 4, 0x00, ((0x7ff0..0x8000), Some(0x4000..0x4010))),
            // Span wrap to start
            (0x17fe, 4, 0x00, ((0x7ff0..0x8000), Some(0x0000..0x0010))),
            // Span teletext halves
            (0x27fe, 4, 0x00, ((0x3ffe..0x4000), Some(0x7c00..0x7c02))),
            // Mask 0x4000->0x0000
            (0x4000, 1, 0x00, ((0x0000..0x0008), None)),
            // Mask 0x6000->0x2000
            (0x6000, 1, 0x00, ((0x3c00..0x3c01), None)),
            // ic32 variant comparison (same address, different modes)
            // Mode 0x00
            (0x1200, 1, 0x00, ((0x5000..0x5008), None)),
            // Mode 0x10
            (0x1200, 1, 0x10, ((0x7000..0x7008), None)),
            // Mode 0x20
            (0x1200, 1, 0x20, ((0x4000..0x4008), None)),
            // Mode 0x30
            (0x1200, 1, 0x30, ((0x6800..0x6808), None)),
        ];

        for (crtc_start, length, ic32, expected) in test_cases {
            let result = VideoMemoryAccess::translate_crtc_range(crtc_start, length, ic32);
            assert_eq!(
                result, expected,
                "addr=0x{:04x}, len={}, mode={:?}",
                crtc_start, length, ic32
            );
        }
    }
}
