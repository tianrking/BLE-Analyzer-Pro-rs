use crate::packet::Packet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const PCAP_MAGIC: u32 = 0xa1b2c3d4;
const PCAP_VERSION_MAJOR: u16 = 2;
const PCAP_VERSION_MINOR: u16 = 4;
const PCAP_SNAPLEN: u32 = 65535;
const PCAP_DLT_BLE_LL_WITH_PHDR: u32 = 256;

const FLAG_DEWHITENED: u16 = 0x0001;
const FLAG_SIGNAL_POWER_VALID: u16 = 0x0002;
const FLAG_REFERENCE_AA_VALID: u16 = 0x0010;
const FLAG_CHECKSUM_INSPECTED: u16 = 0x0400;
const FLAG_CHECKSUM_VALID: u16 = 0x0800;

pub struct PcapWriter {
    writer: BufWriter<File>,
}

impl PcapWriter {
    pub fn create(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let mut writer = BufWriter::new(File::create(path)?);
        write_u32(&mut writer, PCAP_MAGIC)?;
        write_u16(&mut writer, PCAP_VERSION_MAJOR)?;
        write_u16(&mut writer, PCAP_VERSION_MINOR)?;
        write_i32(&mut writer, 0)?;
        write_u32(&mut writer, 0)?;
        write_u32(&mut writer, PCAP_SNAPLEN)?;
        write_u32(&mut writer, PCAP_DLT_BLE_LL_WITH_PHDR)?;
        writer.flush()?;
        Ok(Self { writer })
    }

    pub fn write_packet(&mut self, pkt: &Packet) -> std::io::Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let ts_sec = now.as_secs() as u32;
        let ts_usec = now.subsec_micros();

        let flags = FLAG_DEWHITENED
            | FLAG_SIGNAL_POWER_VALID
            | FLAG_REFERENCE_AA_VALID
            | FLAG_CHECKSUM_INSPECTED
            | FLAG_CHECKSUM_VALID;

        let data_len = 10 + 4 + pkt.pdu.len() + 3;

        write_u32(&mut self.writer, ts_sec)?;
        write_u32(&mut self.writer, ts_usec)?;
        write_u32(&mut self.writer, data_len as u32)?;
        write_u32(&mut self.writer, data_len as u32)?;

        self.writer.write_all(&[pkt.rf_channel()])?;
        self.writer.write_all(&[pkt.rssi as u8])?;
        self.writer.write_all(&[0x80])?;
        self.writer.write_all(&[0x00])?;
        write_u32(&mut self.writer, pkt.access_addr)?;
        write_u16(&mut self.writer, flags)?;

        write_u32(&mut self.writer, pkt.access_addr)?;
        self.writer.write_all(&pkt.pdu)?;
        self.writer.write_all(&[0, 0, 0])?;
        self.writer.flush()?;
        Ok(())
    }
}

fn write_u16(w: &mut impl Write, value: u16) -> std::io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

fn write_u32(w: &mut impl Write, value: u32) -> std::io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

fn write_i32(w: &mut impl Write, value: i32) -> std::io::Result<()> {
    w.write_all(&value.to_le_bytes())
}
