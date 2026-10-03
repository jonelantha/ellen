use std::ops::Range;
#[cfg(test)]
mod tests;

/// Where hires screen memory starts, selected by a two-bit code from the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VideoBase {
    Base4000 = 0b00,
    Base6000 = 0b01,
    Base3000 = 0b10,
    Base5800 = 0b11,
}

impl VideoBase {
    /// `bits` is the two-bit code, 0 to 3.
    pub fn from_bits(bits: u8) -> Self {
        debug_assert!(bits <= 0b11);

        match bits {
            0b00 => VideoBase::Base4000,
            0b01 => VideoBase::Base6000,
            0b10 => VideoBase::Base3000,
            _ => VideoBase::Base5800,
        }
    }
}

pub struct VideoMemoryAccess {}

impl VideoMemoryAccess {
    pub fn translate_crtc_hires_range(
        crtc_start: u16,
        crtc_length: u8,
        video_base: VideoBase,
    ) -> Option<VideoMemoryRanges> {
        let start = Self::translate_crtc_hires_address(crtc_start, video_base)?;
        let end = Self::translate_crtc_hires_address_end(crtc_start, crtc_length, video_base)?;

        // hires ranges will always have a different region if a wrap has occured
        if start.region == end.region {
            // no wrap, one region
            Some((start.address..end.address, None))
        } else {
            // wrapped, two regions
            Some((
                start.address..start.region.end,
                Some(end.region.start..end.address),
            ))
        }
    }

    fn translate_crtc_hires_address_end(
        crtc_address: u16,
        crtc_length: u8,
        video_base: VideoBase,
    ) -> Option<TranslatedAddressAndRegion> {
        debug_assert!(crtc_length > 0);

        Some(
            Self::translate_crtc_hires_address(crtc_address + crtc_length as u16 - 1, video_base)?
                .offsetted(8),
        )
    }

    fn translate_crtc_hires_address(
        crtc_address: u16,
        video_base: VideoBase,
    ) -> Option<TranslatedAddressAndRegion> {
        // https://beebwiki.mdfs.net/Address_translation

        // for hires wrap cases:
        // example when video starts at 0x3000 (VideoBase::Base3000)
        // screen size is 0x5000 = 0x8000 - 0x3000
        // when address gets to wrap point (0x1000), subtract off adjustment 0x0a00 = 0x5000 / 8
        // at second wrap point 0x1a00 = adjustment + 0x1000 to wrap from 0x8000 to 0

        let (region, address) = match (video_base, crtc_address & 0x3fff) {
            (_, 0x0000..0x1000) => (0x0000..0x8000, (crtc_address << 3)),
            (VideoBase::Base3000, 0x1000..0x1a00) => (0x3000..0x8000, (crtc_address - 0x0a00) << 3),
            (VideoBase::Base3000, 0x1a00..0x2000) => (0x0000..0x3000, (crtc_address - 0x1a00) << 3),
            (VideoBase::Base4000, 0x1000..0x1800) => (0x4000..0x8000, (crtc_address - 0x0800) << 3),
            (VideoBase::Base4000, 0x1800..0x2000) => (0x0000..0x4000, (crtc_address - 0x1800) << 3),
            (VideoBase::Base5800, 0x1000..0x1500) => (0x5800..0x8000, (crtc_address - 0x0500) << 3),
            (VideoBase::Base5800, 0x1500..0x2000) => (0x0000..0x5800, (crtc_address - 0x1500) << 3),
            (VideoBase::Base6000, 0x1000..0x1400) => (0x6000..0x8000, (crtc_address - 0x0400) << 3),
            (VideoBase::Base6000, 0x1400..0x2000) => (0x0000..0x6000, (crtc_address - 0x1400) << 3),
            _ => return None,
        };

        Some(TranslatedAddressAndRegion { region, address })
    }

    pub fn translate_crtc_teletext_range(
        crtc_start: u16,
        crtc_length: u8,
    ) -> Option<VideoMemoryRanges> {
        let start = Self::translate_crtc_teletext_address(crtc_start)?;
        let end = Self::translate_crtc_teletext_address_end(crtc_start, crtc_length)?;

        // teletext ranges can share the same region if they wrap
        // but the end address will be less than the start address
        // because the length of data requested will never be more than 100 bytes
        if start.region == end.region && start.address < end.address {
            // no wrap, one region
            Some((start.address..end.address, None))
        } else {
            // wrapped, two regions
            Some((
                start.address..start.region.end,
                Some(end.region.start..end.address),
            ))
        }
    }

    fn translate_crtc_teletext_address_end(
        crtc_address: u16,
        crtc_length: u8,
    ) -> Option<TranslatedAddressAndRegion> {
        debug_assert!(crtc_length > 0);

        Some(
            Self::translate_crtc_teletext_address(crtc_address + crtc_length as u16 - 1)?
                .offsetted(1),
        )
    }

    fn translate_crtc_teletext_address(crtc_address: u16) -> Option<TranslatedAddressAndRegion> {
        // https://beebwiki.mdfs.net/Address_translation

        let region = match crtc_address & 0x3fff {
            0x2000..0x2400 => 0x3c00..0x4000,
            0x2400..0x2800 => 0x3c00..0x4000,
            0x2800..0x2c00 => 0x7c00..0x8000,
            0x2c00..0x3000 => 0x7c00..0x8000,
            0x3000..0x3400 => 0x3c00..0x4000,
            0x3400..0x3800 => 0x3c00..0x4000,
            0x3800..0x3c00 => 0x7c00..0x8000,
            0x3c00..0x4000 => 0x7c00..0x8000,
            _ => return None,
        };

        let address = region.start + (crtc_address & 0x3ff);

        Some(TranslatedAddressAndRegion { region, address })
    }
}

pub type VideoMemoryRanges = (Range<u16>, Option<Range<u16>>);

struct TranslatedAddressAndRegion {
    address: u16,
    region: Range<u16>,
}

impl TranslatedAddressAndRegion {
    fn offsetted(&self, offset: u16) -> Self {
        Self {
            address: self.address + offset,
            region: self.region.clone(),
        }
    }
}
