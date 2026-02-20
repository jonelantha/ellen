use super::Crtc;
use crate::video::VideoRegisters;

fn create_default_registers() -> VideoRegisters {
    VideoRegisters {
        crtc_r0_horizontal_total: 127,
        crtc_r1_horizontal_displayed: 80,
        crtc_r3_sync_width: 0x20,
        crtc_r4_vertical_total: 24,
        crtc_r5_vertical_total_adjust: 0,
        crtc_r6_vertical_displayed: 20,
        crtc_r7_vertical_sync_position: 20,
        crtc_r8_interlace_and_skew: 0x00,
        crtc_r9_maximum_raster_address: 7,
        crtc_r12_start_address_h: 0x20,
        crtc_r13_start_address_l: 0x00,
        ula_control: 0x00,
        ..Default::default()
    }
}

fn setup_test(registers: &VideoRegisters) -> Crtc {
    let mut crtc = Crtc::default();
    crtc.init(registers);
    crtc
}

#[test]
fn should_increment_scanline_by_1_on_each_call() {
    let registers = create_default_registers();
    let mut crtc = setup_test(&registers);

    let expected_scanlines: [u16; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    for (index, expected) in expected_scanlines.iter().enumerate() {
        let actual = crtc
            .advance_scanline(&registers)
            .snapshot_params
            .beam_scanline;

        assert_eq!(actual, *expected, "mismatch at index {index}");
    }
}

#[test]
fn should_cycle_raster_address_even_pattern_after_char_scanlines_calls() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 7;
    registers.crtc_r1_horizontal_displayed = 80;

    let mut crtc = setup_test(&registers);

    let expected_pattern = [0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7];

    for (index, expected) in expected_pattern.iter().enumerate() {
        let actual = crtc
            .advance_scanline(&registers)
            .snapshot_params
            .raster_address_even;

        assert_eq!(actual, *expected, "mismatch at index {index}");
    }
}

#[test]
fn should_advance_address_by_horizontal_displayed_after_char_scanlines() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 7; // 8 scanlines per character line (0-7)
    registers.crtc_r1_horizontal_displayed = 50; // 0x32 in hex

    let mut crtc = setup_test(&registers);

    // Address stays at 0x2000 for all 8 scanlines of first character line,
    // then advances by horizontal_displayed (0x32) to 0x2032 on scanline 8
    let expected_addresses = [
        0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2032,
    ];

    for (index, expected) in expected_addresses.iter().enumerate() {
        let actual = crtc.advance_scanline(&registers).snapshot_params.address;

        assert_eq!(actual, *expected, "mismatch at index {index}");
    }
}

#[test]
fn should_transition_in_scan_from_true_to_false_at_display_boundary() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1; // 2 scanlines per character line
    registers.crtc_r6_vertical_displayed = 2; // 2 character lines displayed
    registers.crtc_r4_vertical_total = 10;

    let mut crtc = setup_test(&registers);

    // in_scan is true for 2 char lines * 2 scanlines = 4 scanlines, then false
    let expected_pattern = [true, true, true, true, false, false, false];

    for (index, expected) in expected_pattern.iter().enumerate() {
        let actual = crtc.advance_scanline(&registers).snapshot_params.in_scan;

        assert_eq!(actual, *expected, "mismatch at index {index}");
    }
}

#[test]
fn should_return_field_complete_true_at_max_lines() {
    let mut registers = create_default_registers();
    // Set vsync position to max to prevent normal field completion,
    // forcing the hardware limit (MAX_LINES) to trigger instead
    registers.crtc_r7_vertical_sync_position = 255;

    let mut crtc = setup_test(&registers);

    let mut field_complete_iteration: i32 = -1;
    for i in 0..(crate::video::MAX_LINES + 10) {
        let result = crtc.advance_scanline(&registers);
        if result.field_complete {
            field_complete_iteration = i as i32;
            break;
        }
    }

    assert_eq!(
        field_complete_iteration,
        crate::video::MAX_LINES as i32 - 1,
        "field_complete should occur at iteration",
    );
}

