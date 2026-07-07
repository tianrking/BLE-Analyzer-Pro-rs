use crate::capture::{run_capture, CaptureConfig, PacketHandler};
use crate::device::find_devices;
use crate::packet::Packet;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WchRsDeviceInfo {
    pub bus: u8,
    pub address: u8,
    pub vendor_id: u16,
    pub product_id: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WchRsPacket {
    pub rssi: i8,
    pub pkt_type: u8,
    pub direction: u8,
    pub channel_index: u8,
    pub rf_channel: u8,
    pub reserved0: [u8; 3],
    pub access_addr: u32,
    pub src_addr: [u8; 6],
    pub dst_addr: [u8; 6],
    pub pkt_index: u64,
    pub timestamp_us: u64,
    pub interval_us: u64,
    pub pdu_len: usize,
}

pub type WchRsPacketCallback = Option<
    unsafe extern "C" fn(
        packet: *const WchRsPacket,
        pdu: *const u8,
        pdu_len: usize,
        user: *mut c_void,
    ) -> c_int,
>;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WchRsCaptureConfig {
    pub pcap_path: *const c_char,
    pub phy: u8,
    pub channel: u8,
    pub verbose: u8,
    pub reserved0: u8,
    pub duration_ms: u64,
    pub max_packets: u64,
    pub callback: WchRsPacketCallback,
    pub user: *mut c_void,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WchRsCaptureReport {
    pub devices_opened: usize,
    pub total_packets: u64,
    pub elapsed_ms: u64,
}

static LAST_ERROR: OnceLock<Mutex<CString>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn wch_rs_version() -> *const c_char {
    const VERSION_BYTES: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION_BYTES.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn wch_rs_last_error() -> *const c_char {
    let lock = LAST_ERROR.get_or_init(|| Mutex::new(CString::new("").unwrap()));
    match lock.lock() {
        Ok(msg) => msg.as_ptr(),
        Err(_) => ptr::null(),
    }
}

/// Finds attached WCH BLE Analyzer MCU devices.
///
/// # Safety
///
/// If `out` is non-null, it must point to writable memory for at least
/// `capacity` `WchRsDeviceInfo` values. Passing a null pointer is allowed when
/// the caller only wants the returned device count.
#[no_mangle]
pub unsafe extern "C" fn wch_rs_find_devices(out: *mut WchRsDeviceInfo, capacity: usize) -> c_int {
    match find_devices() {
        Ok(devices) => {
            if !out.is_null() {
                let count = devices.len().min(capacity);
                for (idx, dev) in devices.iter().take(count).enumerate() {
                    *out.add(idx) = WchRsDeviceInfo {
                        bus: dev.bus,
                        address: dev.address,
                        vendor_id: dev.vendor_id,
                        product_id: dev.product_id,
                    };
                }
            }
            devices.len() as c_int
        }
        Err(err) => {
            set_last_error(err.to_string());
            -1
        }
    }
}

/// Runs a blocking capture using the supplied C ABI configuration.
///
/// # Safety
///
/// `cfg` must be a valid pointer to a `WchRsCaptureConfig` for the duration of
/// the call. If `cfg.pcap_path` is non-null, it must point to a valid
/// NUL-terminated UTF-8 string. If `report_out` is non-null, it must point to
/// writable memory for one `WchRsCaptureReport`. If a callback is provided, it
/// must not store the packet or PDU pointers after returning.
#[no_mangle]
pub unsafe extern "C" fn wch_rs_capture_blocking(
    cfg: *const WchRsCaptureConfig,
    report_out: *mut WchRsCaptureReport,
) -> c_int {
    if cfg.is_null() {
        set_last_error("capture config pointer is null");
        return -1;
    }

    let ffi_cfg = *cfg;
    let pcap_path = match path_from_ptr(ffi_cfg.pcap_path) {
        Ok(path) => path,
        Err(err) => {
            set_last_error(err);
            return -1;
        }
    };

    let config = CaptureConfig {
        phy: if ffi_cfg.phy == 0 { 1 } else { ffi_cfg.phy },
        channel: ffi_cfg.channel,
        pcap_path,
        duration: if ffi_cfg.duration_ms == 0 {
            None
        } else {
            Some(Duration::from_millis(ffi_cfg.duration_ms))
        },
        max_packets: if ffi_cfg.max_packets == 0 {
            None
        } else {
            Some(ffi_cfg.max_packets)
        },
        filter_addr: None,
        log_device_init: ffi_cfg.verbose != 0,
    };

    let stop = AtomicBool::new(false);
    let mut handler = FfiHandler {
        callback: ffi_cfg.callback,
        user: ffi_cfg.user,
        verbose: ffi_cfg.verbose != 0,
    };

    match run_capture(&config, &mut handler, &stop) {
        Ok(report) => {
            if !report_out.is_null() {
                *report_out = WchRsCaptureReport {
                    devices_opened: report.devices_opened,
                    total_packets: report.total_packets,
                    elapsed_ms: report.elapsed.as_millis() as u64,
                };
            }
            0
        }
        Err(err) => {
            set_last_error(err.to_string());
            -1
        }
    }
}

struct FfiHandler {
    callback: WchRsPacketCallback,
    user: *mut c_void,
    verbose: bool,
}

impl PacketHandler for FfiHandler {
    fn on_packet(&mut self, packet: &Packet) -> bool {
        if self.verbose {
            eprintln!("{}", crate::packet::format_packet(packet));
        }

        let Some(callback) = self.callback else {
            return true;
        };

        let ffi_packet = packet_to_ffi(packet);
        let rc = unsafe {
            callback(
                &ffi_packet as *const WchRsPacket,
                packet.pdu.as_ptr(),
                packet.pdu.len(),
                self.user,
            )
        };
        rc == 0
    }
}

fn packet_to_ffi(packet: &Packet) -> WchRsPacket {
    WchRsPacket {
        rssi: packet.rssi,
        pkt_type: packet.pkt_type,
        direction: packet.direction,
        channel_index: packet.channel_index,
        rf_channel: packet.rf_channel(),
        reserved0: [0; 3],
        access_addr: packet.access_addr,
        src_addr: packet.src_addr,
        dst_addr: packet.dst_addr,
        pkt_index: packet.pkt_index,
        timestamp_us: packet.timestamp_us,
        interval_us: packet.interval_us,
        pdu_len: packet.pdu.len(),
    }
}

unsafe fn path_from_ptr(ptr: *const c_char) -> std::result::Result<Option<PathBuf>, String> {
    if ptr.is_null() {
        return Ok(None);
    }
    let value = CStr::from_ptr(ptr)
        .to_str()
        .map_err(|err| format!("pcap path is not valid UTF-8: {err}"))?;
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(PathBuf::from(value)))
    }
}

fn set_last_error(message: impl AsRef<str>) {
    let sanitized = message.as_ref().replace('\0', " ");
    let lock = LAST_ERROR.get_or_init(|| Mutex::new(CString::new("").unwrap()));
    if let Ok(mut slot) = lock.lock() {
        *slot = CString::new(sanitized).unwrap_or_else(|_| CString::new("unknown error").unwrap());
    }
}
