use crate::protocol::{ble_channel_to_rf_channel, pkt_type_name};

#[derive(Debug, Clone)]
pub struct Packet {
    pub rssi: i8,
    pub pkt_type: u8,
    pub direction: u8,
    pub access_addr: u32,
    pub src_addr: [u8; 6],
    pub dst_addr: [u8; 6],
    pub pkt_index: u64,
    pub timestamp_us: u64,
    pub interval_us: u64,
    pub channel_index: u8,
    pub pdu: Vec<u8>,
}

impl Packet {
    pub fn rf_channel(&self) -> u8 {
        ble_channel_to_rf_channel(self.channel_index)
    }

    pub fn pkt_type_name(&self) -> &'static str {
        pkt_type_name(self.pkt_type)
    }
}

pub fn format_mac(mac_wire_order: &[u8; 6]) -> String {
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac_wire_order[5],
        mac_wire_order[4],
        mac_wire_order[3],
        mac_wire_order[2],
        mac_wire_order[1],
        mac_wire_order[0]
    )
}

pub fn format_packet(pkt: &Packet) -> String {
    let src = format_mac(&pkt.src_addr);
    let dst = format_mac(&pkt.dst_addr);
    let mut out = format!(
        "[{:12} us] ch{:02} rf{:02}  {:<18}  rssi {:4} dBm  AA {:08X}  {}",
        pkt.timestamp_us,
        pkt.channel_index,
        pkt.rf_channel(),
        pkt.pkt_type_name(),
        pkt.rssi,
        pkt.access_addr,
        src
    );

    if pkt.pkt_type == 0x03 || pkt.pkt_type == 0x05 {
        out.push_str(" -> ");
        out.push_str(&dst);
    }

    out.push_str(&format!("  PDU[{}]:", pkt.pdu.len()));
    for byte in pkt.pdu.iter().take(24) {
        out.push_str(&format!(" {byte:02x}"));
    }
    if pkt.pdu.len() > 24 {
        out.push_str(" ...");
    }
    out
}
