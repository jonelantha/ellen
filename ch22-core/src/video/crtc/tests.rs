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

// ============================================================================
// BASIC SCANLINE BEHAVIOR
// ============================================================================

#[test]
fn should_increment_scanline_by_1_on_each_call() {
    let registers = create_default_registers();
    let mut crtc = Crtc::default();
    crtc.init(&registers);

    let expected_scanlines: Vec<u16> = (0..50).map(|i| i as u16).collect();

    for (index, expected) in expected_scanlines.iter().enumerate() {
        let actual = crtc.get_snapshot_params(&registers).beam_scanline;

        assert_eq!(actual, *expected, "mismatch at index {index}");

        crtc.advance_scanline(&registers);
    }
}

// ============================================================================
// ADDRESS BEHAVIOR
// ============================================================================

#[test]
fn should_advance_address_by_horizontal_displayed_after_char_scanlines() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 7; // 8 scanlines per character line (0-7)
    registers.crtc_r1_horizontal_displayed = 50; // 0x32 in hex

    let mut crtc = Crtc::default();
    crtc.init(&registers);

    // Address stays at 0x2000 for all 8 scanlines of first character line,
    // then advances by horizontal_displayed (0x32) to 0x2032 on scanline 8
    let expected_addresses = [
        0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2032,
    ];

    for (index, expected) in expected_addresses.iter().enumerate() {
        let actual = crtc.get_snapshot_params(&registers).address;

        assert_eq!(actual, *expected, "mismatch at index {index}");

        crtc.advance_scanline(&registers);
    }
}

#[test]
fn should_stop_incrementing_addr_after_char_line_exceeds_vertical_total() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1; // 2 scanlines per character line
    registers.crtc_r4_vertical_total = 2; // 3 character lines total (0-2)
    registers.crtc_r1_horizontal_displayed = 50; // 0x32 per line
    registers.crtc_r6_vertical_displayed = 2; // 2 character lines visible
    registers.crtc_r7_vertical_sync_position = 10;

    let mut crtc = Crtc::default();
    crtc.init(&registers);

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
        let actual = crtc.get_snapshot_params(&registers);

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

        crtc.advance_scanline(&registers);
    }
}

// ============================================================================
// RASTER ADDRESS BEHAVIOR
// ============================================================================

#[test]
fn should_track_raster_address_progression_correctly() {
    // Test raster_address_even cycles correctly with various r9 values
    let test_cases = [
        (1, [0, 1, 0, 1, 0, 1, 0, 1, 0, 1]),
        (3, [0, 1, 2, 3, 0, 1, 2, 3, 0, 1]),
        (7, [0, 1, 2, 3, 4, 5, 6, 7, 0, 1]),
    ];

    for (r9_value, expected_pattern) in test_cases {
        let mut registers = create_default_registers();
        registers.crtc_r9_maximum_raster_address = r9_value;
        registers.crtc_r8_interlace_and_skew = 0x00;

        let mut crtc = Crtc::default();
        crtc.init(&registers);

        for (index, expected) in expected_pattern.iter().enumerate() {
            let actual = crtc.get_snapshot_params(&registers).raster_address_even;

            assert_eq!(
                actual, *expected,
                "mismatch at index {index} for r9={}",
                r9_value
            );

            crtc.advance_scanline(&registers);
        }
    }
}

// ============================================================================
// VSYNC BEHAVIOR
// ============================================================================

#[test]
fn should_trigger_vsync_at_correct_positions() {
    // Test vsync triggering: vsync starts at char_line * scanlines_per_char,
    // continues for (sync_width >> 4) scanlines.
    // With r9=1 (2 scanlines/char): char_line N starts at scanline N*2
    let test_cases = [
        // r7_vertical_sync_position, r3_sync_width, expected_indices, length
        (1, 0x00, vec![], 10), // vsync at char 1 (scanline 2), width=0: no vsync
        (2, 0x10, vec![5], 15), // vsync at char 2 (scanline 4), width=1
        (2, 0x20, vec![5, 6], 20), // vsync at char 2 (scanline 4), width=2
        (3, 0x20, vec![7, 8], 20), // vsync at char 3 (scanline 6), width=2
        (2, 0x30, vec![5, 6, 7], 20), // vsync at char 2 (scanline 4), width=3
        (2, 0x40, vec![5, 6, 7, 8], 15), // vsync at char 2 (scanline 4), width=4
    ];

    for (r7_vertical_sync_position, r3_sync_width, expected_indices, length) in test_cases {
        let mut registers = create_default_registers();
        registers.crtc_r9_maximum_raster_address = 1;
        registers.crtc_r7_vertical_sync_position = r7_vertical_sync_position;
        registers.crtc_r3_sync_width = r3_sync_width;
        registers.crtc_r4_vertical_total = 10;

        let mut crtc = Crtc::default();
        crtc.init(&registers);

        for index in 0..length {
            let actual = crtc.is_in_vsync();
            let expected = expected_indices.contains(&index);

            assert_eq!(
                actual, expected,
                "vsync mismatch at index {index} for r7_vertical_sync_position={}, r3_sync_width={:#04x}",
                r7_vertical_sync_position, r3_sync_width
            );

            crtc.advance_scanline(&registers);
        }
    }
}

