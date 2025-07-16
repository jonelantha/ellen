use js_sys::Function;
use wasm_bindgen::JsValue;

use super::timer_device::*;

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
    let [_, _, _, _, _, _, flags, _] = params.to_be_bytes();

    if flags & JS_TIMER_FLAG_HAS_TRIGGER != 0 {
        Some(params >> 16)
    } else {
        None
    }
}

const JS_TIMER_FLAG_HAS_TRIGGER: u8 = 0x01;
