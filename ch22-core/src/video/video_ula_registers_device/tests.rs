use crate::devices::IODevice;
use crate::video::VideoRegisters;
use crate::video::VideoULARegistersDevice;
use crate::word::Word;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(test)]
mod test_video_ula_registers_device {
    use super::*;

    #[test]
    fn test_write_palette() {
        let test_cases = [
            // (address, value, initial_palette, expected_palette)
            (0xfe21, 0x07, 0xffffffffffffffff, 0xfffffffffffffff0), // entry 0: (0x07 & 0x0f) ^ 7 = 0
            (0xfe21, 0x54, 0x1234567890abcdef, 0x12345678903bcdef), // entry 5: (0x54 & 0x0f) ^ 7 = 3
            (0xfe21, 0x8f, 0xaaaaaaaaaaaaaaaa, 0xaaaaaaa8aaaaaaaa), // entry 8: (0x8f & 0x0f) ^ 7 = 8
            (0xfe21, 0xf1, 0x0000000000000000, 0x6000000000000000), // entry 15: (0xf1 & 0x0f) ^ 7 = 6
            (0xfe21, 0xa0, 0x123456789abcdef0, 0x123457789abcdef0), // entry 10: (0xa0 & 0x0f) ^ 7 = 7
            (0xfe21, 0x3c, 0xfedcba9876543210, 0xfedcba987654b210), // entry 3: (0x3c & 0x0f) ^ 7 = 11
            (0xfe21, 0x12, 0x8888888888888888, 0x8888888888888858), // entry 1: (0x12 & 0x0f) ^ 7 = 5
            (0xfe21, 0xc9, 0x1111111111111111, 0x111e111111111111), // entry 12: (0xc9 & 0x0f) ^ 7 = 14
        ];

        for (address, value, initial_palette, expected_palette) in test_cases {
            let video_registers = Rc::new(RefCell::new(VideoRegisters::default()));
            video_registers.borrow_mut().ula_palette = initial_palette;

            let mut device = VideoULARegistersDevice::new(video_registers.clone());

            device.write(Word::from(address), value, 0);

            let actual_palette = video_registers.borrow().ula_palette;

            assert_eq!(
                actual_palette, expected_palette,
                "Failed for address=0x{:04x}, value=0x{:02x}, initial=0x{:016x}",
                address, value, initial_palette
            );
        }
    }

    #[test]
    fn test_write_control_register() {
        let test_cases = [
            // (address, value)
            (0xfe20, 0x9c),
            (0xfe22, 0x1e),
            (0xfe23, 0xab),
            (0xfe20, 0x00),
            (0xfe22, 0xff),
        ];

        for (address, value) in test_cases {
            let video_registers = Rc::new(RefCell::new(VideoRegisters::default()));
            let mut device = VideoULARegistersDevice::new(video_registers.clone());

            device.write(Word::from(address), value, 0);

            assert_eq!(
                video_registers.borrow().ula_control,
                value,
                "Failed for address=0x{:04x}, value=0x{:02x}",
                address,
                value
            );
        }
    }

    #[test]
    fn test_reads() {
        let test_cases = [
            0xfe20, 0xfe21, 0xfe22, 0xfe23, 0x0000, 0xffff, 0x1234, 0xabcd,
        ];

        let video_registers = Rc::new(RefCell::new(VideoRegisters::default()));
        let mut device = VideoULARegistersDevice::new(video_registers);

        for address in test_cases {
            let result = device.read(Word::from(address), 0);
            assert_eq!(result, 0xfe, "Failed for address=0x{:04x}", address);
        }
    }
}
