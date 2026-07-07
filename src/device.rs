use crate::error::Result;
use crate::protocol::{DecodeState, WCH_PID_BLE_MCU, WCH_VID};
use rusb::{Context, DeviceHandle, UsbContext};

#[derive(Debug, Clone, Copy)]
pub struct DeviceInfo {
    pub bus: u8,
    pub address: u8,
    pub vendor_id: u16,
    pub product_id: u16,
}

pub(crate) struct McuDevice {
    pub info: DeviceInfo,
    pub handle: DeviceHandle<Context>,
    pub state: DecodeState,
}

impl Drop for McuDevice {
    fn drop(&mut self) {
        let _ = self.handle.release_interface(0);
    }
}

pub fn find_devices() -> Result<Vec<DeviceInfo>> {
    let ctx = Context::new()?;
    find_devices_with_context(&ctx)
}

pub(crate) fn find_devices_with_context(ctx: &Context) -> Result<Vec<DeviceInfo>> {
    let mut out = Vec::new();
    let devices = ctx.devices()?;

    for device in devices.iter() {
        let desc = device.device_descriptor()?;
        if desc.vendor_id() == WCH_VID && desc.product_id() == WCH_PID_BLE_MCU {
            out.push(DeviceInfo {
                bus: device.bus_number(),
                address: device.address(),
                vendor_id: desc.vendor_id(),
                product_id: desc.product_id(),
            });
        }
    }

    out.sort_by_key(|d| (d.bus, d.address));
    Ok(out)
}

pub(crate) fn open_device(ctx: &Context, info: DeviceInfo) -> Result<McuDevice> {
    let devices = ctx.devices()?;

    for device in devices.iter() {
        if device.bus_number() != info.bus || device.address() != info.address {
            continue;
        }

        let handle = device.open()?;
        let _ = handle.set_auto_detach_kernel_driver(true);
        handle.claim_interface(0)?;
        return Ok(McuDevice {
            info,
            handle,
            state: DecodeState::default(),
        });
    }

    Err(rusb::Error::NoDevice.into())
}
