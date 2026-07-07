# BLE Analyzer Pro RS

Rust capture core, CLI, C ABI, and Python wrapper for the WCH / QinHeng BLE
Analyzer Pro.

This is a standalone Rust implementation inspired by the reverse-engineered C
driver in `xecaz/BLE-Analyzer-pro-linux-capture`. The C project remains the
protocol reference. This project is intended to be a cleaner base for long-lived
tooling: a Rust library at the center, a CLI for direct capture, a stable C ABI
for other languages, and a small Python wrapper for automation and analysis.

## Current Status

Working and tested on WSL2 with the WCH BLE Analyzer Pro attached through
`usbipd-win`.

Implemented:

- enumerate the three `VID 1a86 / PID 8009` CH582F MCU devices
- initialize all three MCUs with the known vendor command sequence
- assign BLE advertising channels `37 / 38 / 39` across the three MCUs
- capture BLE advertising packets from bulk endpoint `0x82`
- decode packet metadata: timestamp, channel, RSSI, PDU type, addresses, raw PDU
- write Wireshark-compatible pcap using BLE LL with pseudo-header, linktype 256
- command-line capture tool: `ble-analyzer-pro`
- C ABI shared library: `libble_analyzer_pro.so`
- Python `ctypes` wrapper and live capture example
- CI for formatting, clippy, tests, release build, and Python syntax checks

Experimental / not yet protocol-complete:

- PHY selection is wired into the known config payload, but only the default 1M
  path has been exercised heavily.
- MAC filters, LTK/passkey decryption, and custom 2.4 GHz mode are intentionally
  not advertised as complete. The original C CLI exposes some of these options,
  but the verified command payload does not yet fully document them.
- This captures advertising traffic well. It is not a magic "decrypt every BLE
  connection" tool.

