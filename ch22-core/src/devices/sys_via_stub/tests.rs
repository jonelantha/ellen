use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::devices::SysViaBus;

const ADDR_ORB: u16 = 0x0000;
const ADDR_ORA: u16 = 0x0001;
const ADDR_DDRB: u16 = 0x0002;
const ADDR_DDRA: u16 = 0x0003;
const ADDR_ORA_NO_HANDSHAKE: u16 = 0x000f;
const ADDR_OTHER: u16 = 0x0005;

const DDRA_ALL_OUTPUT: u8 = 0xff;
const DDRB_ALL_OUTPUT: u8 = 0x0f;

#[test]
fn it_sets_an_ic32_latch_bit_without_triggering_a_sound_register_write() {
    let mut harness = Harness::new()
        .ddra_ddrb_output()
        .latch_and_orb(0x00, 0x00)
        .clear_sound_writes();

    // ORB write -> ic32 bit 3 set (0x08 | 0x03)
    harness.stub.phase_2(ADDR_ORB.into(), 0x0b, 100);

    assert_eq!(harness.ic32_latch(), 0x08);
    assert_eq!(harness.get_sound_writes(), []);
}

#[test]
fn it_clears_an_ic32_latch_bit_without_triggering_a_sound_register_write() {
    let mut harness = Harness::new()
        .ddra_ddrb_output()
        .latch_and_orb(0xff, 0x08)
        .clear_sound_writes();

    // ORB write -> ic32 bit 3 clear
    harness.stub.phase_2(ADDR_ORB.into(), 0x03, 100);

    assert_eq!(harness.ic32_latch(), 0xf7);
    assert_eq!(harness.get_sound_writes(), []);
}

#[test]
fn it_reads_an_undriven_ic32_address_pin_as_high() {
    // "address pin": PB0-2 select which IC32 latch bit is written
    // PB0 input -> ic32 bit 1 clear (pins 0x01, already clear)
    let mut harness = Harness::new().ddrb(0x0e).clear_sound_writes();

    // ORB write -> ic32 bit 1 set (pins 0x09), not bit 0
    harness.stub.phase_2(ADDR_ORB.into(), 0x08, 100);

    assert_eq!(harness.ic32_latch(), 0x02);
}

#[test]
fn it_reads_an_undriven_ic32_data_pin_as_high() {
    // "data pin": PB3 is the value (set or clear) written to the selected IC32 latch bit
    // PB3 input -> ic32 bit 0 set (pins 0x08, already set)
    let mut harness = Harness::new()
        .ddra_ddrb_output()
        .latch_and_orb(0x01, 0x08)
        .ddrb(0x07);

    // ORB write -> ic32 bit 2 set (pins 0x0a), not cleared
    harness.stub.phase_2(ADDR_ORB.into(), 0x02, 100);

    assert_eq!(harness.ic32_latch(), 0x05);
}

#[test]
fn it_exposes_ic32_bits_4_and_5_as_the_video_base_bits() {
    let test_cases = [
        // (ic32, expected bits: bit 0 from ic32 bit 4, bit 1 from ic32 bit 5)
        (0x00, 0b00),
        (0x10, 0b01),
        (0x20, 0b10),
        (0x30, 0b11),
        (0xcf, 0b00), // other bits set, bits 4 and 5 clear
        (0xff, 0b11),
        (0xdf, 0b01), // bit 5 clear, bit 4 set
        (0xef, 0b10), // bit 5 set, bit 4 clear
    ];

    for (ic32, expected) in test_cases {
        let harness = Harness::new().ddra_ddrb_output().latch_and_orb(ic32, 0x08);

        assert_eq!(
            harness.bus.borrow().video_base_bits(),
            expected,
            "ic32={ic32:#04x}"
        );
    }
}

#[test]
fn it_records_a_sound_register_write_of_ff_on_init_cycle() {
    let mut harness = Harness::new().clear_sound_writes();

    harness.stub.on_init_cycle(100);

    // ports undriven: IC32 selects the sound chip (output 0 low), data lines float high
    assert_eq!(harness.get_sound_writes(), [(100, 0xff)]);
    assert_eq!(harness.ic32_latch(), 0x80);
}

#[test]
fn it_updates_the_ic32_latch_when_a_ddrb_write_changes_the_pin_levels() {
    let mut harness = Harness::new().ddra_ddrb_output();

    // DDRB write -> PB0-3 all inputs -> ic32 bit 7 set (pins 0x0f)
    harness.stub.phase_2(ADDR_DDRB.into(), 0x00, 100);

    assert_eq!(harness.ic32_latch(), 0x80);
}

#[test]
fn it_triggers_a_sound_register_write_on_the_sound_select_bit_falling() {
    let test_cases = [
        // (initial_ic32, new_orb, expected_ic32, expected_writes)
        (1, 0x00, 0x00, vec![(200, 0xab)]), // falling: selects the sound chip
        (0, 0x00, 0x00, vec![]),            // already selected: no repeat trigger
        (0, 0x08, 0x01, vec![]),            // rising: deselects the sound chip
        (1, 0x08, 0x01, vec![]),            // already deselected: stays deselected
    ];

    for (initial_ic32, new_orb, expected_ic32, expected_writes) in test_cases {
        let mut harness = Harness::new()
            .ddra_ddrb_output()
            .ora(0xab)
            .latch_and_orb(initial_ic32, 0x03)
            .clear_sound_writes();

        harness.stub.phase_2(ADDR_ORB.into(), new_orb, 200);

        assert_eq!(
            harness.ic32_latch(),
            expected_ic32,
            "latch mismatch for initial_ic32={initial_ic32:#x}, new_orb={new_orb:#x}"
        );

        assert_eq!(
            harness.get_sound_writes(),
            expected_writes,
            "sound write mismatch for initial_ic32={initial_ic32:#x}, new_orb={new_orb:#x}"
        );
    }
}