#[test]
fn should_stop_incrementing_addr_after_char_line_exceeds_vertical_total() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1; // 2 scanlines per character line
    registers.crtc_r4_vertical_total = 2; // 3 character lines total (0-2)
    registers.crtc_r1_horizontal_displayed = 50; // 0x32 per line
    registers.crtc_r6_vertical_displayed = 2; // 2 character lines visible
    registers.crtc_r7_vertical_sync_position = 10;

    let mut crtc = setup_test(&registers);

    // After char line 2 exceeds vertical_total (2), address remains at 0x2064
    // until frame resets at scanline 6, then pattern repeats
    let expected = [
        // address, in_scan, scanline, raster_address_even
        (0x2000, true, 0, 0),  // char line 0, raster 0
        (0x2000, true, 1, 1),  // char line 0, raster 1
        (0x2032, true, 2, 0),  // char line 1, raster 0
        (0x2032, true, 3, 1),  // char line 1, raster 1
        (0x2064, false, 4, 0), // char line 2, raster 0 (out of display)
        (0x2064, false, 5, 1), // char line 2, raster 1 (address frozen)
        (0x2000, true, 6, 0),  // frame reset
        (0x2000, true, 7, 1),
        (0x2032, true, 8, 0),
        (0x2032, true, 9, 1),
    ];

    for (index, (expected_address, expected_in_scan, expected_scanline, expected_raster)) in
        expected.iter().enumerate()
    {
        let actual = crtc.advance_scanline(&registers).snapshot_params;

        assert_eq!(
            actual.address, *expected_address,
            "address mismatch at index {index}",
        );
        assert_eq!(
            actual.in_scan, *expected_in_scan,
            "in_scan mismatch at index {index}",
        );
        assert_eq!(
            actual.beam_scanline, *expected_scanline,
            "scanline mismatch at index {index}",
        );
        assert_eq!(
            actual.raster_address_even, *expected_raster,
            "raster_address_even mismatch at index {index}",
        );
    }
}

#[test]
fn should_maintain_consistent_scanline_delta() {
    let registers = create_default_registers();
    let mut crtc = setup_test(&registers);

    let expected_scanlines: Vec<u16> = (0..50).map(|i| i as u16).collect();

    for (index, expected) in expected_scanlines.iter().enumerate() {
        let actual = crtc
            .advance_scanline(&registers)
            .snapshot_params
            .beam_scanline;

        assert_eq!(actual, *expected, "mismatch at index {index}");
    }
}

#[test]
fn should_track_raster_address_progression_correctly() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 3;
    registers.crtc_r8_interlace_and_skew = 0x00;

    let mut crtc = setup_test(&registers);

    let expected_pattern = [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3];

    for (index, expected) in expected_pattern.iter().enumerate() {
        let actual = crtc
            .advance_scanline(&registers)
            .snapshot_params
            .raster_address_even;

        assert_eq!(actual, *expected, "mismatch at index {index}");
    }
}

