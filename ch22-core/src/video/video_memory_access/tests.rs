use crate::video::video_memory_access::{VideoBase, VideoMemoryAccess};

#[cfg(test)]
mod test_video_base {
    use super::*;
    use VideoBase::*;

    #[test]
    fn from_bits_maps_each_code_to_its_base() {
        assert_eq!(VideoBase::from_bits((false, false)), Base4000);
        assert_eq!(VideoBase::from_bits((false, true)), Base6000);
        assert_eq!(VideoBase::from_bits((true, false)), Base3000);
        assert_eq!(VideoBase::from_bits((true, true)), Base5800);
    }
}

#[cfg(test)]
mod test_translate_crtc_range {
    use super::*;
    use VideoBase::*;

    #[test]
    fn translate_crtc_hires_range_invalid_cases() {
        let test_cases = [
            (0x2000, 1, Base4000),  // Teletext
            (0x2500, 5, Base4000),  // Teletext
            (0x3000, 8, Base4000),  // Teletext
            (0x3fff, 1, Base4000),  // Teletext
            (0x1ff8, 16, Base4000), // Mixed
            (0x1ffc, 8, Base4000),  // Mixed
            (0x1ffe, 4, Base4000),  // Mixed
        ];

        for (crtc_start, length, video_base) in test_cases {
            let result =
                VideoMemoryAccess::translate_crtc_hires_range(crtc_start, length, video_base);
            assert!(
                result.is_none(),
                "Failed for crtc_start=0x{:04x}, length={}, video_base={:?}",
                crtc_start,
                length,
                video_base
            );
        }
    }

    #[test]
    fn translate_crtc_hires_range_valid_cases() {
        let test_cases = [
            (0x0000, 1, Base4000, (0x0000..0x0008, None)),
            // offset
            (0x0001, 1, Base4000, (0x0008..0x0010, None)),
            // multi-byte
            (0x0100, 2, Base4000, (0x0800..0x0810, None)),
            // end
            (0x0fff, 1, Base4000, (0x7ff8..0x8000, None)),
            // wrap Base4000 start
            (0x1000, 1, Base4000, (0x4000..0x4008, None)),
            // wrap Base4000 to start
            (0x1800, 1, Base4000, (0x0000..0x0008, None)),
            // wrap Base6000 start
            (0x1000, 1, Base6000, (0x6000..0x6008, None)),
            // wrap Base6000 to start
            (0x1400, 1, Base6000, (0x0000..0x0008, None)),
            // wrap Base3000 start
            (0x1000, 1, Base3000, (0x3000..0x3008, None)),
            // wrap Base3000 to start
            (0x1a00, 1, Base3000, (0x0000..0x0008, None)),
            // wrap Base5800 start
            (0x1000, 1, Base5800, (0x5800..0x5808, None)),
            // wrap Base5800 to start
            (0x1500, 1, Base5800, (0x0000..0x0008, None)),
            // Span to wrap
            (0x0ffe, 4, Base4000, (0x7ff0..0x8000, Some(0x4000..0x4010))),
            // Span wrap to start
            (0x17fe, 4, Base4000, (0x7ff0..0x8000, Some(0x0000..0x0010))),
            // Mask 0x4000->0x0000
            (0x4000, 1, Base4000, (0x0000..0x0008, None)),
            // video base comparison (same address, different bases)
            // Base4000
            (0x1200, 1, Base4000, (0x5000..0x5008, None)),
            // Base6000
            (0x1200, 1, Base6000, (0x7000..0x7008, None)),
            // Base3000
            (0x1200, 1, Base3000, (0x4000..0x4008, None)),
            // Base5800
            (0x1200, 1, Base5800, (0x6800..0x6808, None)),
            // ends on wrap boundary
            (0x17F8, 8, Base4000, (0x7fc0..0x8000, None)),
            // ends on hires boundary
            (0x1FF8, 8, Base4000, (0x3fc0..0x4000, None)),
        ];

        for (crtc_start, length, video_base, expected) in test_cases {
            let result =
                VideoMemoryAccess::translate_crtc_hires_range(crtc_start, length, video_base);
            assert_eq!(
                result,
                Some(expected),
                "addr=0x{:04x}, len={}, video_base={:?}",
                crtc_start,
                length,
                video_base
            );
        }
    }

    #[test]
    fn test_translate_teletext_range_invalid_cases() {
        let test_cases = [
            (0x0000, 1),  // HiRes
            (0x0500, 5),  // HiRes
            (0x1000, 8),  // HiRes
            (0x1fff, 1),  // HiRes
            (0x1ff8, 16), // Mixed
            (0x1ffc, 8),  // Mixed
            (0x1ffe, 4),  // Mixed
        ];

        for (crtc_start, length) in test_cases {
            let result = VideoMemoryAccess::translate_crtc_teletext_range(crtc_start, length);
            assert!(
                result.is_none(),
                "Failed for crtc_start=0x{:04x}, length={}",
                crtc_start,
                length,
            );
        }
    }

    #[test]
    fn test_translate_teletext_range_valid_cases() {
        let test_cases = [
            // start
            (0x2000, 1, (0x3c00..0x3c01, None)),
            // multi-byte
            (0x2100, 2, (0x3d00..0x3d02, None)),
            // 2nd half
            (0x2800, 1, (0x7c00..0x7c01, None)),
            // wrap back
            (0x3000, 1, (0x3c00..0x3c01, None)),
            // 2nd again
            (0x3800, 1, (0x7c00..0x7c01, None)),
            // Span regions
            (0x27fe, 4, (0x3ffe..0x4000, Some(0x7c00..0x7c02))),
            // Mask 0x6000->0x2000
            (0x6000, 1, (0x3c00..0x3c01, None)),
            // ends on wrap boundary
            (0x27F8, 8, (0x3ff8..0x4000, None)),
            // ends on teletext/hires boundary
            (0x3FF8, 8, (0x7ff8..0x8000, None)),
            // crossing wrap boundary into same space
            (0x2be8, 40, (0x7fe8..0x8000, Some(0x7c00..0x7c10))),
        ];

        for (crtc_start, length, expected) in test_cases {
            let result = VideoMemoryAccess::translate_crtc_teletext_range(crtc_start, length);
            assert_eq!(
                result,
                Some(expected),
                "addr=0x{:04x}, len={}",
                crtc_start,
                length
            );
        }
    }
}
