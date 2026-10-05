//! Runs fields through a real `Core` with the system VIA stub attached (the
//! JS callbacks are stubbed out), observing what JS would read from the sound
//! register writes buffer.

use super::*;
use crate::cpu::InterruptType;
use crate::devices::ViaStub;

#[test]
fn the_first_field_records_the_sound_write_made_at_cycle_0() {
    let mut core = core_with_sys_via();

    core.run_one_field();

    let bus = core.sys_via_bus.borrow();
    let writes = bus.sound().register_writes();
    let base_cycle_count = writes.base_cycle_count;

    assert_eq!(base_cycle_count, 0);
    assert_eq!(writes.cycles_and_data(), [(0, 0xff)]);
}

#[test]
fn later_fields_start_a_new_buffer_without_the_cycle_0_write() {
    let mut core = core_with_sys_via();

    let first_field_end = core.run_one_field();

    core.run_one_field();

    let bus = core.sys_via_bus.borrow();
    let writes = bus.sound().register_writes();
    let base_cycle_count = writes.base_cycle_count;

    assert_eq!(base_cycle_count, first_field_end);
    assert_eq!(writes.cycles_and_data(), []);
}

fn core_with_sys_via() -> Core {
    let mut core = Core::default();
    core.setup();

    let sys_via = ViaStub::new(
        Box::new(|_, _| 0),
        Box::new(|_, _, _, _| 0),
        Box::new(|_| 0),
        Box::new(|_| 0),
        core.get_sys_via_bus(),
    );

    let addresses: Vec<u16> = (0xfe40..=0xfe5f).collect();

    core.io_space.add_device(
        &addresses,
        Box::new(sys_via),
        Some(InterruptType::IRQ),
        DeviceSpeed::OneMhz,
    );

    core
}
