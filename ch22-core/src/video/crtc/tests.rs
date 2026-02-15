use std::cell::RefCell;
use std::rc::Rc;

use super::{AdvanceScanlineResult, CRTC, SnapshotParams};
use crate::video::VideoRegisters;

#[derive(Default)]
struct ExpectedSnapshot {
    in_scan: Option<bool>,
    scanline: Option<u16>,
    address: Option<u16>,
    raster_address_even: Option<u8>,
    raster_address_odd: Option<u8>,
}

fn assert_snapshot_matches(actual: &SnapshotParams, expected: &ExpectedSnapshot) {
    if let Some(value) = expected.in_scan {
        assert_eq!(actual.in_scan, value, "in_scan mismatch");
    }
    if let Some(value) = expected.scanline {
        assert_eq!(actual.scanline, value, "scanline mismatch");
    }
    if let Some(value) = expected.address {
        assert_eq!(actual.address, value, "address mismatch");
    }
    if let Some(value) = expected.raster_address_even {
        assert_eq!(actual.raster_address_even, value, "raster_address_even mismatch");
    }
    if let Some(value) = expected.raster_address_odd {
        assert_eq!(actual.raster_address_odd, value, "raster_address_odd mismatch");
    }
}

fn create_default_registers() -> VideoRegisters {
    let mut registers = VideoRegisters::default();

    registers.crtc_r0_horizontal_total = 127;
    registers.crtc_r1_horizontal_displayed = 80;
    registers.crtc_r3_sync_width = 0x20;
    registers.crtc_r4_vertical_total = 24;
    registers.crtc_r5_vertical_total_adjust = 0;
    registers.crtc_r6_vertical_displayed = 20;
    registers.crtc_r7_vertical_sync_position = 20;
    registers.crtc_r8_interlace_and_skew = 0x00;
    registers.crtc_r9_maximum_raster_address = 7;
    registers.crtc_r12_start_address_h = 0x20;
    registers.crtc_r13_start_address_l = 0x00;
    registers.ula_control = 0x00;

    registers
}

fn setup_test(registers: Rc<RefCell<VideoRegisters>>) -> CRTC {
    let mut crtc = CRTC::new(registers);
    crtc.init();
    crtc
}

fn collect_results(crtc: &mut CRTC, iterations: usize) -> Vec<AdvanceScanlineResult> {
    (0..iterations).map(|_| crtc.advance_scanline()).collect()
}

#[test]
fn should_increment_scanline_by_1_on_each_call() {
    let registers = Rc::new(RefCell::new(create_default_registers()));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 10);
    let scanlines: Vec<u16> = results.iter().map(|r| r.snapshot_params.scanline).collect();

    let expected_scanlines: Vec<u16> = (0..10).map(|i| i as u16).collect();

    assert_eq!(scanlines, expected_scanlines);
}

#[test]
fn should_cycle_raster_address_even_pattern_after_char_scanlines_calls() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 7;
    registers.crtc_r1_horizontal_displayed = 80;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 16);
    let raster_pattern: Vec<u8> = results
        .iter()
        .map(|r| r.snapshot_params.raster_address_even)
        .collect();

    let expected_pattern: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7];

    assert_eq!(raster_pattern, expected_pattern);
}

#[test]
fn should_advance_address_by_horizontal_displayed_after_char_scanlines() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 7;
    registers.crtc_r1_horizontal_displayed = 50;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 9);
    let addresses: Vec<u16> = results.iter().map(|r| r.snapshot_params.address).collect();

    let expected_addresses: Vec<u16> = vec![
        0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2032,
    ];

    assert_eq!(addresses, expected_addresses);
}

#[test]
fn should_transition_in_scan_from_true_to_false_at_display_boundary() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1;
    registers.crtc_r6_vertical_displayed = 2;
    registers.crtc_r4_vertical_total = 10;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 7);
    let in_scan_values: Vec<bool> = results.iter().map(|r| r.snapshot_params.in_scan).collect();

    let expected_pattern = vec![true, true, true, true, false, false, false];

    assert_eq!(in_scan_values, expected_pattern);
}

#[test]
fn should_return_field_complete_true_at_max_lines() {
    let mut registers = create_default_registers();
    registers.crtc_r7_vertical_sync_position = 255;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let mut field_complete_iteration: i32 = -1;
    for i in 0..(crate::video::MAX_LINES + 10) {
        let result = crtc.advance_scanline();
        if result.field_complete {
            field_complete_iteration = i as i32;
            break;
        }
    }

    assert_eq!(
        field_complete_iteration,
        crate::video::MAX_LINES as i32 - 1,
        "field_complete should occur at iteration {}",
        crate::video::MAX_LINES - 1
    );
}

