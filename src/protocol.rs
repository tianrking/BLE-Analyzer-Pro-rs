use crate::error::{Error, Result};
use crate::packet::Packet;
use rusb::{Context, DeviceHandle};
use std::time::Duration;

pub const WCH_VID: u16 = 0x1a86;
pub const WCH_PID_BLE_MCU: u16 = 0x8009;
pub const WCH_PID_HUB: u16 = 0x8091;

pub const EP_BULK_IN: u8 = 0x82;
pub const EP_BULK_OUT: u8 = 0x02;
pub const BULK_TRANSFER_SIZE: usize = 0x2800;

const WCH_MAGIC: u8 = 0xaa;
const CMD_IDENTIFY: u8 = 0x84;
const CMD_BLE_CONFIG: u8 = 0x81;
const CMD_SCAN_START: u8 = 0xa1;
const IAP_STR: &[u8; 15] = b"BLEAnalyzer&IAP";

const FRAME_MAGIC: u8 = 0x55;
const FRAME_TYPE_DATA: u8 = 0x10;
const FRAME_TYPE_STATUS: u8 = 0x01;
const BLE_ADV_AA: u32 = 0x8e89bed6;
const MIN_DATA_PAYLOAD: usize = 18;

#[derive(Debug, Clone, Copy)]
pub struct StartConfig {
    pub phy: u8,
    pub ble_channel: u8,
}

impl Default for StartConfig {
    fn default() -> Self {
        Self {
            phy: 1,
            ble_channel: 0,
        }
    }
}

pub fn validate_phy(phy: u8) -> Result<()> {
    if (1..=4).contains(&phy) {
        Ok(())
    } else {
        Err(Error::InvalidConfig(format!("PHY must be 1..4, got {phy}")))
    }
}

pub fn validate_channel(channel: u8) -> Result<()> {
    if channel <= 39 {
        Ok(())
    } else {
        Err(Error::InvalidConfig(format!(
            "channel must be 0..39, got {channel}"
        )))
    }
}

pub fn start_capture(handle: &mut DeviceHandle<Context>, cfg: StartConfig) -> Result<Vec<String>> {
    validate_phy(cfg.phy)?;
    validate_channel(cfg.ble_channel)?;

    let mut log = Vec::new();
    let mut frame = [0u8; 64];
    let mut resp = [0u8; 64];

    frame[0] = WCH_MAGIC;
    frame[1] = CMD_IDENTIFY;
    frame[2] = 0x13;
    frame[3] = 0x00;
    frame[8..23].copy_from_slice(IAP_STR);
    write_bulk(handle, &frame[..23])?;

    match read_bulk(handle, &mut resp, Duration::from_millis(2000)) {
        Ok(got) if got > 0 => log.push(format!(
            "AA84 response[0]=0x{:02x} ({})",
            resp[0],
            if resp[0] != 0 {
                "firmware present"
            } else {
                "no firmware?"
            }
        )),
        Ok(_) | Err(rusb::Error::Timeout) => {}
        Err(err) => return Err(err.into()),
    }

    frame.fill(0);
    frame[0] = WCH_MAGIC;
    frame[1] = CMD_BLE_CONFIG;
    frame[2] = 0x19;
    frame[3] = 0x00;
    frame[4] = 0xff;
    frame[5] = cfg.phy;
    frame[6] = cfg.ble_channel;
    frame[15] = 0xd6;
    frame[16] = 0xbe;
    frame[17] = 0x89;
    frame[18] = 0x8e;
    frame[19] = 0x55;
    frame[20] = 0x55;
    frame[21] = 0x55;
    frame[22] = 0x10;
    write_bulk(handle, &frame[..29])?;

    match read_bulk(handle, &mut resp, Duration::from_millis(100)) {
        Ok(got) if got > 0 => log.push(format!("AA81 triggered {got} byte(s)")),
        Ok(_) | Err(rusb::Error::Timeout) => {}
        Err(err) => return Err(err.into()),
    }

    frame.fill(0);
    frame[0] = WCH_MAGIC;
    frame[1] = CMD_SCAN_START;
    write_bulk(handle, &frame[..4])?;

    match read_bulk(handle, &mut resp, Duration::from_millis(1000)) {
        Ok(got) if got > 0 => log.push(format!(
            "AAA1 response: {got} bytes (magic=0x{:02x} type=0x{:02x})",
            resp[0],
            if got > 1 { resp[1] } else { 0 }
        )),
        Ok(_) | Err(rusb::Error::Timeout) => {}
        Err(err) => return Err(err.into()),
    }

    Ok(log)
}