#[test]
fn should_trigger_vsync_at_correct_positions() {
    // Test vsync triggering: vsync starts at char_line * scanlines_per_char,
    // continues for (sync_width >> 4) scanlines.
    // With r9=1 (2 scanlines/char): char_line N starts at scanline N*2
    let test_cases = [
        // r7_vertical_sync_position, r3_sync_width, expected_indices, length
        (1, 0x00, vec![], 10), // vsync at char 1 (scanline 2), width=0: no vsync
        (2, 0x10, vec![4], 15), // vsync at char 2 (scanline 4), width=1
        (2, 0x20, vec![4, 5], 20), // vsync at char 2 (scanline 4), width=2
        (3, 0x20, vec![6, 7], 20), // vsync at char 3 (scanline 6), width=2
        (2, 0x30, vec![4, 5, 6], 20), // vsync at char 2 (scanline 4), width=3
        (2, 0x40, vec![4, 5, 6, 7], 15), // vsync at char 2 (scanline 4), width=4
    ];

    for (r7_vertical_sync_position, r3_sync_width, expected_indices, length) in test_cases {
        let mut registers = create_default_registers();
        registers.crtc_r9_maximum_raster_address = 1;
        registers.crtc_r7_vertical_sync_position = r7_vertical_sync_position;
        registers.crtc_r3_sync_width = r3_sync_width;
        registers.crtc_r4_vertical_total = 10;

        let mut crtc = setup_test(&registers);

        for index in 0..length {
            let actual = crtc.advance_scanline(&registers).vsync;
            let expected = expected_indices.contains(&index);

            assert_eq!(
                actual, expected,
                "vsync mismatch at index {index} for r7_vertical_sync_position={}, r3_sync_width={:#04x}",
                r7_vertical_sync_position, r3_sync_width
            );
        }
    }
}

#[test]
fn should_complete_frame_at_correct_boundary() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1; // 2 scanlines per char line
    registers.crtc_r4_vertical_total = 2; // 3 char lines (0-2)
    registers.crtc_r5_vertical_total_adjust = 0;

    let mut crtc = setup_test(&registers);

    // Frame = 3 char lines * 2 scanlines = 6 scanlines total.
    // Address increments by 0x50 (default horizontal_displayed=80) per char line.
    // At scanline 6, frame completes and address resets to start (0x2000)
    let expected_addresses = [
        0x2000, 0x2000, 0x2050, 0x2050, 0x20a0, 0x20a0, 0x2000, 0x2000,
    ];

    for (index, expected) in expected_addresses.iter().enumerate() {
        let actual = crtc.advance_scanline(&registers).snapshot_params.address;

        assert_eq!(actual, *expected, "mismatch at index {index}");
    }
}

#[test]
fn should_reset_scanline_after_field_complete() {
    let registers = create_default_registers();
    let mut crtc = setup_test(&registers);

    // Advance to iteration 160 where field_complete should occur
    for iteration in 0..160 {
        let result = crtc.advance_scanline(&registers);
        assert!(
            !result.field_complete,
            "field_complete should be false before iteration 160 ({iteration})"
        );
    }

    // At iteration 160: field_complete should be true and scanline should be 160
    let result_at_160 = crtc.advance_scanline(&registers);
    assert!(
        result_at_160.field_complete,
        "field_complete should be true at iteration 160"
    );
    assert_eq!(
        result_at_160.snapshot_params.beam_scanline, 160,
        "scanline should be 160 at field_complete"
    );

    // At iteration 161: scanline should reset to 0
    let result_at_161 = crtc.advance_scanline(&registers);
    assert_eq!(
        result_at_161.snapshot_params.beam_scanline, 0,
        "scanline should reset to 0 after field_complete"
    );
}

#[test]
fn should_respect_vertical_total_adjust_when_completing_frames() {
    let total_adjust_cases = [
        // name, r8, r9
        ("normal", 0x00, 1),
        ("interlace sync and video", 0x03, 2),
    ];

    for (mode_name, r8_interlace_and_skew, r9_maximum_raster_address) in total_adjust_cases {
        let mut registers = create_default_registers();
        registers.crtc_r1_horizontal_displayed = 50;
        registers.crtc_r4_vertical_total = 2;
        registers.crtc_r5_vertical_total_adjust = 3;
        registers.crtc_r6_vertical_displayed = 2;
        registers.crtc_r7_vertical_sync_position = 0;
        registers.crtc_r8_interlace_and_skew = r8_interlace_and_skew;
        registers.crtc_r9_maximum_raster_address = r9_maximum_raster_address;
        registers.ula_control = 0x00;

        let mut crtc = setup_test(&registers);

        let expected = [
            (0x2000, true),  // char line 0, raster 0
            (0x2000, true),  // char line 0, raster 1
            (0x2032, true),  // char line 1, raster 0
            (0x2032, true),  // char line 1, raster 1
            (0x2064, false), // char line 2, raster 0
            (0x2064, false), // char line 2, raster 1
            (0x2096, false), // adjust scanline 0 (address frozen)
            (0x2096, false), // adjust scanline 1 (address frozen)
            (0x2096, false), // adjust scanline 2 (raster wraps, frame resets after)
            (0x2000, true),
        ];

        for (index, (expected_address, expected_in_scan)) in expected.iter().enumerate() {
            let actual = crtc.advance_scanline(&registers).snapshot_params;

            assert_eq!(
                actual.address, *expected_address,
                "{mode_name}: address mismatch at index {index}",
            );
            assert_eq!(
                actual.in_scan, *expected_in_scan,
                "{mode_name}: in_scan mismatch at index {index}",
            );
        }
    }
}

