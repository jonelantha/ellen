use std::ops::Range;
#[cfg(test)]
mod tests;

pub struct VideoMemoryAccess {}

impl VideoMemoryAccess {
    pub fn get_crtc_range_type(crtc_start: u16, crtc_length: u8) -> CRTCRangeType {
        if crtc_length == 0 {
            return CRTCRangeType::Empty;
        }

        let crtc_start = crtc_start & 0x3fff;

        // end is inclusive as exclusive may be in the next range
        let crtc_end = (crtc_start + crtc_length as u16 - 1) & 0x3fff;

        match (crtc_start, crtc_end) {
            (..0x2000, ..0x2000) => CRTCRangeType::HiRes,
            (0x2000.., 0x2000..) => CRTCRangeType::Teletext,
            _ => CRTCRangeType::Mixed,
        }
    }

    pub fn translate_crtc_range(
        crtc_start: u16,
        crtc_length: u8,
        ic32_latch_value: u8,
    ) -> VideoMemoryRanges {
        let crtc_start = crtc_start & 0x3fff;

        // end is exclusive but need to use this to ensure all the
        // scanline bytes for the final address are included
        let crtc_end = (crtc_start + crtc_length as u16) & 0x3fff;

        let (start, start_region) = translate_crtc_address(crtc_start, ic32_latch_value);
        let (end, end_region) = translate_crtc_address(crtc_end, ic32_latch_value);

        // a length of <= 0x10 (u8) means crtc ranges will never span more than 2 regions
        // for hires: neighbouring regions always have different region ranges
        // for teletext: neighbouring regions can have the same region
        // but in case of wrap, end will be less than start
        // (because length (u8) <= 0x10 <= 0x400 = teletext region size)
        // so this first case will be false for all wrap cases
        if start < end && start_region == end_region {
            (start..end, None)
        } else if end == end_region.start {
            // due to how end is exclusive, second region may be empty
            (start..start_region.end, None)
        } else {
            // wrap case
            (start..start_region.end, Some(end_region.start..end))
        }
    }
}

fn translate_crtc_address(crtc_address: u16, ic32_latch_value: u8) -> TranslatedAddress {
    // https://beebwiki.mdfs.net/Address_translation

    // for hires wrap cases:
    // example when video starts at 0x3000 (ic32_latch_value & 0x30 == 0b0010_0000)
    // screen size is 0x5000 = 0x8000 - 0x3000
    // when address gets to wrap point (0x1000), subtract off adjustment 0x0a00 = 0x5000 / 8
    // at second wrap point 0x1a00 = adjustment + 0x1000 to wrap from 0x8000 to 0

    // for teletext wrap cases:
    // each case represents two crtc regions 0x2000-0x2800 => 0x2000-0x2400, 0x2400-0x2800

    match (crtc_address, ic32_latch_value >> 4 & 0x03) {
        (0x0000..0x1000, _) => (crtc_address << 3, 0x0000..0x8000),
        (0x1000..0x1a00, 0b10) => ((crtc_address - 0x0a00) << 3, 0x3000..0x8000),
        (0x1a00..0x2000, 0b10) => ((crtc_address - 0x1a00) << 3, 0x0000..0x8000),
        (0x1000..0x1800, 0b00) => ((crtc_address - 0x0800) << 3, 0x4000..0x8000),
        (0x1800..0x2000, 0b00) => ((crtc_address - 0x1800) << 3, 0x0000..0x8000),
        (0x1000..0x1500, 0b11) => ((crtc_address - 0x0500) << 3, 0x5800..0x8000),
        (0x1500..0x2000, 0b11) => ((crtc_address - 0x1500) << 3, 0x0000..0x8000),
        (0x1000..0x1400, 0b01) => ((crtc_address - 0x0400) << 3, 0x6000..0x8000),
        (0x1400..0x2000, 0b01) => ((crtc_address - 0x1400) << 3, 0x0000..0x8000),
        (0x2000..0x2800, _) => (0x3c00 + (crtc_address & 0x3ff), 0x3c00..0x4000),
        (0x2800..0x3000, _) => (0x7c00 + (crtc_address & 0x3ff), 0x7c00..0x8000),
        (0x3000..0x3800, _) => (0x3c00 + (crtc_address & 0x3ff), 0x3c00..0x4000),
        (0x3800..0x4000, _) => (0x7c00 + (crtc_address & 0x3ff), 0x7c00..0x8000),
        _ => panic!("invalid CRTC address"),
    }
}

pub type VideoMemoryRanges = (Range<u16>, Option<Range<u16>>);

#[derive(PartialEq, Eq, Debug)]
pub enum CRTCRangeType {
    HiRes,
    Teletext,
    Mixed,
    Empty,
}

type TranslatedAddress = (u16, Range<u16>);
