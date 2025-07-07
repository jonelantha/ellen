use js_sys::Function;
use wasm_bindgen::JsValue;

use crate::devices_lib::timer_device::TimerDevice;

pub struct JsTimerDevice {
    handle_trigger: Box<dyn Fn(u64) -> u64>,
}

impl JsTimerDevice {
    pub fn new(js_handle_trigger: Function) -> Self {
        let handle_trigger = Box::new(move |cycles: u64| {
            js_handle_trigger
                .call1(&JsValue::NULL, &cycles.into())
                .expect("js_handle_trigger error")
                .try_into()
                .expect("js_handle_trigger error")
        });

        JsTimerDevice { handle_trigger }
    }
}

impl TimerDevice for JsTimerDevice {
    fn sync(&mut self, cycles: u64) -> Option<u64> {
        get_js_trigger_params((self.handle_trigger)(cycles))
    }
}

// trig trig trig trig trig trig flags null

fn get_js_trigger_params(params: u64) -> Option<u64> {
    let flags = (params >> 8) & 0xff;

    if flags & 0x01 != 0 {
        Some(params >> 16)
    } else {
        None
    }
}