#[test]
fn should_maintain_consistent_next_scanline_trigger_in_both_frequency_modes() {
    // ula_control bit 4 selects frequency: 0=normal, 1=high
    // Trigger = (horizontal_total + 1) * clock_divisor
    // Normal mode: (127+1) * 2 = 256, High freq: (127+1) * 1 = 128
    let test_cases = [(0x00, 256u16), (0x10, 128u16)];

    for (ula_control, expected_trigger) in test_cases {
        let mut registers = create_default_registers();
        registers.crtc_r0_horizontal_total = 127;
        registers.ula_control = ula_control;

        let mut crtc = setup_test(&registers);

        let expected_triggers = [expected_trigger; 20];

        for (index, expected) in expected_triggers.iter().enumerate() {
            let actual = crtc.advance_scanline(&registers).next_scanline_trigger;

            assert_eq!(actual, *expected, "mismatch at index {index}");
        }
    }
}

#[test]
fn field_complete_should_only_occur_at_boundaries() {
    let registers = create_default_registers();
    let mut crtc = setup_test(&registers);

    // With default registers: 20 char lines displayed * 8 scanlines/char = 160 scanlines
    // field_complete should occur exactly once at scanline 160
    for iteration in 0..165 {
        let actual_field_complete = crtc.advance_scanline(&registers).field_complete;
        let expected_field_complete = iteration == 160;

        assert_eq!(
            actual_field_complete, expected_field_complete,
            "mismatch at iteration {iteration}"
        );
    }
}

#[test]
fn address_should_increment_by_horizontal_displayed_per_char_line() {
    let mut registers = create_default_registers();
    registers.crtc_r1_horizontal_displayed = 0x28;
    registers.crtc_r9_maximum_raster_address = 1; // 2 scanlines per char line
    registers.crtc_r4_vertical_total = 10;

    let mut crtc = setup_test(&registers);

    // Address advances by horizontal_displayed (0x28) after each char line completes
    let expected_addresses = [0x2000, 0x2000, 0x2028, 0x2028, 0x2050, 0x2050];

    for (index, expected) in expected_addresses.iter().enumerate() {
        let actual = crtc.advance_scanline(&registers).snapshot_params.address;

        assert_eq!(actual, *expected, "mismatch at index {index}");
    }
}

#[test]
fn raster_address_odd_should_equal_even_plus_one_when_interlaced() {
    let mut registers = create_default_registers();
    registers.crtc_r8_interlace_and_skew = 0x03; // Interlace mode enabled
    registers.crtc_r9_maximum_raster_address = 6; // 8 scanlines per field

    let mut crtc = setup_test(&registers);

    // In interlace mode, odd field raster addresses are offset by +1 from even field

    let expected = [
        // raster_address_even, raster_address_odd
        (0, 1),
        (2, 3),
        (4, 5),
        (6, 7),
        (0, 1),
        (2, 3),
        (4, 5),
        (6, 7),
        (0, 1),
        (2, 3),
    ];

    for (index, (expected_even, expected_odd)) in expected.iter().enumerate() {
        let actual = crtc.advance_scanline(&registers).snapshot_params;

        assert_eq!(
            actual.raster_address_even, *expected_even,
            "raster_address_even mismatch at index {index}",
        );
        assert_eq!(
            actual.raster_address_odd, *expected_odd,
            "raster_address_odd mismatch at index {index}",
        );
    }
}

