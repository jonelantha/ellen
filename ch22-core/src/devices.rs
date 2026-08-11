mod device;
mod device_list;
mod js_device;
mod rom_select;
mod static_device;

pub use device::Device;
pub use device_list::{DeviceSpeed, DeviceID, DeviceList};
pub use js_device::JsDevice;
pub use rom_select::RomSelect;
pub use static_device::StaticDevice;

#[cfg(test)]
pub mod device_mock;
