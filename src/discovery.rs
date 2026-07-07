use crate::ad::{parse_advertising_data, AdvertisingData};
use crate::packet::{format_mac, Packet};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoverySort {
    RssiChange,
    Strongest,
    Packets,
    Name,
    Manufacturer,
    Kind,
}

#[derive(Debug, Default)]
pub struct DiscoveryTable {
    candidates: BTreeMap<String, Candidate>,
}

impl DiscoveryTable {
    pub fn update(&mut self, packet: &Packet) -> &Candidate {
        let address = format_mac(&packet.src_addr);
        let entry = self
            .candidates
            .entry(address.clone())
            .or_insert_with(|| Candidate::new(address));
        entry.update(packet);
        entry
    }

    pub fn get(&self, address: &str) -> Option<&Candidate> {
        self.candidates.get(address)
    }

    pub fn sorted(&self, sort: DiscoverySort) -> Vec<&Candidate> {
        let mut out: Vec<&Candidate> = self.candidates.values().collect();
        out.sort_by(|a, b| compare_candidates(a, b, sort));
        out
    }

    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub address: String,
    pub packets: u64,
    pub first_rssi: i8,
    pub last_rssi: i8,
    pub min_rssi: i8,
    pub max_rssi: i8,
    rssi_sum: i64,
    pub names: BTreeSet<String>,
    pub manufacturer_ids: BTreeSet<u16>,
    pub service_uuids: BTreeSet<String>,
    pub service_data: BTreeSet<String>,
    pub pdu_types: BTreeSet<String>,
    pub channels: BTreeSet<u8>,
}

impl Candidate {
    fn new(address: String) -> Self {
        Self {
            address,
            packets: 0,
            first_rssi: 0,
            last_rssi: 0,
            min_rssi: i8::MAX,
            max_rssi: i8::MIN,
            rssi_sum: 0,
            names: BTreeSet::new(),
            manufacturer_ids: BTreeSet::new(),
            service_uuids: BTreeSet::new(),
            service_data: BTreeSet::new(),
            pdu_types: BTreeSet::new(),
            channels: BTreeSet::new(),
        }
    }

    fn update(&mut self, packet: &Packet) {
        if self.packets == 0 {
            self.first_rssi = packet.rssi;
        }
        self.packets += 1;
        self.last_rssi = packet.rssi;
        self.min_rssi = self.min_rssi.min(packet.rssi);
        self.max_rssi = self.max_rssi.max(packet.rssi);
        self.rssi_sum += packet.rssi as i64;
        self.pdu_types.insert(packet.pkt_type_name().to_string());
        self.channels.insert(packet.channel_index);

        let ad = parse_advertising_data(packet.pkt_type, &packet.pdu);
        self.merge_ad(ad);
    }

    fn merge_ad(&mut self, ad: AdvertisingData) {
        if let Some(name) = ad.display_name() {
            self.names.insert(name.to_string());
        }
        self.manufacturer_ids.extend(ad.manufacturer_ids);
        self.service_uuids.extend(ad.service_uuids);
        self.service_data.extend(ad.service_data);
    }

    pub fn avg_rssi(&self) -> f64 {
        if self.packets == 0 {
            0.0
        } else {
            self.rssi_sum as f64 / self.packets as f64
        }
    }

    pub fn rssi_delta(&self) -> i16 {
        self.max_rssi as i16 - self.min_rssi as i16
    }