#[test]
fn raster_address_odd_should_equal_even_when_not_interlaced() {
    let mut registers = create_default_registers();
    registers.crtc_r8_interlace_and_skew = 0x00;

    let mut crtc = setup_test(&registers);

    let expected = [
        // raster_address_even, raster_address_odd
        (0, 0),
        (1, 1),
        (2, 2),
        (3, 3),
        (4, 4),
        (5, 5),
        (6, 6),
        (7, 7),
        (0, 0),
        (1, 1),
    ];

    for (index, (expected_even, expected_odd)) in expected.iter().enumerate() {
        let actual = crtc.advance_scanline(&registers).snapshot_params;

        assert_eq!(
            actual.raster_address_even, *expected_even,
            "raster_address_even mismatch at index {index}",
        );
        assert_eq!(
            actual.raster_address_odd, *expected_odd,
            "raster_address_odd mismatch at index {index}",
        );
    }
}

#[test]
fn address_should_only_update_from_start_registers_at_frame_boundary() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1;
    registers.crtc_r4_vertical_total = 2;
    registers.crtc_r5_vertical_total_adjust = 0;
    registers.crtc_r1_horizontal_displayed = 50;
    registers.crtc_r12_start_address_h = 0x20;
    registers.crtc_r13_start_address_l = 0x00;
    registers.crtc_r7_vertical_sync_position = 10;

    let mut crtc = setup_test(&registers);

    let first_expected_addresses = [
        0x2000, 0x2000, 0x2032, 0x2032, 0x2064, 0x2064, 0x2000, 0x2000,
    ];

    for (index, expected) in first_expected_addresses.iter().enumerate() {
        let actual = crtc.advance_scanline(&registers).snapshot_params.address;

        assert_eq!(
            actual, *expected,
            "mismatch at index {index} (first sequence)"
        );
    }

    // Change start address register mid-test
    registers.crtc_r12_start_address_h = 0x30;

    // Start address register change takes effect only at next frame boundary.
    // Continues at 0x2032, 0x2064, then resets to new start 0x3000 at frame boundary
    let second_expected_addresses = [0x2032, 0x2032, 0x2064, 0x2064, 0x3000, 0x3000];

    for (index, expected) in second_expected_addresses.iter().enumerate() {
        let actual = crtc.advance_scanline(&registers).snapshot_params.address;

        assert_eq!(
            actual, *expected,
            "mismatch at index {index} (second sequence)"
        );
    }
}

#[test]
fn should_handle_scan_lines_per_char_zero() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 0; // Edge case: 1 scanline per char
    registers.crtc_r6_vertical_displayed = 2;
    registers.crtc_r4_vertical_total = 5;

    let mut crtc = setup_test(&registers);

    let expected_addresses = [0, 0, 0, 0, 0];

    for (index, expected) in expected_addresses.iter().enumerate() {
        let actual = crtc
            .advance_scanline(&registers)
            .snapshot_params
            .raster_address_even;

        assert_eq!(actual, *expected, "mismatch at index {index}");
    }
}

#[test]
fn should_handle_minimal_display_area() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1;
    registers.crtc_r6_vertical_displayed = 1;
    registers.crtc_r4_vertical_total = 5;

    let mut crtc = setup_test(&registers);

    let expected_pattern = [true, true, false, false, false];

    for (index, expected) in expected_pattern.iter().enumerate() {
        let actual = crtc.advance_scanline(&registers).snapshot_params.in_scan;

        assert_eq!(actual, *expected, "mismatch at index {index}");
    }
}

