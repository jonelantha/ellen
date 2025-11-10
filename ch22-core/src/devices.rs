mod io_device;
mod io_device_list;
mod js_io_device;
mod js_timer_device;
mod rom_select;
mod static_device;
mod timer_device;
mod timer_device_list;

pub use io_device::IODevice;
pub use io_device_list::{DeviceSpeed, IODeviceID, IODeviceList};
pub use js_io_device::JsIODevice;
pub use js_timer_device::JsTimerDevice;
pub use rom_select::RomSelect;
pub use static_device::StaticDevice;
pub use timer_device::TimerDevice;
pub use timer_device_list::{TimerDeviceID, TimerDeviceList};

#[cfg(test)]
mod io_device_mock;
#[cfg(test)]
pub use io_device_mock::{IODeviceAccesses, IODeviceMock, MemoryAccess};
#[cfg(test)]
mod timer_device_mock;