// ============================================================================
// FRAME AND SCAN BEHAVIOR
// ============================================================================

#[test]
fn should_transition_in_scan_from_true_to_false_at_display_boundary() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1; // 2 scanlines per character line
    registers.crtc_r6_vertical_displayed = 2; // 2 character lines displayed
    registers.crtc_r4_vertical_total = 10;

    let mut crtc = Crtc::default();
    crtc.init(&registers);

    // in_scan is true for 2 char lines * 2 scanlines = 4 scanlines, then false
    let expected_pattern = [true, true, true, true, false, false, false];

    for (index, expected) in expected_pattern.iter().enumerate() {
        let actual = crtc.get_snapshot_params(&registers).in_scan;

        assert_eq!(actual, *expected, "mismatch at index {index}");

        crtc.advance_scanline(&registers);
    }
}

#[test]
fn should_return_beam_scanline_0_at_max_lines() {
    let mut registers = create_default_registers();
    // Set vsync position to max to prevent normal field completion,
    // forcing the hardware limit (MAX_LINES) to trigger instead
    registers.crtc_r7_vertical_sync_position = 255;

    let mut crtc = Crtc::default();
    crtc.init(&registers);

    // Skip iteration 0 (initial state with beam_scanline == 0)
    crtc.advance_scanline(&registers);

    let mut beam_scanline_0_iteration: i32 = -1;
    for i in 1..(crate::video::MAX_LINES + 10) {
        let beam_scanline = crtc.get_snapshot_params(&registers).beam_scanline;

        if beam_scanline == 0 {
            beam_scanline_0_iteration = i as i32;
            break;
        }

        crtc.advance_scanline(&registers);
    }

    assert_eq!(
        beam_scanline_0_iteration,
        crate::video::MAX_LINES as i32,
        "beam scanline should be 0 at iteration MAX_LINES (after hardware limit reset)",
    );
}

#[test]
fn should_complete_frame_at_correct_boundary() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1; // 2 scanlines per char line
    registers.crtc_r4_vertical_total = 2; // 3 char lines (0-2)
    registers.crtc_r5_vertical_total_adjust = 0;

    let mut crtc = Crtc::default();
    crtc.init(&registers);

    // Frame = 3 char lines * 2 scanlines = 6 scanlines total.
    // Address increments by 0x50 (default horizontal_displayed=80) per char line.
    // At scanline 6, frame completes and address resets to start (0x2000)
    let expected_addresses = [
        0x2000, 0x2000, 0x2050, 0x2050, 0x20a0, 0x20a0, 0x2000, 0x2000,
    ];

    for (index, expected) in expected_addresses.iter().enumerate() {
        let actual = crtc.get_snapshot_params(&registers).address;

        assert_eq!(actual, *expected, "mismatch at index {index}");

        crtc.advance_scanline(&registers);
    }
}

// ============================================================================
// BEAM RESET BEHAVIOR
// ============================================================================

#[test]
fn should_reset_scanline_after_beam_reset() {
    let registers = create_default_registers();
    let mut crtc = Crtc::default();
    crtc.init(&registers);

    assert!(
        crtc.is_beam_reset(),
        "is_beam_reset should be true for first iteration"
    );

    crtc.advance_scanline(&registers);

    // Advance to iteration 160 where next beam_reset should occur
    for iteration in 0..160 {
        let actual = crtc.is_beam_reset();

        assert_eq!(
            actual, false,
            "is_beam_reset should be false before iteration 160 ({iteration})"
        );

        crtc.advance_scanline(&registers);
    }

    // At iteration 160: is_beam_reset should be true and scanline should be 0
    let snapshot_params_at_160 = crtc.get_snapshot_params(&registers);
    let beam_reset_at_160 = crtc.is_beam_reset();

    assert_eq!(
        beam_reset_at_160, true,
        "is_beam_reset should be true at iteration 160"
    );
    assert_eq!(
        snapshot_params_at_160.beam_scanline, 0,
        "scanline should be 0 at beam_reset"
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

        let mut crtc = Crtc::default();
        crtc.init(&registers);

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
            let actual = crtc.get_snapshot_params(&registers);

            assert_eq!(
                actual.address, *expected_address,
                "{mode_name}: address mismatch at index {index}",
            );
            assert_eq!(
                actual.in_scan, *expected_in_scan,
                "{mode_name}: in_scan mismatch at index {index}",
            );

            crtc.advance_scanline(&registers);
        }
    }
}

