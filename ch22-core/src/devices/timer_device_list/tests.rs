use std::{cell::RefCell, rc::Rc};

use crate::devices::timer_device_list::*;
use crate::devices::timer_device_mock::*;

#[test]
fn it_needs_sync_for_the_earliest_timing_only() {
    let mut timer_devices = TimerDeviceList::default();
    let (device_1_id, _) = setup_test_device(&mut timer_devices, None);
    let (device_2_id, _) = setup_test_device(&mut timer_devices, None);

    timer_devices.set_device_trigger(device_1_id, Some(4));
    timer_devices.set_device_trigger(device_2_id, Some(8));

    assert!(timer_devices.needs_sync(4));
    assert!(!timer_devices.needs_sync(8));
}

#[test]
fn it_needs_sync_for_earliest_timings_returned_from_device_sync() {
    let mut timer_devices = TimerDeviceList::default();
    let (device_1_id, _) = setup_test_device(&mut timer_devices, Some(8));
    let (device_2_id, _) = setup_test_device(&mut timer_devices, Some(12));

    timer_devices.set_device_trigger(device_1_id, Some(4));
    timer_devices.set_device_trigger(device_2_id, Some(4));

    timer_devices.sync(4);

    assert!(!timer_devices.needs_sync(4));
    assert!(timer_devices.needs_sync(8));
    assert!(!timer_devices.needs_sync(12));
}

#[test]
fn it_only_syncs_devices_with_the_specified_timing() {
    let mut timer_devices = TimerDeviceList::default();
    let (device_1_id, device_1_accesses) = setup_test_device(&mut timer_devices, None);
    let (device_2_id, device_2_accesses) = setup_test_device(&mut timer_devices, None);

    timer_devices.set_device_trigger(device_1_id, Some(4));
    timer_devices.set_device_trigger(device_2_id, Some(8));

    timer_devices.sync(4);

    assert_eq!(device_1_accesses.borrow().syncs, [4]);
    assert_eq!(device_2_accesses.borrow().syncs, []);
}

fn setup_test_device(
    timer_device_list: &mut TimerDeviceList,
    sync_result: Option<u64>,
) -> (TimerDeviceID, Rc<RefCell<TimerDeviceAccessess>>) {
    let test_device = Box::new(TimerDeviceMock::new(sync_result));
    let test_device_accesses = test_device.get_accesses();

    let device_id = timer_device_list.add_device(test_device);

    (device_id, test_device_accesses)
}