pub fn stop_capture(handle: &mut DeviceHandle<Context>) -> Result<()> {
    let frame = [WCH_MAGIC, CMD_SCAN_START, 0x00, 0x00];
    match write_bulk(handle, &frame) {
        Ok(_) | Err(rusb::Error::Timeout) | Err(rusb::Error::NoDevice) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

pub fn read_packets(
    handle: &mut DeviceHandle<Context>,
    buf: &mut [u8],
    state: &mut DecodeState,
    timeout: Duration,
) -> Result<Vec<Packet>> {
    match read_bulk(handle, buf, timeout) {
        Ok(transfer_len) => Ok(decode_transfer(&buf[..transfer_len], state)),
        Err(rusb::Error::Timeout) => Ok(Vec::new()),
        Err(err) => Err(err.into()),
    }
}

fn write_bulk(handle: &mut DeviceHandle<Context>, bytes: &[u8]) -> rusb::Result<usize> {
    handle.write_bulk(EP_BULK_OUT, bytes, Duration::from_millis(1000))
}

fn read_bulk(
    handle: &mut DeviceHandle<Context>,
    buf: &mut [u8],
    timeout: Duration,
) -> rusb::Result<usize> {
    handle.read_bulk(EP_BULK_IN, buf, timeout)
}

#[derive(Debug, Default, Clone)]
pub struct DecodeState {
    pub rx_count: u64,
    pub err_count: u64,
    pub ts_prev_us: u32,
    pub ts_hi_us: u64,
    pub pkt_seq: u64,
}

pub fn decode_transfer(buf: &[u8], state: &mut DecodeState) -> Vec<Packet> {
    let mut out = Vec::new();
    let mut offset = 0usize;

    while offset + 4 <= buf.len() {
        if buf[offset] != FRAME_MAGIC {
            offset += 1;
            continue;
        }

        let ftype = buf[offset + 1];
        let plen = u16::from_le_bytes([buf[offset + 2], buf[offset + 3]]) as usize;
        let frame_size = 4 + plen;
        if offset + frame_size > buf.len() {
            break;
        }

        let payload = &buf[offset + 4..offset + frame_size];

        if payload.len() > 5 && payload[5] != 0 {
            state.err_count += 1;
            offset += frame_size;
            continue;
        }

        if ftype == FRAME_TYPE_STATUS {
            state.err_count += 1;
            offset += frame_size;
            continue;
        }

        if ftype != FRAME_TYPE_DATA {
            state.err_count += 1;
            offset += 1;
            continue;
        }

        if payload.len() < MIN_DATA_PAYLOAD {
            state.err_count += 1;
            offset += frame_size;
            continue;
        }

        let channel = payload[4];
        if channel > 39 {
            state.err_count += 1;
            offset += frame_size;
            continue;
        }

        let ts32 = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        if ts32 < state.ts_prev_us {
            state.ts_hi_us += 0x1_0000_0000;
        }
        let prev_ts64 = state.ts_hi_us | state.ts_prev_us as u64;
        let ts64 = state.ts_hi_us | ts32 as u64;
        let interval_us = ts64.saturating_sub(prev_ts64);
        state.ts_prev_us = ts32;

        let rssi = payload[8] as i8;
        let pdu_hdr0 = payload[10];
        let pdu_plen = payload[11] as usize;
        let pkt_type = pdu_hdr0 & 0x0f;
        let direction = payload[5] & 0x01;
        let pdu_start = 10usize;
        let requested = 2 + pdu_plen;
        let available = payload.len().saturating_sub(pdu_start);
        let pdu_len = requested.min(available);
        let pdu = payload[pdu_start..pdu_start + pdu_len].to_vec();

        let mut src_addr = [0u8; 6];
        src_addr.copy_from_slice(&payload[12..18]);

        let mut dst_addr = [0u8; 6];
        if (pkt_type == 0x03 || pkt_type == 0x05) && pdu_plen >= 12 && payload.len() >= 24 {
            dst_addr.copy_from_slice(&payload[18..24]);
        }

        let pkt = Packet {
            rssi,
            pkt_type,
            direction,
            access_addr: BLE_ADV_AA,
            src_addr,
            dst_addr,
            pkt_index: state.pkt_seq,
            timestamp_us: ts64,
            interval_us,
            channel_index: channel,
            pdu,
        };
        state.pkt_seq += 1;
        state.rx_count += 1;
        out.push(pkt);
        offset += frame_size;
    }

    out
}

pub fn ble_channel_to_rf_channel(ch: u8) -> u8 {
    match ch {
        37 => 0,
        38 => 12,
        39 => 39,
        0..=10 => ch + 1,
        _ => ch + 2,
    }
}

pub fn pkt_type_name(pkt_type: u8) -> &'static str {
    match pkt_type {
        0x00 => "ADV_IND",
        0x01 => "ADV_DIRECT_IND",
        0x02 => "ADV_NONCONN_IND",
        0x03 => "SCAN_REQ",
        0x04 => "SCAN_RSP",
        0x05 => "CONNECT_REQ",
        0x06 => "ADV_SCAN_IND",
        0x07 => "AUX_SCAN_REQ",
        0x08 => "AUX_CONNECT_REQ",
        0x09 => "AUX_COMMON",
        0x0a => "AUX_ADV_IND",
        0x0b => "AUX_SCAN_RSP",
        0x0c => "AUX_SYNC_IND",
        0x0d => "AUX_CONNECT_RSP",
        0x0e => "AUX_CHAIN_IND",
        _ => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_ble_logical_channels_to_rf_channels() {
        assert_eq!(ble_channel_to_rf_channel(37), 0);
        assert_eq!(ble_channel_to_rf_channel(38), 12);
        assert_eq!(ble_channel_to_rf_channel(39), 39);
        assert_eq!(ble_channel_to_rf_channel(0), 1);
        assert_eq!(ble_channel_to_rf_channel(10), 11);
        assert_eq!(ble_channel_to_rf_channel(11), 13);
        assert_eq!(ble_channel_to_rf_channel(36), 38);
    }

    #[test]
    fn decodes_one_minimal_advertising_frame() {
        let mut state = DecodeState::default();
        let frame = [
            0x55, 0x10, 0x12, 0x00, // frame header, payload len 18
            0x01, 0x00, 0x00, 0x00, // timestamp
            37,   // channel
            0x00, // flags
            0x00, 0x00, // reserved
            0xd6, // rssi = -42
            0x00, // reserved
            0x42, // ADV_NONCONN_IND, TxAdd set
            0x06, // payload len
            1, 2, 3, 4, 5, 6, // AdvA
        ];

        let packets = decode_transfer(&frame, &mut state);
        assert_eq!(packets.len(), 1);
        let pkt = &packets[0];
        assert_eq!(pkt.rssi, -42);
        assert_eq!(pkt.channel_index, 37);
        assert_eq!(pkt.rf_channel(), 0);
        assert_eq!(pkt.pkt_type, 0x02);
        assert_eq!(pkt.src_addr, [1, 2, 3, 4, 5, 6]);
        assert_eq!(pkt.pdu, vec![0x42, 0x06, 1, 2, 3, 4, 5, 6]);
    }
}
