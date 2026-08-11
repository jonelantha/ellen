use super::Video;
use crate::video::VideoRegisters;

#[test]
fn process_scanline_accumulates_next_scanline_trigger() {
    let registers = VideoRegisters {
        crtc_r0_horizontal_total: 127,
        crtc_r1_horizontal_displayed: 80,
        crtc_r4_vertical_total: 2,
        crtc_r5_vertical_total_adjust: 0,
        crtc_r9_maximum_raster_address: 1,
        ula_control: 0x00,
        ..Default::default()
    };

    let mut video = Video::default();
    *video.registers.borrow_mut() = registers;
    video.crtc.init(&registers);
    video.field_counter = 0;
    video.vsync = false;
    video.next_scanline_trigger = 0;

    let before = video.get_next_scanline_trigger();
    let expected_increment = video.crtc.get_next_scanline_cycles(&registers);

    video.process_scanline(0, |_| &[], |_| {});

    assert!(
        expected_increment > 0,
        "expected_increment should be greater than 0"
    );

    assert_eq!(
        video.get_next_scanline_trigger() - before,
        expected_increment,
        "next_scanline_trigger should increase by the current scanline cost"
    );
}

#[test]
fn process_scanline_increments_field_counter_at_new_field_boundary() {
    let registers = VideoRegisters {
        crtc_r4_vertical_total: 24,
        crtc_r5_vertical_total_adjust: 0,
        crtc_r7_vertical_sync_position: 20,
        crtc_r9_maximum_raster_address: 7,
        ..Default::default()
    };

    let mut video = Video::default();
    *video.registers.borrow_mut() = registers;
    video.crtc.init(&registers);
    video.field_counter = 0;
    video.vsync = false;
    video.next_scanline_trigger = 0;

    // Prove the field buffer is reset on a new field boundary.
    video.field_data.get_line_raw_data_mut(0)[0] = 0xff;

    video.process_scanline(0, |_| &[], |_| {});

    assert_eq!(
        video.field_counter, 1,
        "new field should increment field counter at scanline 0"
    );
    assert_eq!(
        video.field_data.get_line_raw_data(0)[0],
        0,
        "new field should clear the underlying field data"
    );
}

#[test]
fn process_scanline_invokes_vsync_callback_on_state_changes() {
    let registers = VideoRegisters {
        crtc_r3_sync_width: 0x20,
        crtc_r4_vertical_total: 10,
        crtc_r7_vertical_sync_position: 2,
        crtc_r9_maximum_raster_address: 1,
        ..Default::default()
    };

    let mut video = Video::default();
    *video.registers.borrow_mut() = registers;
    video.crtc.init(&registers);
    video.field_counter = 0;
    video.vsync = false;
    video.next_scanline_trigger = 0;

    let mut callback_events = Vec::new();
    for iteration in 0..12 {
        video.process_scanline(0, |_| &[], |value| callback_events.push((iteration, value)));
    }

    assert_eq!(
        callback_events.len(),
        2,
        "vsync callback should fire exactly twice in this window"
    );
    assert_eq!(
        callback_events[0],
        (4, true),
        "vsync callback should fire at the start of the pulse"
    );
    assert_eq!(
        callback_events[1],
        (6, false),
        "vsync callback should fire when the pulse ends"
    );
}

#[test]
fn process_scanline_skips_snapshot_when_scanline_is_not_displayed() {
    let registers = VideoRegisters {
        crtc_r1_horizontal_displayed: 0x50,
        crtc_r4_vertical_total: 10,
        crtc_r6_vertical_displayed: 0,
        crtc_r9_maximum_raster_address: 1,
        ..Default::default()
    };

    let mut video = Video::default();
    *video.registers.borrow_mut() = registers;
    video.crtc.init(&registers);
    video.field_counter = 0;
    video.vsync = false;
    video.next_scanline_trigger = 0;

    video.process_scanline(0, |_| &[], |_| {});

    let line_flags = video.field_data.get_line_raw_data(0)[0];
    assert_eq!(
        line_flags, 0,
        "when the scanline is not displayed, the field line flags must stay zero"
    );
}

#[test]
fn is_field_complete_matches_beam_reset_boundary() {
    let registers = VideoRegisters {
        crtc_r4_vertical_total: 24,
        crtc_r5_vertical_total_adjust: 0,
        crtc_r7_vertical_sync_position: 20,
        crtc_r9_maximum_raster_address: 7,
        ..Default::default()
    };

    let mut video = Video::default();
    *video.registers.borrow_mut() = registers;
    video.crtc.init(&registers);
    video.field_counter = 0;
    video.vsync = false;
    video.next_scanline_trigger = 0;

    for iteration in 0..165 {
        let expected = iteration == 0 || iteration == 161;
        let actual = video.is_field_complete();

        assert_eq!(
            actual, expected,
            "field completion mismatch at iteration {iteration}"
        );

        video.process_scanline(0, |_| &[], |_| {});
    }
}