#[test]
fn should_stop_incrementing_addr_after_char_line_exceeds_vertical_total() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1;
    registers.crtc_r4_vertical_total = 2;
    registers.crtc_r1_horizontal_displayed = 50;
    registers.crtc_r6_vertical_displayed = 2;
    registers.crtc_r7_vertical_sync_position = 10;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 10);

    let expected = vec![
        ExpectedSnapshot {
            address: Some(0x2000),
            in_scan: Some(true),
            scanline: Some(0),
            raster_address_even: Some(0),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2000),
            in_scan: Some(true),
            scanline: Some(1),
            raster_address_even: Some(1),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2032),
            in_scan: Some(true),
            scanline: Some(2),
            raster_address_even: Some(0),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2032),
            in_scan: Some(true),
            scanline: Some(3),
            raster_address_even: Some(1),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2064),
            in_scan: Some(false),
            scanline: Some(4),
            raster_address_even: Some(0),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2064),
            in_scan: Some(false),
            scanline: Some(5),
            raster_address_even: Some(1),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2000),
            in_scan: Some(true),
            scanline: Some(6),
            raster_address_even: Some(0),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2000),
            in_scan: Some(true),
            scanline: Some(7),
            raster_address_even: Some(1),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2032),
            in_scan: Some(true),
            scanline: Some(8),
            raster_address_even: Some(0),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2032),
            in_scan: Some(true),
            scanline: Some(9),
            raster_address_even: Some(1),
            ..ExpectedSnapshot::default()
        },
    ];

    for (actual, expected) in results.iter().zip(expected.iter()) {
        assert_snapshot_matches(&actual.snapshot_params, expected);
    }
}

#[test]
fn should_maintain_consistent_scanline_delta() {
    let registers = Rc::new(RefCell::new(create_default_registers()));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 50);
    let scanlines: Vec<u16> = results.iter().map(|r| r.snapshot_params.scanline).collect();

    let expected_scanlines: Vec<u16> = (0..50).map(|i| i as u16).collect();

    assert_eq!(scanlines, expected_scanlines);
}

#[test]
fn should_track_raster_address_progression_correctly() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 3;
    registers.crtc_r8_interlace_and_skew = 0x00;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 12);
    let raster_addresses: Vec<u8> = results
        .iter()
        .map(|r| r.snapshot_params.raster_address_even)
        .collect();

    let expected_pattern: Vec<u8> = vec![0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3];

    assert_eq!(raster_addresses, expected_pattern);
}

#[test]
fn should_trigger_vsync_at_correct_positions() {
    let test_cases = vec![
        (1, 0x00, vec![], 10),
        (2, 0x10, vec![4], 15),
        (2, 0x20, vec![4, 5], 20),
        (3, 0x20, vec![6, 7], 20),
        (2, 0x30, vec![4, 5, 6], 20),
        (2, 0x40, vec![4, 5, 6, 7], 15),
    ];

    for (r7_vertical_sync_position, r3_sync_width, expected_indices, length) in test_cases {
        let mut registers = create_default_registers();
        registers.crtc_r9_maximum_raster_address = 1;
        registers.crtc_r7_vertical_sync_position = r7_vertical_sync_position;
        registers.crtc_r3_sync_width = r3_sync_width;
        registers.crtc_r4_vertical_total = 10;

        let registers = Rc::new(RefCell::new(registers));
        let mut crtc = setup_test(registers);

        let results = collect_results(&mut crtc, length);
        let vsync_values: Vec<bool> = results.iter().map(|r| r.vsync).collect();

        let expected: Vec<bool> = (0..length)
            .map(|i| expected_indices.contains(&i))
            .collect();

        assert_eq!(vsync_values, expected);
    }
}

#[test]
fn should_complete_frame_at_correct_boundary() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1;
    registers.crtc_r4_vertical_total = 2;
    registers.crtc_r5_vertical_total_adjust = 0;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 8);
    let addresses: Vec<u16> = results.iter().map(|r| r.snapshot_params.address).collect();
    let start_address = addresses[0];

    let mut reset_iteration: i32 = -1;
    for i in 1..addresses.len() {
        if addresses[i] == start_address && i > 4 {
            reset_iteration = i as i32;
            break;
        }
    }

    assert_eq!(reset_iteration, 6, "Address should reset at iteration 6");
}

#[test]
fn should_reset_scanline_after_field_complete() {
    let registers = Rc::new(RefCell::new(create_default_registers()));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 165);

    let mut field_complete_index: i32 = -1;
    for (i, result) in results.iter().enumerate() {
        if result.field_complete {
            field_complete_index = i as i32;
            break;
        }
    }

    assert_eq!(field_complete_index, 160);

    let scanline_at_field_complete = results[field_complete_index as usize]
        .snapshot_params
        .scanline;
    assert_eq!(scanline_at_field_complete, 160);

    let scanline_after_reset = results[field_complete_index as usize + 1]
        .snapshot_params
        .scanline;
    assert_eq!(scanline_after_reset, 0);
}

