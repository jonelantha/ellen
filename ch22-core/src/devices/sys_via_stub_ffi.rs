use std::{cell::Cell, rc::Rc};

use js_sys::Function;
use wasm_bindgen::JsValue;

use super::sys_via_stub::SysViaStub;
use super::via_port_connections::ViaPortConnections;

pub fn new_sys_via_stub<PortConnections: ViaPortConnections>(
    js_read: Function,
    js_write: Function,
    js_on_vsync_change: Function,
    js_handle_trigger: Function,
    ic32_latch: Rc<Cell<u8>>,
    port_connections: PortConnections,
) -> SysViaStub<PortConnections> {
    let read = Box::new(move |address: u16, cycles: u64| {
        js_read
            .call2(&JsValue::NULL, &address.into(), &cycles.into())
            .expect("js_read error")
            .try_into()
            .expect("js_read error")
    });

    let write = Box::new(move |address: u16, value: u8, ic32: u8, cycles: u64| {
        js_write
            .call4(
                &JsValue::NULL,
                &address.into(),
                &value.into(),
                &ic32.into(),
                &cycles.into(),
            )
            .expect("js_write error")
            .try_into()
            .expect("js_write error")
    });

    let on_vsync_change = Box::new(move |vsync: bool| {
        js_on_vsync_change
            .call1(&JsValue::NULL, &vsync.into())
            .expect("js_on_vsync_change error")
            .try_into()
            .expect("js_on_vsync_change error")
    });

    let handle_trigger = Box::new(move |cycles: u64| {
        js_handle_trigger
            .call1(&JsValue::NULL, &cycles.into())
            .expect("js_handle_trigger error")
            .try_into()
            .expect("js_handle_trigger error")
    });

    SysViaStub::new(
        read,
        write,
        on_vsync_change,
        handle_trigger,
        ic32_latch,
        port_connections,
    )
}
