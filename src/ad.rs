#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvertisingData {
    pub local_name: Option<String>,
    pub short_name: Option<String>,
    pub manufacturer_ids: Vec<u16>,
    pub service_uuids: Vec<String>,
    pub service_data: Vec<String>,
    pub flags: Option<u8>,
}

impl AdvertisingData {
    pub fn display_name(&self) -> Option<&str> {
        self.local_name.as_deref().or(self.short_name.as_deref())
    }

    pub fn has_identity_fields(&self) -> bool {
        self.display_name().is_some()
            || !self.manufacturer_ids.is_empty()
            || !self.service_uuids.is_empty()
            || !self.service_data.is_empty()
    }
}

pub fn parse_advertising_data(pkt_type: u8, pdu: &[u8]) -> AdvertisingData {
    let Some(ad_bytes) = advertising_data_slice(pkt_type, pdu) else {
        return AdvertisingData::default();
    };
    parse_ad_structures(ad_bytes)
}

fn advertising_data_slice(pkt_type: u8, pdu: &[u8]) -> Option<&[u8]> {
    if !matches!(pkt_type, 0x00 | 0x02 | 0x04 | 0x06) || pdu.len() < 8 {
        return None;
    }

    let payload_len = pdu[1] as usize;
    if pdu.len() < 2 + payload_len || payload_len < 6 {
        return None;
    }

    let payload = &pdu[2..2 + payload_len];
    Some(&payload[6..])
}

fn parse_ad_structures(mut bytes: &[u8]) -> AdvertisingData {
    let mut out = AdvertisingData::default();

    while let Some((&field_len, rest)) = bytes.split_first() {
        if field_len == 0 {
            break;
        }

        let field_len = field_len as usize;
        if rest.len() < field_len {
            break;
        }

        let ad_type = rest[0];
        let value = &rest[1..field_len];
        match ad_type {
            0x01 if !value.is_empty() => out.flags = Some(value[0]),
            0x08 => set_name(&mut out.short_name, value),
            0x09 => set_name(&mut out.local_name, value),
            0x02 | 0x03 => collect_uuid16(value, &mut out.service_uuids),
            0x04 | 0x05 => collect_uuid32(value, &mut out.service_uuids),
            0x06 | 0x07 => collect_uuid128(value, &mut out.service_uuids),
            0x16 if value.len() >= 2 => {
                let uuid = u16::from_le_bytes([value[0], value[1]]);
                out.service_data.push(format!("0x{uuid:04X}"));
            }
            0x20 if value.len() >= 4 => {
                let uuid = u32::from_le_bytes([value[0], value[1], value[2], value[3]]);
                out.service_data.push(format!("0x{uuid:08X}"));
            }
            0x21 if value.len() >= 16 => {
                out.service_data.push(format_uuid128(&value[..16]));
            }
            0xff if value.len() >= 2 => {
                out.manufacturer_ids
                    .push(u16::from_le_bytes([value[0], value[1]]));
            }
            _ => {}
        }

        bytes = &rest[field_len..];
    }

    out.manufacturer_ids.sort_unstable();
    out.manufacturer_ids.dedup();
    out.service_uuids.sort();
    out.service_uuids.dedup();
    out.service_data.sort();
    out.service_data.dedup();
    out
}

fn set_name(slot: &mut Option<String>, value: &[u8]) {
    let name = String::from_utf8_lossy(value).trim().to_string();
    if !name.is_empty() {
        *slot = Some(name);
    }
}

fn collect_uuid16(value: &[u8], out: &mut Vec<String>) {
    for chunk in value.chunks_exact(2) {
        let uuid = u16::from_le_bytes([chunk[0], chunk[1]]);
        out.push(format!("0x{uuid:04X}"));
    }
}

fn collect_uuid32(value: &[u8], out: &mut Vec<String>) {
    for chunk in value.chunks_exact(4) {
        let uuid = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        out.push(format!("0x{uuid:08X}"));
    }
}

fn collect_uuid128(value: &[u8], out: &mut Vec<String>) {
    for chunk in value.chunks_exact(16) {
        out.push(format_uuid128(chunk));
    }
}

fn format_uuid128(bytes_le: &[u8]) -> String {
    let mut b = [0u8; 16];
    b.copy_from_slice(&bytes_le[..16]);
    b.reverse();
    format!(
        "{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        b[0],
        b[1],
        b[2],
        b[3],
        b[4],
        b[5],
        b[6],
        b[7],
        b[8],
        b[9],
        b[10],
        b[11],
        b[12],
        b[13],
        b[14],
        b[15]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_name_manufacturer_and_services() {
        let pdu = [
            0x42, 0x17, // header, payload length
            1, 2, 3, 4, 5, 6, // AdvA
            2, 0x01, 0x06, // flags
            5, 0x09, b'T', b'e', b's', b't', // complete name
            3, 0xff, 0x34, 0x12, // manufacturer ID
            3, 0x03, 0x0f, 0x18, // UUID16
        ];

        let ad = parse_advertising_data(0x02, &pdu);
        assert_eq!(ad.display_name(), Some("Test"));
        assert_eq!(ad.flags, Some(0x06));
        assert_eq!(ad.manufacturer_ids, vec![0x1234]);
        assert_eq!(ad.service_uuids, vec!["0x180F"]);
    }

    #[test]
    fn ignores_pdus_without_advertising_data() {
        let pdu = [0x83, 0x0c, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        assert_eq!(
            parse_advertising_data(0x03, &pdu),
            AdvertisingData::default()
        );
    }
}
