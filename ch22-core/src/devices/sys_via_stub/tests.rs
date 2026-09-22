use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::*;

// IC32 register address (low nibble 0)
const ADDR_IC32: u16 = 0xfe40;
// ORA/sound register address (low nibble 1)
const ADDR_ORA: u16 = 0xfe41;
// ORA mirror address (low nibble 15)
const ADDR_ORA_MIRROR: u16 = 0xfe4f;
// DDRB address (low nibble 2)
const ADDR_DDRB: u16 = 0xfe42;
// an address that phase_2 doesn't treat specially
const ADDR_OTHER: u16 = 0xfe43;

// DDRB value with all bits configured as output, required before an IC32 write
const DDRB_ALL_OUTPUT: u8 = 0x0f;

#[test]
fn it_sets_an_ic32_latch_bit_without_triggering_a_sound_register_write() {
    let (mut stub, ic32_latch, sound_writes) = make_stub(0x00);
    prime_ddrb(&mut stub);

    // bit 3 set (0x08 | 0x03)
    stub.phase_2(Word::from(ADDR_IC32), 0x0b, 100);

    assert_eq!(ic32_latch.get(), 0x08);
    assert_eq!(*sound_writes.borrow(), []);
}

#[test]
fn it_clears_an_ic32_latch_bit_without_triggering_a_sound_register_write() {
    let (mut stub, ic32_latch, sound_writes) = make_stub(0xff);
    prime_ddrb(&mut stub);

    // bit 3 clear
    stub.phase_2(Word::from(ADDR_IC32), 0x03, 100);

    assert_eq!(ic32_latch.get(), 0xf7);
    assert_eq!(*sound_writes.borrow(), []);
}

#[test]
#[should_panic(expected = "DDRB is not set to output for all bits")]
fn it_panics_when_writing_ic32_latch_while_ddrb_is_not_fully_output() {
    let (mut stub, _ic32_latch, _sound_writes) = make_stub(0x00);

    stub.phase_2(Word::from(ADDR_DDRB), 0x0e, 1);
    stub.phase_2(Word::from(ADDR_IC32), 0x08, 2);
}

#[test]
fn it_triggers_a_sound_register_write_on_the_sound_select_bit_falling() {
    let test_cases = [
        // (initial ic32 bit0, written value, expected new latch, expect a trigger)
        (1, 0x00, 0x00, true),  // falling: selects the sound chip
        (0, 0x00, 0x00, false), // already selected: no repeat trigger
        (0, 0x08, 0x01, false), // rising: deselects the sound chip
        (1, 0x08, 0x01, false), // already deselected: stays deselected
    ];

    for (initial_bit0, written_value, expected_latch, expect_trigger) in test_cases {
        let (mut stub, ic32_latch, sound_writes) = make_stub(initial_bit0);
        prime_ddrb(&mut stub);

        // latch the ORA value that a trigger should report
        stub.phase_2(Word::from(ADDR_ORA), 0xab, 50);
        sound_writes.borrow_mut().clear();

        stub.phase_2(Word::from(ADDR_IC32), written_value, 200);

        assert_eq!(
            ic32_latch.get(),
            expected_latch,
            "latch mismatch for initial_bit0={initial_bit0:#x}, written_value={written_value:#x}"
        );

        let expected_writes = if expect_trigger {
            vec![(200, 0xab)]
        } else {
            vec![]
        };
        assert_eq!(
            *sound_writes.borrow(),
            expected_writes,
            "sound write mismatch for initial_bit0={initial_bit0:#x}, written_value={written_value:#x}"
        );
    }
}

#[test]
fn it_forwards_an_ora_write_to_the_sound_register_when_the_sound_chip_is_selected() {
    let (mut stub, _ic32_latch, sound_writes) = make_stub(0x00);

    stub.phase_2(Word::from(ADDR_ORA), 0x9c, 500);

    assert_eq!(*sound_writes.borrow(), [(500, 0x9c)]);
}

#[test]
fn it_does_not_forward_an_ora_write_to_the_sound_register_when_the_sound_chip_is_not_selected() {
    let (mut stub, _ic32_latch, sound_writes) = make_stub(0x01);

    stub.phase_2(Word::from(ADDR_ORA), 0x9c, 600);

    assert_eq!(*sound_writes.borrow(), []);
}

#[test]
fn it_treats_the_mirrored_ora_address_the_same_as_the_primary_one() {
    let (mut stub, _ic32_latch, sound_writes) = make_stub(0x00);

    stub.phase_2(Word::from(ADDR_ORA_MIRROR), 0x77, 700);

    assert_eq!(*sound_writes.borrow(), [(700, 0x77)]);
}

#[test]
fn it_never_triggers_a_sound_register_write_for_ddrb_writes() {
    let (mut stub, _ic32_latch, sound_writes) = make_stub(0x00);

    stub.phase_2(Word::from(ADDR_DDRB), 0xaa, 800);

    assert_eq!(*sound_writes.borrow(), []);
}

#[test]
fn it_ignores_addresses_that_are_not_ic32_ora_or_ddrb() {
    let (mut stub, ic32_latch, sound_writes) = make_stub(0x00);
    let latch_before = ic32_latch.get();

    stub.phase_2(Word::from(ADDR_OTHER), 0x55, 900);

    assert_eq!(ic32_latch.get(), latch_before);
    assert_eq!(*sound_writes.borrow(), []);
}

fn prime_ddrb(stub: &mut SysViaStub<impl Fn(u64, u8)>) {
    stub.phase_2(Word::from(ADDR_DDRB), DDRB_ALL_OUTPUT, 0);
}

#[allow(clippy::type_complexity)]
fn make_stub(
    initial_ic32_latch: u8,
) -> (
    SysViaStub<impl Fn(u64, u8)>,
    Rc<Cell<u8>>,
    Rc<RefCell<Vec<(u64, u8)>>>,
) {
    let ic32_latch = Rc::new(Cell::new(initial_ic32_latch));
    let sound_writes = Rc::new(RefCell::new(Vec::new()));
    let recorder = sound_writes.clone();

    let stub = SysViaStub::new(
        Box::new(|_, _| 0),
        Box::new(|_, _, _, _| 0),
        Box::new(|_| 0),
        Box::new(|_| 0),
        ic32_latch.clone(),
        move |cycles, value| recorder.borrow_mut().push((cycles, value)),
    );

    (stub, ic32_latch, sound_writes)
}