#[test]
fn should_respect_vertical_total_adjust_when_completing_frames() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1;
    registers.crtc_r4_vertical_total = 2;
    registers.crtc_r5_vertical_total_adjust = 3;
    registers.crtc_r1_horizontal_displayed = 50;
    registers.crtc_r6_vertical_displayed = 2;
    registers.crtc_r7_vertical_sync_position = 0;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 12);

    let expected = vec![
        ExpectedSnapshot {
            address: Some(0x2000),
            in_scan: Some(true),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2000),
            in_scan: Some(true),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2032),
            in_scan: Some(true),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2032),
            in_scan: Some(true),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2064),
            in_scan: Some(false),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2064),
            in_scan: Some(false),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2096),
            in_scan: Some(false),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2096),
            in_scan: Some(false),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2096),
            in_scan: Some(false),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2000),
            in_scan: Some(true),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            address: Some(0x2000),
            in_scan: Some(true),
            ..ExpectedSnapshot::default()
        },
    ];

    for (actual, expected) in results.iter().zip(expected.iter()) {
        assert_snapshot_matches(&actual.snapshot_params, expected);
    }
}

#[test]
fn should_maintain_consistent_next_scanline_trigger_in_both_frequency_modes() {
    let test_cases = vec![(0x00, 256u16), (0x10, 128u16)];

    for (ula_control, expected_trigger) in test_cases {
        let mut registers = create_default_registers();
        registers.crtc_r0_horizontal_total = 127;
        registers.ula_control = ula_control;

        let registers = Rc::new(RefCell::new(registers));
        let mut crtc = setup_test(registers);

        let results = collect_results(&mut crtc, 20);
        let triggers: Vec<u16> = results.iter().map(|r| r.next_scanline_trigger).collect();
        let expected_triggers = vec![expected_trigger; 20];

        assert_eq!(triggers, expected_triggers);
    }
}

#[test]
fn field_complete_should_only_occur_at_boundaries() {
    let registers = Rc::new(RefCell::new(create_default_registers()));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 165);

    let field_complete_indices: Vec<usize> = results
        .iter()
        .enumerate()
        .filter_map(|(i, r)| if r.field_complete { Some(i) } else { None })
        .collect();

    assert_eq!(field_complete_indices.len(), 1);
    assert_eq!(field_complete_indices[0], 160);
}

#[test]
fn address_should_increment_by_horizontal_displayed_per_char_line() {
    let mut registers = create_default_registers();
    registers.crtc_r1_horizontal_displayed = 40;
    registers.crtc_r9_maximum_raster_address = 1;
    registers.crtc_r4_vertical_total = 10;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 6);
    let addresses: Vec<u16> = results.iter().map(|r| r.snapshot_params.address).collect();

    let expected_addresses: Vec<u16> = vec![0x2000, 0x2000, 0x2028, 0x2028, 0x2050, 0x2050];

    assert_eq!(addresses, expected_addresses);
}

#[test]
fn raster_address_odd_should_equal_even_plus_one_when_interlaced() {
    let mut registers = create_default_registers();
    registers.crtc_r8_interlace_and_skew = 0x03;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 10);

    let expected = vec![
        ExpectedSnapshot {
            raster_address_even: Some(0),
            raster_address_odd: Some(1),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(2),
            raster_address_odd: Some(3),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(4),
            raster_address_odd: Some(5),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(6),
            raster_address_odd: Some(7),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(0),
            raster_address_odd: Some(1),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(2),
            raster_address_odd: Some(3),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(4),
            raster_address_odd: Some(5),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(6),
            raster_address_odd: Some(7),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(0),
            raster_address_odd: Some(1),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(2),
            raster_address_odd: Some(3),
            ..ExpectedSnapshot::default()
        },
    ];

    for (actual, expected) in results.iter().zip(expected.iter()) {
        assert_snapshot_matches(&actual.snapshot_params, expected);
    }
}