#[test]
fn it_forwards_an_ora_write_to_the_sound_register_when_the_sound_chip_is_selected() {
    let mut harness = Harness::new().ddra_ddrb_output().clear_sound_writes();

    harness.stub.phase_2(ADDR_ORA.into(), 0x9c, 500);

    assert_eq!(harness.get_sound_writes(), [(500, 0x9c)]);
}

#[test]
fn it_does_not_forward_an_ora_write_that_leaves_the_sound_chip_input_unchanged() {
    let mut harness = Harness::new().ddra_ddrb_output().clear_sound_writes();

    harness.stub.phase_2(ADDR_ORA.into(), 0x9c, 500);
    harness.stub.phase_2(ADDR_ORA.into(), 0x9c, 510); // same data, chip still selected
    harness.stub.phase_2(ADDR_ORA.into(), 0x5a, 520);

    assert_eq!(harness.get_sound_writes(), [(500, 0x9c), (520, 0x5a)]);
}

#[test]
fn it_does_not_forward_an_ora_write_to_the_sound_register_when_the_sound_chip_is_not_selected() {
    let mut harness = Harness::new()
        .ddra_ddrb_output()
        .latch_and_orb(0x01, 0x08)
        .clear_sound_writes();

    harness.stub.phase_2(ADDR_ORA.into(), 0x9c, 600);

    assert_eq!(harness.get_sound_writes(), []);
}

#[test]
fn it_treats_the_no_handshake_ora_address_the_same_as_the_primary_one() {
    let mut harness = Harness::new().ddra_ddrb_output().clear_sound_writes();

    harness
        .stub
        .phase_2(ADDR_ORA_NO_HANDSHAKE.into(), 0x77, 700);

    assert_eq!(harness.get_sound_writes(), [(700, 0x77)]);
}

#[test]
fn it_never_triggers_a_sound_register_write_for_ddrb_writes() {
    let mut harness = Harness::new().ddra_ddrb_output().clear_sound_writes();

    harness.stub.phase_2(ADDR_DDRB.into(), 0xaa, 800);

    assert_eq!(harness.get_sound_writes(), []);
}

#[test]
fn it_ignores_addresses_that_are_not_ic32_ora_or_ddrb() {
    let mut harness = Harness::new()
        .ddra_ddrb_output()
        .latch_and_orb(0xff, 0x08)
        .clear_sound_writes();

    harness.stub.phase_2(ADDR_OTHER.into(), 0x55, 900);

    assert_eq!(harness.ic32_latch(), 0xff);
    assert_eq!(harness.get_sound_writes(), []);
}

struct Harness {
    stub: SysViaStub<Rc<RefCell<SysViaBus>>>,
    bus: Rc<RefCell<SysViaBus>>,
}

impl Harness {
    fn new() -> Self {
        let bus = Rc::new(RefCell::new(SysViaBus::default()));

        let stub = SysViaStub::new(
            Box::new(|_, _| 0),
            Box::new(|_, _, _, _| 0),
            Box::new(|_| 0),
            Box::new(|_| 0),
            bus.clone(),
        );

        Harness { stub, bus }
    }

    fn ic32_latch(&self) -> u8 {
        self.bus.borrow().ic32()
    }

    fn ddrb(mut self, value: u8) -> Self {
        self.stub.phase_2(ADDR_DDRB.into(), value, 0);

        self
    }

    fn ddra_ddrb_output(mut self) -> Self {
        self.stub.phase_2(ADDR_DDRA.into(), DDRA_ALL_OUTPUT, 0);
        self.stub.phase_2(ADDR_DDRB.into(), DDRB_ALL_OUTPUT, 0);

        self
    }

    fn ora(mut self, value: u8) -> Self {
        self.stub.phase_2(ADDR_ORA.into(), value, 0);

        self
    }

    fn latch_and_orb(mut self, latch: u8, orb: u8) -> Self {
        for bit in 0..8 {
            let level = if latch & (1 << bit) != 0 { 0x08 } else { 0x00 };
            self.stub.phase_2(ADDR_ORB.into(), level | bit, 0);
        }
        self.stub.phase_2(ADDR_ORB.into(), orb, 0);

        self
    }

    fn get_sound_writes(&self) -> Vec<(u64, u8)> {
        let bus = self.bus.borrow();
        let writes = bus.sound().register_writes();
        let base_cycle_count = writes.base_cycle_count;

        writes.entries[..writes.num_entries]
            .iter()
            .map(|entry| (base_cycle_count + u64::from(entry.cycle_offset), entry.data))
            .collect()
    }

    fn clear_sound_writes(self) -> Self {
        self.bus.borrow_mut().sound_mut().start_field(0);

        self
    }
}
