#[cfg(test)]
mod test_video_crtc_registers_device {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::devices::IODevice;
    use crate::video::VideoCRTCRegistersDevice;
    use crate::video::VideoRegisters;
    use crate::word::Word;

    #[test]
    fn test_writes_register_masking() {
        // (control_reg, write_value, expected_masked_value)
        let test_cases = [
            (0, 0x5a, 0x5a),  // r0 no mask
            (1, 0xff, 0xff),  // r1 no mask
            (2, 0x12, 0x12),  // r2 no mask
            (3, 0xee, 0xee),  // r3 no mask
            (4, 0xff, 0x7f),  // r4 mask 0x7f
            (5, 0xff, 0x1f),  // r5 mask 0x1f
            (6, 0xff, 0x7f),  // r6 mask 0x7f
            (7, 0xff, 0x7f),  // r7 mask 0x7f
            (8, 0xaa, 0xaa),  // r8 no mask
            (9, 0xff, 0x1f),  // r9 mask 0x1f
            (10, 0xff, 0x7f), // r10 mask 0x7f
            (11, 0xff, 0x1f), // r11 mask 0x1f
            (12, 0xff, 0x3f), // r12 mask 0x3f
            (13, 0x77, 0x77), // r13 no mask
            (14, 0xff, 0x3f), // r14 mask 0x3f
            (15, 0x42, 0x42), // r15 no mask
        ];

        for (control_reg, write_value, expected_masked) in test_cases {
            let video_registers = Rc::new(RefCell::new(VideoRegisters::default()));
            let mut device = VideoCRTCRegistersDevice::new(video_registers.clone());

            // Select control register (address low bits != 0x01)
            device.write(Word::from(0xfe20), control_reg, 0); // sets control_reg = value & 0x1f
            // Perform write to selected register (address low bits == 0x01)
            device.write(Word::from(0xfe21), write_value, 0);

            let regs = video_registers.borrow();
            let actual = regs.get_crtc_register(control_reg);

            assert_eq!(
                actual, expected_masked,
                "Failed for control_reg={}, write=0x{:02x}",
                control_reg, write_value
            );
        }
    }

    #[test]
    fn test_reads_cursor_registers_and_defaults() {
        let video_registers = VideoRegisters {
            crtc_r14_cursor_h: 0x3f,
            crtc_r15_cursor_l: 0x5a,
            ..VideoRegisters::default()
        };

        let mut device = VideoCRTCRegistersDevice::new(Rc::new(RefCell::new(video_registers)));

        // Read r14
        device.write(Word::from(0xfe20), 14, 0); // select r14
        let r14_read = device.read(Word::from(0xfe21), 0);
        assert_eq!(r14_read, 0x3f, "Read of r14 should return value 0x3f");

        // Read r15
        device.write(Word::from(0xfe20), 15, 0); // select r15
        let r15_read = device.read(Word::from(0xfe21), 0);
        assert_eq!(r15_read, 0x5a, "Read of r15 should return value 0x5a");

        // Reads of other control registers (e.g. r0) should return 0
        device.write(Word::from(0xfe20), 0, 0); // select r0
        let r0_read = device.read(Word::from(0xfe21), 0);
        assert_eq!(r0_read, 0x00, "Non-readable register should return 0");
    }

    #[test]
    #[should_panic(expected = "not implemented")]
    fn test_read_unimplemented_r12_should_panic() {
        let video_registers = VideoRegisters::default();
        let mut device = VideoCRTCRegistersDevice::new(Rc::new(RefCell::new(video_registers)));

        device.write(Word::from(0xfe20), 12, 0); // select register
        device.read(Word::from(0xfe21), 0); // should panic
    }

    #[test]
    #[should_panic(expected = "not implemented")]
    fn test_read_unimplemented_r13_should_panic() {
        let video_registers = VideoRegisters::default();
        let mut device = VideoCRTCRegistersDevice::new(Rc::new(RefCell::new(video_registers)));

        device.write(Word::from(0xfe20), 13, 0); // select register
        device.read(Word::from(0xfe21), 0); // should panic
    }

    #[test]
    #[should_panic(expected = "not implemented")]
    fn test_read_unimplemented_r16_should_panic() {
        let video_registers = VideoRegisters::default();
        let mut device = VideoCRTCRegistersDevice::new(Rc::new(RefCell::new(video_registers)));

        device.write(Word::from(0xfe20), 16, 0); // select register
        device.read(Word::from(0xfe21), 0); // should panic
    }

    #[test]
    #[should_panic(expected = "not implemented")]
    fn test_read_unimplemented_r17_should_panic() {
        let video_registers = VideoRegisters::default();
        let mut device = VideoCRTCRegistersDevice::new(Rc::new(RefCell::new(video_registers)));

        device.write(Word::from(0xfe20), 17, 0); // select register
        device.read(Word::from(0xfe21), 0); // should panic
    }
}