#[test]
fn raster_address_odd_should_equal_even_when_not_interlaced() {
    let mut registers = create_default_registers();
    registers.crtc_r8_interlace_and_skew = 0x00;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 10);

    let expected = vec![
        ExpectedSnapshot {
            raster_address_even: Some(0),
            raster_address_odd: Some(0),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(1),
            raster_address_odd: Some(1),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(2),
            raster_address_odd: Some(2),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(3),
            raster_address_odd: Some(3),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(4),
            raster_address_odd: Some(4),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(5),
            raster_address_odd: Some(5),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(6),
            raster_address_odd: Some(6),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(7),
            raster_address_odd: Some(7),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(0),
            raster_address_odd: Some(0),
            ..ExpectedSnapshot::default()
        },
        ExpectedSnapshot {
            raster_address_even: Some(1),
            raster_address_odd: Some(1),
            ..ExpectedSnapshot::default()
        },
    ];

    for (actual, expected) in results.iter().zip(expected.iter()) {
        assert_snapshot_matches(&actual.snapshot_params, expected);
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

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers.clone());

    let first_results = collect_results(&mut crtc, 8);
    let first_addresses: Vec<u16> = first_results
        .iter()
        .map(|r| r.snapshot_params.address)
        .collect();
    assert_eq!(
        first_addresses,
        vec![0x2000, 0x2000, 0x2032, 0x2032, 0x2064, 0x2064, 0x2000, 0x2000]
    );

    registers.borrow_mut().crtc_r12_start_address_h = 0x30;
    let second_results = collect_results(&mut crtc, 6);
    let second_addresses: Vec<u16> = second_results
        .iter()
        .map(|r| r.snapshot_params.address)
        .collect();
    assert_eq!(
        second_addresses,
        vec![0x2032, 0x2032, 0x2064, 0x2064, 0x3000, 0x3000]
    );
}

#[test]
fn should_handle_scan_lines_per_char_zero() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 0;
    registers.crtc_r6_vertical_displayed = 2;
    registers.crtc_r4_vertical_total = 5;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 5);
    let raster_addresses: Vec<u8> = results
        .iter()
        .map(|r| r.snapshot_params.raster_address_even)
        .collect();

    let expected_addresses = vec![0, 0, 0, 0, 0];

    assert_eq!(raster_addresses, expected_addresses);
}

#[test]
fn should_handle_minimal_display_area() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1;
    registers.crtc_r6_vertical_displayed = 1;
    registers.crtc_r4_vertical_total = 5;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 5);
    let in_scan_values: Vec<bool> = results.iter().map(|r| r.snapshot_params.in_scan).collect();

    let expected_pattern = vec![true, true, false, false, false];

    assert_eq!(in_scan_values, expected_pattern);
}

#[test]
fn should_handle_horizontal_total_zero() {
    let mut registers = create_default_registers();
    registers.crtc_r0_horizontal_total = 0;
    registers.ula_control = 0x00;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let result = crtc.advance_scanline();

    assert_eq!(result.next_scanline_trigger, 2);
}

#[test]
fn should_handle_horizontal_total_255() {
    let mut registers = create_default_registers();
    registers.crtc_r0_horizontal_total = 255;
    registers.ula_control = 0x10;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let result = crtc.advance_scanline();

    assert_eq!(result.next_scanline_trigger, 256);
}

#[test]
fn should_maintain_consistent_trigger_values_across_multiple_frames() {
    let mut registers = create_default_registers();
    registers.crtc_r0_horizontal_total = 127;
    registers.ula_control = 0x00;
    registers.crtc_r9_maximum_raster_address = 1;
    registers.crtc_r4_vertical_total = 2;
    registers.crtc_r5_vertical_total_adjust = 0;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let frame_length = 3 * 2;
    let results = collect_results(&mut crtc, frame_length * 2 + 5);
    let triggers: Vec<u16> = results.iter().map(|r| r.next_scanline_trigger).collect();

    let expected_triggers = vec![256u16; 17];

    assert_eq!(triggers, expected_triggers);
}

#[test]
fn should_show_pattern_stability_across_multiple_complete_frames() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1;
    registers.crtc_r4_vertical_total = 2;
    registers.crtc_r5_vertical_total_adjust = 0;
    registers.crtc_r1_horizontal_displayed = 50;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let frame_length = 3 * 2;
    let results = collect_results(&mut crtc, frame_length * 3);
    let scanlines: Vec<u16> = results.iter().map(|r| r.snapshot_params.scanline).collect();

    let expected_scanlines: Vec<u16> = (0..18).map(|i| i as u16).collect();

    assert_eq!(scanlines, expected_scanlines);
}

#[test]
fn should_toggle_interlace_frame_and_double_trigger_on_alternate_frames() {
    let mut registers = create_default_registers();
    registers.crtc_r9_maximum_raster_address = 1;
    registers.crtc_r4_vertical_total = 1;
    registers.crtc_r5_vertical_total_adjust = 0;
    registers.crtc_r0_horizontal_total = 127;
    registers.ula_control = 0x00;
    registers.crtc_r8_interlace_and_skew = 0x01;
    registers.crtc_r7_vertical_sync_position = 0;

    let registers = Rc::new(RefCell::new(registers));
    let mut crtc = setup_test(registers);

    let results = collect_results(&mut crtc, 13);
    let triggers: Vec<u16> = results.iter().map(|r| r.next_scanline_trigger).collect();

    let expected_triggers: Vec<u16> = vec![
        256, 256, 256, 512, 256, 256, 256, 256, 256, 256, 256, 512, 256,
    ];

    assert_eq!(triggers, expected_triggers);
}