#[test]
fn should_handle_horizontal_total_zero() {
    let mut registers = create_default_registers();
    registers.crtc_r0_horizontal_total = 0; // Edge case: minimum value
    registers.ula_control = 0x00;

    let mut crtc = setup_test(&registers);

    let actual = crtc.advance_scanline(&registers);

    // Trigger = (0 + 1) * 2 = 2
    assert_eq!(actual.next_scanline_trigger, 2);
}

#[test]
fn should_handle_horizontal_total_255() {
    let mut registers = create_default_registers();
    registers.crtc_r0_horizontal_total = 255; // Edge case: maximum value
    registers.ula_control = 0x10; // High frequency mode

    let mut crtc = setup_test(&registers);

    let actual = crtc.advance_scanline(&registers);

    // Trigger = (255 + 1) * 1 = 256 (high freq mode uses divisor 1)
    assert_eq!(actual.next_scanline_trigger, 256);
}

#[test]
fn should_maintain_consistent_trigger_values_across_multiple_frames() {
    let mut registers = create_default_registers();
    registers.crtc_r0_horizontal_total = 127;
    registers.ula_control = 0x00;
    registers.crtc_r9_maximum_raster_address = 1; // 2 scanlines per char
    registers.crtc_r4_vertical_total = 2; // 3 char lines
    registers.crtc_r5_vertical_total_adjust = 0;

    let mut crtc = setup_test(&registers);

    // Frame = 3 char lines * 2 scanlines = 6 scanlines. Test 17 scanlines = 2.8 frames
    // next_scanline_trigger should remain constant (256) throughout
    let expected_triggers = [256u16; 17];

    for (index, expected) in expected_triggers.iter().enumerate() {
        let actual = crtc.advance_scanline(&registers).next_scanline_trigger;

        assert_eq!(actual, *expected, "mismatch at index {index}");
    }
}

#[test]
fn should_show_pattern_stability_across_multiple_complete_frames() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1; // 2 scanlines per char
    registers.crtc_r4_vertical_total = 2; // 3 char lines
    registers.crtc_r5_vertical_total_adjust = 0;
    registers.crtc_r1_horizontal_displayed = 50;

    let mut crtc = setup_test(&registers);

    // Frame = 3 char lines * 2 scanlines = 6 scanlines.
    // Test 17 scanlines = 2 complete frames + 5 scanlines into 3rd frame.
    // Scanline counter increments continuously without resetting
    let expected_scanlines = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    for (index, expected) in expected_scanlines.iter().enumerate() {
        let actual = crtc
            .advance_scanline(&registers)
            .snapshot_params
            .beam_scanline;

        assert_eq!(actual, *expected, "mismatch at index {index}");
    }
}

#[test]
fn should_toggle_interlace_frame_and_double_trigger_on_alternate_frames() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1;
    registers.crtc_r4_vertical_total = 1; // 2 char lines per frame
    registers.crtc_r5_vertical_total_adjust = 0;
    registers.crtc_r0_horizontal_total = 127;
    registers.ula_control = 0x00;
    registers.crtc_r8_interlace_and_skew = 0x01; // Interlace sync enabled
    registers.crtc_r7_vertical_sync_position = 0;

    let mut crtc = setup_test(&registers);

    // Frame = 2 char lines * 2 scanlines = 4 scanlines.
    // In interlace sync mode, every 4th scanline (at char line 1 of odd frames)
    // gets double trigger (512) for half-line offset. Pattern: 256,256,256,512,...
    let expected_triggers = [
        256, 256, 256, 512, 256, 256, 256, 256, 256, 256, 256, 512, 256,
    ];

    for (index, expected) in expected_triggers.iter().enumerate() {
        let actual = crtc.advance_scanline(&registers).next_scanline_trigger;

        assert_eq!(actual, *expected, "mismatch at index {index}");
    }
}
