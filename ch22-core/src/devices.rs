mod device;
mod device_list;
mod js_device;
mod rom_select;
mod static_device;
mod via;

pub use device::Device;
pub use device_list::{DeviceID, DeviceList, DeviceSpeed};
pub use js_device::JsDevice;
pub use rom_select::RomSelect;
pub use static_device::StaticDevice;
#[cfg(test)]
pub use via::ViaStub;
pub use via::{SysViaBus, new_via_stub};

#[cfg(test)]
pub mod device_mock;
