use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::cpu::InterruptType;
use crate::devices::{IODeviceAccesses, IODeviceMock, MemoryAccess, TimerDeviceList};
use crate::system::Clock;

use DeviceSpeed::*;
use InterruptType::*;

const TEST_ADDRESS: u16 = 0x1234;
const TEST_VALUE: u8 = 4;

#[test]
fn it_reads_from_a_two_mhz_device_without_adjusting_cycles() {
    let mut io_space = IOSpace::default();
    let mut timer_device_list = TimerDeviceList::default();
    let mut cycles = 1000u64;
    let mut clock = Clock::new(&mut cycles, &mut timer_device_list);

    let test_device_accesses = setup_test_device(&mut io_space, TwoMhz, false, None);

    let read_value = io_space.read(TEST_ADDRESS.into(), &mut clock);

    assert_eq!(cycles, 1000);
    assert_eq!(read_value, TEST_VALUE);
    assert_eq!(
        *test_device_accesses.borrow().memory,
        [MemoryAccess::Read(TEST_ADDRESS, 1000)]
    );
}

#[test]
fn it_writes_to_a_two_mhz_device_without_adjusting_cycles() {
    let mut io_space = IOSpace::default();
    let mut timer_device_list = TimerDeviceList::default();
    let mut cycles = 1000u64;
    let mut clock = Clock::new(&mut cycles, &mut timer_device_list);

    let test_device_accesses = setup_test_device(&mut io_space, TwoMhz, false, None);

    io_space.write(TEST_ADDRESS.into(), 12, &mut clock);

    assert_eq!(cycles, 1000);
    assert_eq!(
        *test_device_accesses.borrow().memory,
        [MemoryAccess::Write(TEST_ADDRESS, 12, 1000)]
    );
}

#[test]
fn it_reads_from_a_one_mhz_device_with_an_additional_cycle_afterwards() {
    let mut io_space = IOSpace::default();
    let mut timer_device_list = TimerDeviceList::default();
    let mut cycles = 1000u64;
    let mut clock = Clock::new(&mut cycles, &mut timer_device_list);

    let test_device_accesses = setup_test_device(&mut io_space, OneMhz, false, None);

    let read_value = io_space.read(TEST_ADDRESS.into(), &mut clock);

    assert_eq!(cycles, 1001);
    assert_eq!(read_value, TEST_VALUE);
    assert_eq!(
        *test_device_accesses.borrow().memory,
        [MemoryAccess::Read(TEST_ADDRESS, 1000)]
    );
}

#[test]
fn it_reads_from_a_one_mhz_device_syncing_to_even_cycles_beforehand() {
    let mut io_space = IOSpace::default();
    let mut timer_device_list = TimerDeviceList::default();
    let mut cycles = 1001u64;
    let mut clock = Clock::new(&mut cycles, &mut timer_device_list);

    let test_device_accesses = setup_test_device(&mut io_space, OneMhz, false, None);

    let read_value = io_space.read(TEST_ADDRESS.into(), &mut clock);

    assert_eq!(cycles, 1003);
    assert_eq!(read_value, TEST_VALUE);
    assert_eq!(
        *test_device_accesses.borrow().memory,
        [MemoryAccess::Read(TEST_ADDRESS, 1002)]
    );
}

#[test]
fn it_writes_to_a_one_mhz_device_with_an_additional_cycle_afterwards() {
    let mut io_space = IOSpace::default();
    let mut timer_device_list = TimerDeviceList::default();
    let mut cycles = 1000u64;
    let mut clock = Clock::new(&mut cycles, &mut timer_device_list);

    let test_device_accesses = setup_test_device(&mut io_space, OneMhz, false, None);

    io_space.write(TEST_ADDRESS.into(), 12, &mut clock);

    assert_eq!(cycles, 1001);
    assert_eq!(
        *test_device_accesses.borrow().memory,
        [MemoryAccess::Write(TEST_ADDRESS, 12, 1000)]
    );
}

#[test]
fn it_writes_to_a_one_mhz_device_syncing_to_even_cycles_beforehand() {
    let mut io_space = IOSpace::default();
    let mut timer_device_list = TimerDeviceList::default();
    let mut cycles = 1001u64;
    let mut clock = Clock::new(&mut cycles, &mut timer_device_list);

    let test_device_accesses = setup_test_device(&mut io_space, OneMhz, false, None);

    io_space.write(TEST_ADDRESS.into(), 12, &mut clock);

    assert_eq!(cycles, 1003);
    assert_eq!(
        *test_device_accesses.borrow().memory,
        [MemoryAccess::Write(TEST_ADDRESS, 12, 1002)]
    );
}

#[test]
fn it_only_reads_the_irq_interrupt_for_irq_devices() {
    let mut io_space = IOSpace::default();
    let mut timer_device_list = TimerDeviceList::default();
    let mut cycles = 1000u64;
    let clock = Clock::new(&mut cycles, &mut timer_device_list);

    let irq_test_device_accesses = setup_test_device(&mut io_space, OneMhz, true, Some(IRQ));
    let nmi_test_device_accesses = setup_test_device(&mut io_space, OneMhz, true, Some(NMI));

    io_space.get_interrupt(IRQ, &clock);

    assert_eq!(*irq_test_device_accesses.borrow().interrupt, [1000]);
    assert_eq!(*nmi_test_device_accesses.borrow().interrupt, []);
}

#[test]
fn it_only_reads_the_nmi_interrupt_for_nmi_devices() {
    let mut io_space = IOSpace::default();
    let mut timer_device_list = TimerDeviceList::default();
    let mut cycles = 1000u64;
    let clock = Clock::new(&mut cycles, &mut timer_device_list);

    let irq_test_device_accesses = setup_test_device(&mut io_space, OneMhz, true, Some(IRQ));
    let nmi_test_device_accesses = setup_test_device(&mut io_space, OneMhz, true, Some(NMI));

    io_space.get_interrupt(NMI, &clock);

    assert_eq!(*irq_test_device_accesses.borrow().interrupt, []);
    assert_eq!(*nmi_test_device_accesses.borrow().interrupt, [1000]);
}

#[test]
fn it_keeps_reading_interrupts_from_devices_until_interrupt_found() {
    let mut io_space = IOSpace::default();
    let mut timer_device_list = TimerDeviceList::default();
    let mut cycles = 1000u64;
    let clock = Clock::new(&mut cycles, &mut timer_device_list);

    let first_test_device_accesses = setup_test_device(&mut io_space, OneMhz, false, Some(NMI));
    let second_test_device_accesses = setup_test_device(&mut io_space, OneMhz, true, Some(NMI));
    let third_test_device_accesses = setup_test_device(&mut io_space, OneMhz, true, Some(NMI));

    io_space.get_interrupt(NMI, &clock);

    assert_eq!(*first_test_device_accesses.borrow().interrupt, [1000]);
    assert_eq!(*second_test_device_accesses.borrow().interrupt, [1000]);
    assert_eq!(*third_test_device_accesses.borrow().interrupt, []);
}

fn setup_test_device(
    io_space: &mut IOSpace,
    speed: DeviceSpeed,
    interrupt_on: bool,
    interrupt_type: Option<InterruptType>,
) -> Rc<RefCell<IODeviceAccesses>> {
    let test_device = Box::new(IODeviceMock::new(
        &[(TEST_ADDRESS, TEST_VALUE)],
        interrupt_on,
    ));
    let test_device_accesses = test_device.get_accesses();

    io_space.add_device(&[TEST_ADDRESS], test_device, interrupt_type, speed);

    test_device_accesses
}