For the strict support table, see [`docs/FEATURE_MATRIX.md`](docs/FEATURE_MATRIX.md).
For module boundaries and design rules, see [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Hardware Model

The analyzer appears as a small WCH USB hub and three independent MCU devices.

```text
VID:PID    Meaning
1a86:8091  WCH CH334 USB hub
1a86:8009  CH582F BLE analyzer MCU, three devices total
```

On Linux/WSL, a healthy attach looks like:

```bash
lsusb | grep 1a86
```

Expected:

```text
Bus 001 Device 005: ID 1a86:8009 QinHeng Electronics ble analyzer
Bus 001 Device 006: ID 1a86:8009 QinHeng Electronics ble analyzer
Bus 001 Device 007: ID 1a86:8009 QinHeng Electronics ble analyzer
```

## Requirements

Ubuntu / WSL:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libusb-1.0-0-dev tshark
```

Rust:

```bash
rustc --version
cargo --version
```

WSL2 USB passthrough on Windows requires `usbipd-win`.

## WSL2 USB Attach

On Windows PowerShell:

```powershell
usbipd list -u
```

If the analyzer is visible but not shared, bind once as Administrator:

```powershell
usbipd bind --busid 3-1 --force
usbipd bind --busid 3-3 --force
usbipd bind --busid 3-4 --force
```

Attach whenever WSL loses the devices:

```powershell
usbipd attach --wsl Ubuntu-26.04 --busid 3-1
usbipd attach --wsl Ubuntu-26.04 --busid 3-3
usbipd attach --wsl Ubuntu-26.04 --busid 3-4
```

The exact bus IDs can change. Trust `usbipd list -u`; look for three
`1a86:8009` devices.

This repo also includes a helper script for the local WSL setup:

```powershell
.\scripts\attach-wsl.ps1
```

## Build

```bash
cd /home/w0x7ce/BLE-Analyzer-pro-rs
cargo build --release
```

Outputs:

```text
target/release/ble-analyzer-pro
target/release/libble_analyzer_pro.so
```

Common local development commands:

```bash
make check      # fmt, clippy, tests, Python syntax check
make release    # release binary and shared library
make list       # list attached analyzer MCU devices
make capture    # short verbose capture to /tmp/ble-analyzer-pro-rs.pcap
make py-live    # Python ctypes live capture example
```

CI runs the same core checks on GitHub Actions.

## CLI Usage

List attached MCU devices:

```bash
./target/release/ble-analyzer-pro --list
```

Capture to pcap for Wireshark:

```bash
mkdir -p ~/captures
./target/release/ble-analyzer-pro -w ~/captures/ble.pcap
```

Capture and print packets live:

```bash
./target/release/ble-analyzer-pro -v -w ~/captures/ble.pcap
```

Short capture:

```bash
./target/release/ble-analyzer-pro -v -w /tmp/ble-rs-test.pcap --duration-ms 3000
```

Stop a long capture with `Ctrl+C`.

Useful options:

```text
--list                 list WCH analyzer MCU devices
-v, --verbose          print decoded packets while capturing
-w, --write FILE       write Wireshark-compatible pcap
-p, --phy N            PHY value 1..4, default 1
-c, --channel N        BLE channel 0..39; 0 means auto 37/38/39 across MCUs
--duration-ms N        stop after N milliseconds
--max-packets N        stop after N packets
--quiet-init           suppress USB init logs
```

## Analyze Captures With tshark

File summary:

```bash
capinfos ~/captures/ble.pcap
```

Show first packets:

```bash
tshark -r ~/captures/ble.pcap -c 20
```

Export useful fields:

```bash
tshark -r ~/captures/ble.pcap \
  -T fields \
  -e frame.time_relative \
  -e btle_rf.channel \
  -e btle_rf.signal_dbm \
  -e btle.advertising_header.pdu_type \
  -e btle.advertising_address \
  -e btcommon.eir_ad.entry.device_name
```

Find named devices:

```bash
tshark -r ~/captures/ble.pcap \
  -Y 'btcommon.eir_ad.entry.device_name' \
  -T fields \
  -e btle.advertising_address \
  -e btcommon.eir_ad.entry.device_name \
| sort | uniq -c | sort -nr
```

Find the busiest advertisers:

```bash
tshark -r ~/captures/ble.pcap \
  -Y 'btle.advertising_address' \
  -T fields -e btle.advertising_address \
| sort | uniq -c | sort -nr | head -20
```

Manufacturer company IDs:

```bash
tshark -r ~/captures/ble.pcap \
  -Y 'btcommon.eir_ad.entry.company_id' \
  -T fields \
  -e btle.advertising_address \
  -e btcommon.eir_ad.entry.company_id \
| sort | uniq -c | sort -nr | head -30
```

Wireshark notes:

- pcap linktype is `Bluetooth Low Energy Link Layer RF`.
- `btle_rf.channel` is the physical RF channel. BLE advertising logical
  channels map as `37 -> 0`, `38 -> 12`, `39 -> 39`.
- The hardware strips the on-air CRC. The pcap writer emits zero CRC bytes and
  marks the packet as checksum-inspected/checksum-valid so Wireshark decodes it.

## Python Usage

Build the release shared library first:

```bash
cargo build --release
```

Run the live example:

```bash
python3 examples/python_live.py --duration-ms 5000 --max-packets 20 -w /tmp/python-capture.pcap
```

Use from your own script:

```python
import sys
sys.path.insert(0, "/home/w0x7ce/BLE-Analyzer-pro-rs/python")

from ble_analyzer_pro import BleAnalyzer

analyzer = BleAnalyzer()
print(analyzer.version)
print(analyzer.list_devices())

def on_packet(pkt):
    print(pkt.type_name, pkt.rssi, pkt.src_addr, pkt.pdu[:8].hex())
    return True

report = analyzer.capture(
    pcap_path="/tmp/ble-python.pcap",
    duration_ms=3000,
    on_packet=on_packet,
)
print(report.total_packets, report.devices_opened, report.elapsed_ms)
```

The wrapper defaults to:

```text
target/release/libble_analyzer_pro.so
```

Override with:

```bash
export BLE_ANALYZER_PRO_LIB=/path/to/libble_analyzer_pro.so
```

## C ABI

Header:

```text
include/ble_analyzer_pro.h
```

Exported functions:

```c
const char *wch_rs_version(void);
const char *wch_rs_last_error(void);
int wch_rs_find_devices(WchRsDeviceInfo *out, size_t capacity);
int wch_rs_capture_blocking(const WchRsCaptureConfig *cfg,
                            WchRsCaptureReport *report_out);
```

`wch_rs_capture_blocking` can write pcap, call a packet callback, or both.
The callback receives metadata plus a pointer to the raw BLE LL PDU bytes. The
PDU pointer is only valid during the callback.

Callback return convention:

```text
0     continue
non-0 stop capture
```

## Architecture

```text
src/protocol.rs   USB command frames, bulk read, WCH frame decoder
src/device.rs     libusb enumeration and interface claim
src/packet.rs     decoded packet model and formatting
src/pcap.rs       Wireshark-compatible pcap writer
src/capture.rs    multi-MCU capture loop and reports
src/ffi.rs        stable C ABI for Python/other languages
src/main.rs       CLI
python/           ctypes wrapper
examples/         Python live capture and pcap analysis helpers
```

The Rust core is deliberately separated from Python. This keeps USB capture and
packet parsing fast, typed, and safe, while allowing Python to do higher-level
automation, dashboards, notebooks, and post-processing.

## Why Rust Here?

Rust is a good fit for this project because the hard part is not GUI code; it is
USB I/O, byte parsing, pcap serialization, long-running capture loops, and a
stable native boundary for other languages. Rust gives:

- memory-safe packet parsing without losing C-like performance
- a single core reusable from CLI, ABI, and Python
- explicit ownership around callback lifetimes and raw packet buffers
- easy packaging as both binary and shared library

Python is still useful above the Rust core. Use it for experiments, filtering,
statistics, dashboards, device naming, CSV export, and integration with other
tools.

## Troubleshooting

No devices from `--list`:

```bash
lsusb | grep 1a86
```

If WSL does not show three `1a86:8009` devices, check Windows:

```powershell
usbipd list -u
```

If state is `Shared (forced)` instead of `Attached`, run:

```powershell
usbipd attach --wsl Ubuntu-26.04 --busid 3-1
usbipd attach --wsl Ubuntu-26.04 --busid 3-3
usbipd attach --wsl Ubuntu-26.04 --busid 3-4
```

Permission denied on Linux:

```bash
sudo cp 99-wch-ble-analyzer.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
sudo usermod -aG plugdev "$USER"
```

Then log out/in or restart WSL.

PCAP opens but looks odd:

- BLE advertising channels use physical RF channel numbers in Wireshark.
- Some extended advertising packets may show strict Wireshark warnings depending
  on payload shape. The raw PDU is still preserved.
- The analyzer provides validated packets without real on-air CRC bytes.

## Roadmap

- CI artifact upload and release packaging
- more unit tests from real captured frame fixtures
- optional JSON/CSV streaming output
- PyO3 package as an alternative to `ctypes`
- protocol research for MAC filters and LTK/passkey handling
- protocol research for custom 2.4 GHz mode
- richer examples for identifying Apple, Xiaomi, ESP32, and GATT-related
  advertising data