// ============================================================================
// TRIGGER AND FREQUENCY BEHAVIOR
// ============================================================================

#[test]
fn should_maintain_consistent_next_scanline_trigger_in_both_frequency_modes() {
    // ula_control bit 4 selects frequency: 0=normal, 1=high
    // Trigger = (horizontal_total + 1) * clock_divisor
    // Normal mode: (127+1) * 2 = 256, High freq: (127+1) * 1 = 128
    let frequency_test_cases = [(0x00, 256u16), (0x10, 128u16)];

    for (ula_control, expected_trigger) in frequency_test_cases {
        let mut registers = create_default_registers();
        registers.crtc_r0_horizontal_total = 127;
        registers.ula_control = ula_control;

        let mut crtc = Crtc::default();
        crtc.init(&registers);

        let expected_triggers = [expected_trigger; 20];

        for (index, expected) in expected_triggers.iter().enumerate() {
            let actual = crtc.get_next_scanline_trigger(&registers);

            assert_eq!(
                actual, *expected,
                "mismatch at index {index} for ula_control={:#04x}",
                ula_control
            );

            crtc.advance_scanline(&registers);
        }
    }
}

#[test]
fn beam_reset_should_only_occur_at_boundaries() {
    let registers = create_default_registers();
    let mut crtc = Crtc::default();
    crtc.init(&registers);

    // With default registers: 20 char lines displayed * 8 scanlines/char = 160 scanlines
    // beam_reset should occur at 0 and 161
    for iteration in 0..165 {
        let actual_beam_reset = crtc.is_beam_reset();
        let expected_beam_reset = iteration == 0 || iteration == 161;

        assert_eq!(
            actual_beam_reset, expected_beam_reset,
            "mismatch at iteration {iteration}"
        );

        crtc.advance_scanline(&registers);
    }
}

#[test]
fn raster_address_odd_should_equal_even_plus_one_when_interlaced() {
    let mut registers = create_default_registers();
    registers.crtc_r8_interlace_and_skew = 0x03; // Interlace mode enabled
    registers.crtc_r9_maximum_raster_address = 6; // 8 scanlines per field

    let mut crtc = Crtc::default();
    crtc.init(&registers);

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
        let actual = crtc.get_snapshot_params(&registers);

        assert_eq!(
            actual.raster_address_even, *expected_even,
            "raster_address_even mismatch at index {index}",
        );
        assert_eq!(
            actual.raster_address_odd, *expected_odd,
            "raster_address_odd mismatch at index {index}",
        );

        crtc.advance_scanline(&registers);
    }
}

