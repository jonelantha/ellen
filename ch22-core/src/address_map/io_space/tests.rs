use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::clock::*;
use crate::devices::io_device_mock::*;
use crate::interrupt_type::*;

const TEST_ADDRESS: u16 = 0x1234;
const TEST_VALUE: u8 = 4;

#[test]
fn it_reads_from_a_two_mhz_device_without_adjusting_cycles() {
    let mut io_space = IOSpace::default();
    let mut clock = Clock::new(1000);

    let test_device_accesses = setup_test_device(&mut io_space, DeviceSpeed::TwoMhz, None);

    let read_value = io_space.read(TEST_ADDRESS.into(), &mut clock);

    assert_eq!(clock.get_cycles(), 1000);
    assert_eq!(read_value, TEST_VALUE);
    assert_eq!(
        *test_device_accesses.borrow(),
        [Access::Read(TEST_ADDRESS, 1000)]
    );
}

#[test]
fn it_writes_to_a_two_mhz_device_without_adjusting_cycles() {
    let mut io_space = IOSpace::default();
    let mut clock = Clock::new(1000);

    let test_device_accesses = setup_test_device(&mut io_space, DeviceSpeed::TwoMhz, None);

    io_space.write(TEST_ADDRESS.into(), 12, &mut clock);

    assert_eq!(clock.get_cycles(), 1000);
    assert_eq!(
        *test_device_accesses.borrow(),
        [Access::Write(TEST_ADDRESS, 12, 1000)]
    );
}

#[test]
fn it_reads_from_a_one_mhz_device_with_an_additional_cycle_afterwards() {
    let mut io_space = IOSpace::default();
    let mut clock = Clock::new(1000);

    let test_device_accesses = setup_test_device(&mut io_space, DeviceSpeed::OneMhz, None);

    let read_value = io_space.read(TEST_ADDRESS.into(), &mut clock);

    assert_eq!(clock.get_cycles(), 1001);
    assert_eq!(read_value, TEST_VALUE);
    assert_eq!(
        *test_device_accesses.borrow(),
        [Access::Read(TEST_ADDRESS, 1000)]
    );
}

#[test]
fn it_reads_from_a_one_mhz_device_syncing_to_even_cycles_beforehand() {
    let mut io_space = IOSpace::default();
    let mut clock = Clock::new(1001);

    let test_device_accesses = setup_test_device(&mut io_space, DeviceSpeed::OneMhz, None);

    let read_value = io_space.read(TEST_ADDRESS.into(), &mut clock);

    assert_eq!(clock.get_cycles(), 1003);
    assert_eq!(read_value, TEST_VALUE);
    assert_eq!(
        *test_device_accesses.borrow(),
        [Access::Read(TEST_ADDRESS, 1002)]
    );
}

#[test]
fn it_writes_to_a_one_mhz_device_with_an_additional_cycle_afterwards() {
    let mut io_space = IOSpace::default();
    let mut clock = Clock::new(1000);

    let test_device_accesses = setup_test_device(&mut io_space, DeviceSpeed::OneMhz, None);

    io_space.write(TEST_ADDRESS.into(), 12, &mut clock);

    assert_eq!(clock.get_cycles(), 1001);
    assert_eq!(
        *test_device_accesses.borrow(),
        [Access::Write(TEST_ADDRESS, 12, 1000)]
    );
}

#[test]
fn it_writes_to_a_one_mhz_device_syncing_to_even_cycles_beforehand() {
    let mut io_space = IOSpace::default();
    let mut clock = Clock::new(1001);

    let test_device_accesses = setup_test_device(&mut io_space, DeviceSpeed::OneMhz, None);

    io_space.write(TEST_ADDRESS.into(), 12, &mut clock);

    assert_eq!(clock.get_cycles(), 1003);
    assert_eq!(
        *test_device_accesses.borrow(),
        [Access::Write(TEST_ADDRESS, 12, 1002)]
    );
}

fn setup_test_device(
    io_space: &mut IOSpace,
    speed: DeviceSpeed,
    interrupt_type: Option<InterruptType>,
) -> Rc<RefCell<Vec<Access>>> {
    let test_device = Box::new(IODeviceMock::new(&[(TEST_ADDRESS, TEST_VALUE)]));
    let test_device_accesses = test_device.get_accesses();

    io_space.add_device(&[TEST_ADDRESS], test_device, interrupt_type, speed);

    test_device_accesses
}
