use crate::device::{find_devices_with_context, open_device, McuDevice};
use crate::error::{Error, Result};
use crate::packet::Packet;
use crate::pcap::PcapWriter;
use crate::protocol::{read_packets, start_capture, stop_capture, validate_channel, validate_phy};
use crate::protocol::{StartConfig, BULK_TRANSFER_SIZE};
use rusb::Context;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub phy: u8,
    pub channel: u8,
    pub pcap_path: Option<PathBuf>,
    pub duration: Option<Duration>,
    pub max_packets: Option<u64>,
    pub log_device_init: bool,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            phy: 1,
            channel: 0,
            pcap_path: None,
            duration: None,
            max_packets: None,
            log_device_init: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeviceReport {
    pub bus: u8,
    pub address: u8,
    pub rx_count: u64,
    pub err_count: u64,
}

#[derive(Debug, Clone)]
pub struct CaptureReport {
    pub devices_opened: usize,
    pub total_packets: u64,
    pub device_reports: Vec<DeviceReport>,
    pub elapsed: Duration,
}

pub trait PacketHandler {
    fn on_packet(&mut self, packet: &Packet) -> bool;
}

impl<F> PacketHandler for F
where
    F: FnMut(&Packet) -> bool,
{
    fn on_packet(&mut self, packet: &Packet) -> bool {
        self(packet)
    }
}

pub fn run_capture(
    cfg: &CaptureConfig,
    handler: &mut dyn PacketHandler,
    stop: &AtomicBool,
) -> Result<CaptureReport> {
    validate_phy(cfg.phy)?;
    validate_channel(cfg.channel)?;

    let started = Instant::now();
    let ctx = Context::new()?;
    let infos = find_devices_with_context(&ctx)?;
    if infos.is_empty() {
        return Err(Error::NoDevices);
    }

    let mut devices = Vec::new();
    let mut open_errors = Vec::new();
    for info in infos {
        match open_device(&ctx, info) {
            Ok(dev) => devices.push(dev),
            Err(err) => open_errors.push(format!("bus={} addr={}: {err}", info.bus, info.address)),
        }
    }

    if devices.is_empty() {
        return Err(Error::OpenFailed(open_errors.join("; ")));
    }

    let adv_channels = [37u8, 38, 39];
    let ndev = devices.len();
    for (idx, dev) in devices.iter_mut().enumerate() {
        let ble_channel = if cfg.channel == 0 && ndev > 1 {
            adv_channels.get(idx).copied().unwrap_or(0)
        } else {
            cfg.channel
        };

        let log = start_capture(
            &mut dev.handle,
            StartConfig {
                phy: cfg.phy,
                ble_channel,
            },
        )?;

        if cfg.log_device_init {
            eprintln!(
                "MCU {} bus={} addr={} BLE ch{}",
                idx,
                dev.info.bus,
                dev.info.address,
                if ble_channel == 0 { 37 } else { ble_channel }
            );
            for line in log {
                eprintln!("  {line}");
            }
        }
    }

    let mut pcap = match &cfg.pcap_path {
        Some(path) => Some(PcapWriter::create(path)?),
        None => None,
    };

    let mut bufs = vec![vec![0u8; BULK_TRANSFER_SIZE]; devices.len()];
    let drain_poll = Duration::from_millis(5);
    let idle_wait = Duration::from_millis(100);
    let mut total_packets = 0u64;
    let mut should_stop = false;

    while !stop.load(Ordering::Relaxed) && !should_stop {
        if let Some(duration) = cfg.duration {
            if started.elapsed() >= duration {
                break;
            }
        }
        if let Some(max_packets) = cfg.max_packets {
            if total_packets >= max_packets {
                break;
            }
        }

        let mut any_data = false;

        for (dev, buf) in devices.iter_mut().zip(bufs.iter_mut()) {
            let result = drain_device(
                dev,
                buf,
                drain_poll,
                cfg,
                &mut pcap,
                handler,
                &mut total_packets,
            )?;
            any_data |= result.any_data;
            should_stop |= result.should_stop;
            if should_stop {
                break;
            }
        }

        if !any_data && !should_stop {
            for (dev, buf) in devices.iter_mut().zip(bufs.iter_mut()) {
                let result = poll_device_once(
                    dev,
                    buf,
                    idle_wait,
                    cfg,
                    &mut pcap,
                    handler,
                    &mut total_packets,
                )?;
                should_stop |= result.should_stop;
                if should_stop {
                    break;
                }
            }
        }
    }

    let mut reports = Vec::new();
    for dev in devices.iter_mut() {
        let _ = stop_capture(&mut dev.handle);
        reports.push(DeviceReport {
            bus: dev.info.bus,
            address: dev.info.address,
            rx_count: dev.state.rx_count,
            err_count: dev.state.err_count,
        });
    }

    Ok(CaptureReport {
        devices_opened: reports.len(),
        total_packets,
        device_reports: reports,
        elapsed: started.elapsed(),
    })
}

#[derive(Debug, Clone, Copy)]
struct PacketBatchResult {
    any_data: bool,
    should_stop: bool,
}

fn drain_device(
    dev: &mut McuDevice,
    buf: &mut [u8],
    timeout: Duration,
    cfg: &CaptureConfig,
    pcap: &mut Option<PcapWriter>,
    handler: &mut dyn PacketHandler,
    total_packets: &mut u64,
) -> Result<PacketBatchResult> {
    let mut any_data = false;

    loop {
        let result = poll_device_once(dev, buf, timeout, cfg, pcap, handler, total_packets)?;
        if !result.any_data {
            return Ok(PacketBatchResult {
                any_data,
                should_stop: result.should_stop,
            });
        }
        any_data = true;
        if result.should_stop {
            return Ok(PacketBatchResult {
                any_data,
                should_stop: true,
            });
        }
    }
}

fn poll_device_once(
    dev: &mut McuDevice,
    buf: &mut [u8],
    timeout: Duration,
    cfg: &CaptureConfig,
    pcap: &mut Option<PcapWriter>,
    handler: &mut dyn PacketHandler,
    total_packets: &mut u64,
) -> Result<PacketBatchResult> {
    let packets = read_packets(&mut dev.handle, buf, &mut dev.state, timeout)?;
    let any_data = !packets.is_empty();

    for pkt in packets {
        let keep_going = process_packet(&pkt, pcap, handler)?;
        *total_packets += 1;
        if !keep_going || stop_after_packet(cfg, *total_packets) {
            return Ok(PacketBatchResult {
                any_data,
                should_stop: true,
            });
        }
    }

    Ok(PacketBatchResult {
        any_data,
        should_stop: false,
    })
}

fn process_packet(
    pkt: &Packet,
    pcap: &mut Option<PcapWriter>,
    handler: &mut dyn PacketHandler,
) -> Result<bool> {
    if let Some(writer) = pcap.as_mut() {
        writer.write_packet(pkt)?;
    }
    Ok(handler.on_packet(pkt))
}

fn stop_after_packet(cfg: &CaptureConfig, total_packets: u64) -> bool {
    match cfg.max_packets {
        Some(max_packets) => total_packets >= max_packets,
        None => false,
    }
}