#[test]
fn raster_address_odd_should_equal_even_when_not_interlaced() {
    let mut registers = create_default_registers();
    registers.crtc_r8_interlace_and_skew = 0x00;

    let mut crtc = Crtc::default();
    crtc.init(&registers);

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
        let actual = crtc.get_snapshot_params(&registers);

        assert_eq!(
            actual.raster_address_even, *expected_even,
            "raster_address_even mismatch at index {index}",
        );
        assert_eq!(
            actual.raster_address_odd, *expected_odd,
            "raster_address_odd mismatch at index {index}",
        );

        crtc.advance_scanline(&registers);
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

    let mut crtc = Crtc::default();
    crtc.init(&registers);

    let first_expected_addresses = [
        0x2000, 0x2000, 0x2032, 0x2032, 0x2064, 0x2064, 0x2000, 0x2000,
    ];

    for (index, expected) in first_expected_addresses.iter().enumerate() {
        let actual = crtc.get_snapshot_params(&registers).address;

        assert_eq!(
            actual, *expected,
            "mismatch at index {index} (first sequence)"
        );

        crtc.advance_scanline(&registers);
    }

    // Change start address register mid-test
    registers.crtc_r12_start_address_h = 0x30;

    // Start address register change takes effect only at next frame boundary.
    // Continues at 0x2032, 0x2064, then resets to new start 0x3000 at frame boundary
    let second_expected_addresses = [0x2032, 0x2032, 0x2064, 0x2064, 0x3000, 0x3000];

    for (index, expected) in second_expected_addresses.iter().enumerate() {
        let actual = crtc.get_snapshot_params(&registers).address;

        assert_eq!(
            actual, *expected,
            "mismatch at index {index} (second sequence)"
        );

        crtc.advance_scanline(&registers);
    }
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn should_handle_scan_lines_per_char_zero() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 0; // Edge case: 1 scanline per char
    registers.crtc_r6_vertical_displayed = 2;
    registers.crtc_r4_vertical_total = 5;

    let mut crtc = Crtc::default();
    crtc.init(&registers);

    let expected_addresses = [0, 0, 0, 0, 0];

    for (index, expected) in expected_addresses.iter().enumerate() {
        let actual = crtc.get_snapshot_params(&registers).raster_address_even;

        assert_eq!(actual, *expected, "mismatch at index {index}");

        crtc.advance_scanline(&registers);
    }
}

#[test]
fn should_handle_minimal_display_area() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1;
    registers.crtc_r6_vertical_displayed = 1;
    registers.crtc_r4_vertical_total = 5;

    let mut crtc = Crtc::default();
    crtc.init(&registers);

    let expected_pattern = [true, true, false, false, false];

    for (index, expected) in expected_pattern.iter().enumerate() {
        let actual = crtc.get_snapshot_params(&registers).in_scan;

        assert_eq!(actual, *expected, "mismatch at index {index}");

        crtc.advance_scanline(&registers);
    }
}

#[test]
fn should_handle_horizontal_total_zero() {
    let mut registers = create_default_registers();
    registers.crtc_r0_horizontal_total = 0; // Edge case: minimum value
    registers.ula_control = 0x00;

    let mut crtc = Crtc::default();
    crtc.init(&registers);

    let actual = crtc.get_next_scanline_trigger(&registers);

    // Trigger = (0 + 1) * 2 = 2
    assert_eq!(actual, 2);
}

#[test]
fn should_handle_horizontal_total_255() {
    let mut registers = create_default_registers();
    registers.crtc_r0_horizontal_total = 255; // Edge case: maximum value
    registers.ula_control = 0x10; // High frequency mode

    let mut crtc = Crtc::default();
    crtc.init(&registers);

    let actual = crtc.get_next_scanline_trigger(&registers);

    // Trigger = (255 + 1) * 1 = 256 (high freq mode uses divisor 1)
    assert_eq!(actual, 256);
}

#[test]
fn should_maintain_consistent_trigger_values_across_multiple_frames() {
    let mut registers = create_default_registers();
    registers.crtc_r0_horizontal_total = 127;
    registers.ula_control = 0x00;
    registers.crtc_r9_maximum_raster_address = 1; // 2 scanlines per char
    registers.crtc_r4_vertical_total = 2; // 3 char lines
    registers.crtc_r5_vertical_total_adjust = 0;

    let mut crtc = Crtc::default();
    crtc.init(&registers);

    // Frame = 3 char lines * 2 scanlines = 6 scanlines. Test 17 scanlines = 2.8 frames
    // next_scanline_trigger should remain constant (256) throughout
    let expected_triggers = [256u16; 17];

    for (index, expected) in expected_triggers.iter().enumerate() {
        let actual = crtc.get_next_scanline_trigger(&registers);

        assert_eq!(actual, *expected, "mismatch at index {index}");

        crtc.advance_scanline(&registers);
    }
}

// ============================================================================
// INTERLACE BEHAVIOR
// ============================================================================

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

    let mut crtc = Crtc::default();
    crtc.init(&registers);

    // Frame = 2 char lines * 2 scanlines = 4 scanlines.
    // In interlace sync mode, every 4th scanline (at char line 1 of odd frames)
    // gets double trigger (512) for half-line offset. Pattern: 256,256,256,512,...
    let expected_triggers = [
        256, 256, 256, 256, 512, 256, 256, 256, 256, 256, 256, 256, 512, 256,
    ];

    for (index, expected) in expected_triggers.iter().enumerate() {
        let actual = crtc.get_next_scanline_trigger(&registers);

        assert_eq!(actual, *expected, "mismatch at index {index}");

        crtc.advance_scanline(&registers);
    }
}