    pub fn kind(&self) -> &'static str {
        if !self.names.is_empty() {
            "named"
        } else if !self.manufacturer_ids.is_empty() {
            "manufacturer"
        } else if !self.service_uuids.is_empty() || !self.service_data.is_empty() {
            "service"
        } else {
            "address-only"
        }
    }

    pub fn name_summary(&self) -> String {
        join_limited(self.names.iter().map(String::as_str), 24)
    }

    pub fn manufacturer_summary(&self) -> String {
        join_limited(
            self.manufacturer_ids
                .iter()
                .map(|id| format!("0x{id:04X}"))
                .collect::<Vec<_>>()
                .iter()
                .map(String::as_str),
            18,
        )
    }

    pub fn service_summary(&self) -> String {
        let mut values: Vec<String> = self.service_uuids.iter().cloned().collect();
        values.extend(self.service_data.iter().map(|svc| format!("data:{svc}")));
        join_limited(values.iter().map(String::as_str), 22)
    }

    pub fn type_summary(&self) -> String {
        join_limited(self.pdu_types.iter().map(String::as_str), 20)
    }

    pub fn channel_summary(&self) -> String {
        join_limited(
            self.channels
                .iter()
                .map(|ch| ch.to_string())
                .collect::<Vec<_>>()
                .iter()
                .map(String::as_str),
            12,
        )
    }
}

fn compare_candidates(a: &Candidate, b: &Candidate, sort: DiscoverySort) -> std::cmp::Ordering {
    match sort {
        DiscoverySort::RssiChange => b
            .rssi_delta()
            .cmp(&a.rssi_delta())
            .then_with(|| b.packets.cmp(&a.packets)),
        DiscoverySort::Strongest => b
            .max_rssi
            .cmp(&a.max_rssi)
            .then_with(|| b.avg_rssi().total_cmp(&a.avg_rssi())),
        DiscoverySort::Packets => b.packets.cmp(&a.packets),
        DiscoverySort::Name => a
            .name_summary()
            .cmp(&b.name_summary())
            .then_with(|| b.packets.cmp(&a.packets)),
        DiscoverySort::Manufacturer => a
            .manufacturer_summary()
            .cmp(&b.manufacturer_summary())
            .then_with(|| b.packets.cmp(&a.packets)),
        DiscoverySort::Kind => kind_rank(a.kind())
            .cmp(&kind_rank(b.kind()))
            .then_with(|| b.rssi_delta().cmp(&a.rssi_delta())),
    }
    .then_with(|| a.address.cmp(&b.address))
}

fn kind_rank(kind: &str) -> u8 {
    match kind {
        "named" => 0,
        "manufacturer" => 1,
        "service" => 2,
        _ => 3,
    }
}

fn join_limited<'a>(items: impl Iterator<Item = &'a str>, max: usize) -> String {
    let mut out = String::new();
    for item in items {
        if item.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(',');
        }
        out.push_str(item);
    }
    if out.len() > max {
        out.truncate(max.saturating_sub(1));
        out.push('~');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(address: [u8; 6], rssi: i8, pdu: Vec<u8>) -> Packet {
        Packet {
            rssi,
            pkt_type: pdu[0] & 0x0f,
            direction: 0,
            access_addr: 0x8e89bed6,
            src_addr: address,
            dst_addr: [0; 6],
            pkt_index: 0,
            timestamp_us: 0,
            interval_us: 0,
            channel_index: 37,
            pdu,
        }
    }

    #[test]
    fn aggregates_identity_and_rssi_delta() {
        let pdu = vec![
            0x42, 0x10, 1, 2, 3, 4, 5, 6, 5, 0x09, b'T', b'e', b's', b't', 3, 0xff, 0x34, 0x12,
        ];
        let mut table = DiscoveryTable::default();
        table.update(&packet([1, 2, 3, 4, 5, 6], -80, pdu.clone()));
        table.update(&packet([1, 2, 3, 4, 5, 6], -40, pdu));

        let candidate = table.get("06:05:04:03:02:01").unwrap();
        assert_eq!(candidate.packets, 2);
        assert_eq!(candidate.rssi_delta(), 40);
        assert_eq!(candidate.name_summary(), "Test");
        assert_eq!(candidate.manufacturer_summary(), "0x1234");
        assert_eq!(candidate.kind(), "named");
    }

    #[test]
    fn sorts_kind_by_useful_identity_first() {
        assert!(kind_rank("named") < kind_rank("manufacturer"));
        assert!(kind_rank("manufacturer") < kind_rank("service"));
        assert!(kind_rank("service") < kind_rank("address-only"));
    }
}
