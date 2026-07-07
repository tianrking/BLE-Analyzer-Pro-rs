<div align="center">

# BLE Analyzer Pro RS

**A Rust-native capture stack for the WCH / QinHeng BLE Analyzer Pro.**

[简体中文](README.zh-CN.md) | English

[![CI](https://github.com/tianrking/BLE-Analyzer-Pro-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/tianrking/BLE-Analyzer-Pro-rs/actions/workflows/ci.yml)
[![Release Binaries](https://github.com/tianrking/BLE-Analyzer-Pro-rs/actions/workflows/release.yml/badge.svg)](https://github.com/tianrking/BLE-Analyzer-Pro-rs/actions/workflows/release.yml)
[![Rust](https://img.shields.io/badge/Rust-1.95%2B-f74c00?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-ctypes-3776AB?logo=python&logoColor=white)](python/ble_analyzer_pro.py)
[![C ABI](https://img.shields.io/badge/C%20ABI-stable-00599C?logo=c&logoColor=white)](include/ble_analyzer_pro.h)
[![libusb](https://img.shields.io/badge/libusb-1.0-4B8BBE)](https://libusb.info/)
[![Wireshark](https://img.shields.io/badge/Wireshark-PCAP-1679A7?logo=wireshark&logoColor=white)](https://www.wireshark.org/)
[![WSL2](https://img.shields.io/badge/WSL2-usbipd--win-4D4D4D?logo=windows&logoColor=white)](https://github.com/dorssel/usbipd-win)
[![License](https://img.shields.io/badge/License-Unlicense-lightgrey.svg)](LICENSE)

![Tags](https://img.shields.io/badge/BLE-advertising-00B894)
![Tags](https://img.shields.io/badge/Linux-capture-2D3436)
![Tags](https://img.shields.io/badge/CH582F-reverse--engineered-6C5CE7)
![Tags](https://img.shields.io/badge/pcap-linktype%20256-0984E3)
![Tags](https://img.shields.io/badge/Python-automation-FDCB6E)
![Tags](https://img.shields.io/badge/Linux-x86__64%20%7C%20arm64-00A86B)
![Tags](https://img.shields.io/badge/macOS-Intel%20%7C%20Apple%20Silicon-111111?logo=apple&logoColor=white)
![Tags](https://img.shields.io/badge/Windows-x86__64-0078D4?logo=windows&logoColor=white)

</div>

## Overview

`BLE Analyzer Pro RS` is a standalone Rust implementation for the **WCH BLE
Analyzer Pro**, a USB BLE advertising sniffer built around three CH582F MCU
devices behind a WCH USB hub.

The project keeps the proven Linux capture path small, typed, and reusable:

- a Rust capture core for USB I/O and packet decoding
- a CLI for direct captures
- a Wireshark-compatible pcap writer
- a stable C ABI for other runtimes
- a Python `ctypes` wrapper for scripts, dashboards, and analysis tools

The original C driver remains the protocol reference. This project is the
cleaner long-term Rust stack.

## What Works Today

| Area | Status | Notes |
| --- | --- | --- |
| Linux / WSL2 device enumeration | Complete | Finds the three `1a86:8009` MCU devices through libusb. |
| BLE advertising capture | Complete | Real hardware tested on channels 37, 38, and 39. |
| PCAP output | Complete | Wireshark-compatible BLE LL RF pcap, linktype 256. |
| CLI | Complete | List, capture, verbose output, duration, max-packet stop. |
| Python calls | Complete | `ctypes` wrapper over the native shared library. |
| C ABI | Complete | Stable header and `cdylib` shared library. |
| CI | Complete | rustfmt, clippy, tests, release build, Python syntax check. |
| Release artifacts | Complete | Linux x86_64, Linux arm64, macOS Intel, macOS Apple Silicon, Windows x86_64. |
| MAC filter / LTK / 2.4 GHz | Not claimed | Needs verified vendor USB traces before implementation. |

For strict feature status, see [`docs/FEATURE_MATRIX.md`](docs/FEATURE_MATRIX.md).
For module boundaries and design notes, see [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Repository Layout

```text
src/protocol.rs          USB commands, bulk reads, WCH frame decoder
src/device.rs            libusb enumeration and interface claim
src/packet.rs            packet model and formatting helpers
src/pcap.rs              Wireshark-compatible pcap writer
src/capture.rs           multi-MCU capture orchestration
src/ffi.rs               C ABI
src/main.rs              CLI
python/                  Python ctypes wrapper
examples/                Python live capture and pcap analysis helpers
include/                 C header
docs/                    architecture and feature matrix
scripts/                 local WSL helper scripts
.github/workflows/       CI and release binary automation
```

## Hardware Model

The analyzer enumerates as one hub plus three MCU devices.

```text
VID:PID    Role
1a86:8091  WCH CH334 USB hub
1a86:8009  CH582F BLE analyzer MCU, three devices total
```

A healthy Linux/WSL attach looks like this:

```bash
lsusb | grep 1a86
```

Expected:

```text
Bus 001 Device 032: ID 1a86:8009 QinHeng Electronics ble analyzer
Bus 001 Device 033: ID 1a86:8009 QinHeng Electronics ble analyzer
Bus 001 Device 034: ID 1a86:8009 QinHeng Electronics ble analyzer
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

WSL2 USB passthrough requires `usbipd-win` on Windows.

## Linux USB Permissions

On a native Linux host, install the included udev rules to allow non-root access
to the WCH MCU devices and hub:

```bash
sudo cp 99-wch-ble-analyzer.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
sudo groupadd -f plugdev
sudo usermod -aG plugdev "$USER"
```

Then unplug/replug the analyzer, log out/in, or restart WSL. The rule covers:

```text
1a86:8009  WCH BLE Analyzer MCU
1a86:8091  WCH CH334 hub
```

This solves Linux device permissions. On WSL2, you still also need USB
passthrough from Windows with `usbipd-win`.

## Platform Support

| Platform | Build artifact | Runtime status |
| --- | --- | --- |
| Linux x86_64 | `ble-analyzer-pro-rs-linux-x86_64.tar.gz` | Hardware capture verified on WSL2/Linux. |
| Linux arm64 | `ble-analyzer-pro-rs-linux-aarch64.tar.gz` | Expected to work with libusb; needs target-device smoke testing. |
| macOS Intel | `ble-analyzer-pro-rs-macos-x86_64.tar.gz` | Builds for integration; hardware capture not yet claimed. |
| macOS Apple Silicon | `ble-analyzer-pro-rs-macos-aarch64.tar.gz` | Builds for integration; hardware capture not yet claimed. |
| Windows x86_64 | `ble-analyzer-pro-rs-windows-x86_64.zip` | Builds with vcpkg libusb; direct USB capture needs driver validation. |

The project is designed to be portable at the Rust core and ABI layers. The
verified production path today is still Linux/WSL2 because USB driver binding
and analyzer behavior have been tested there with real hardware.

Release packaging is documented in [`docs/RELEASING.md`](docs/RELEASING.md).

## WSL2 USB Attach

On Windows PowerShell:

```powershell
wsl -l -v
usbipd list -u
```

If the three `1a86:8009` MCU devices are visible but not shared, bind them once
as Administrator. Replace `<BUSID_N>` with the bus IDs shown by
`usbipd list -u`:

```powershell
usbipd bind --busid <BUSID_1> --force
usbipd bind --busid <BUSID_2> --force
usbipd bind --busid <BUSID_3> --force
```

Attach them to WSL whenever they are not visible from Linux. Replace
`<YourDistro>` with the exact name from `wsl -l -v`, for example `Ubuntu`,
`Ubuntu-24.04`, or `Ubuntu-26.04`:

```powershell
usbipd attach --wsl <YourDistro> --busid <BUSID_1>
usbipd attach --wsl <YourDistro> --busid <BUSID_2>
usbipd attach --wsl <YourDistro> --busid <BUSID_3>
```

The bus IDs can change. Trust `usbipd list -u`; look for the three
`1a86:8009` devices. On this development machine the distro name happened to be
`Ubuntu-26.04` and the bus IDs happened to be `3-1`, `3-3`, and `3-4`; those are
examples, not project requirements.

This repository also includes a local helper:

```powershell
.\scripts\attach-wsl.ps1 -Distro <YourDistro> -BusIds <BUSID_1>,<BUSID_2>,<BUSID_3>
```

## Build

```bash
cd ~/BLE-Analyzer-pro-rs
cargo build --release
```

Outputs:

```text
target/release/ble-analyzer-pro
target/release/libble_analyzer_pro.so
```

Common development commands:

```bash
make check      # rustfmt, clippy, tests, Python syntax check
make release    # optimized CLI and shared library
make package    # local tar.gz package for the host Rust target
make list       # list attached analyzer MCU devices
make capture    # short verbose capture
make py-live    # Python ctypes live capture example
```

## CLI Usage

List attached devices:

```bash
./target/release/ble-analyzer-pro --list
```

Capture to pcap:

```bash
mkdir -p ~/captures
./target/release/ble-analyzer-pro -w ~/captures/ble.pcap
```

Capture with live packet output:

```bash
./target/release/ble-analyzer-pro -v -w ~/captures/ble.pcap
```

Short capture:

```bash
./target/release/ble-analyzer-pro -v -w /tmp/ble-rs-test.pcap --duration-ms 3000
```

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

Stop a long capture with `Ctrl+C`.

## Analyze Captures

File summary:

```bash
capinfos ~/captures/ble.pcap
```

Show packets:

```bash
tshark -r ~/captures/ble.pcap -c 20
```

Export useful BLE fields:

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

Find named BLE devices:

```bash
tshark -r ~/captures/ble.pcap \
  -Y 'btcommon.eir_ad.entry.device_name' \
  -T fields \
  -e btle.advertising_address \
  -e btcommon.eir_ad.entry.device_name \
| sort | uniq -c | sort -nr
```

Find busiest advertisers:

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

- PCAP encapsulation is `Bluetooth Low Energy Link Layer RF`.
- `btle_rf.channel` is the physical RF channel. BLE advertising logical
  channels map as `37 -> 0`, `38 -> 12`, `39 -> 39`.
- The hardware strips the real on-air CRC. The writer emits zero CRC bytes and
  marks packets checksum-inspected/checksum-valid so Wireshark decodes them.

## Python Usage

Build first:

```bash
cargo build --release
```

Run the live example:

```bash
PYTHONPATH=python python3 examples/python_live.py \
  --duration-ms 5000 \
  --max-packets 20 \
  -w /tmp/python-capture.pcap
```

Use from your own script:

```python
import sys
from pathlib import Path

sys.path.insert(0, str(Path.home() / "BLE-Analyzer-pro-rs" / "python"))

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

Callback convention:

```text
0       continue
non-0   stop capture
```

The packet metadata and PDU pointers are valid only during the callback.

## Troubleshooting

No devices from `--list`:

```bash
lsusb | grep 1a86
```

If WSL does not show three `1a86:8009` devices, check Windows:

```powershell
usbipd list -u
```

If state is `Shared (forced)` instead of `Attached`, the analyzer is shared by
Windows but not currently attached to WSL. This commonly happens after stopping
a capture or after the device re-enumerates. Reattach the three MCU devices:

```powershell
usbipd attach --wsl <YourDistro> --busid <BUSID_1>
usbipd attach --wsl <YourDistro> --busid <BUSID_2>
usbipd attach --wsl <YourDistro> --busid <BUSID_3>
```

Then confirm inside WSL:

```bash
lsusb | grep 1a86
./target/release/ble-analyzer-pro --list
```

If the original C driver says:

```text
No WCH BLE Analyzer MCUs found (VID 0x1A86 / PID 0x8009).
```

the same rule applies: first make sure `lsusb` shows the three `1a86:8009`
devices. Both the C and Rust tools use libusb and need the devices attached to
WSL.

Permission denied on Linux means the USB device is visible but your user cannot
open it:

```bash
sudo cp 99-wch-ble-analyzer.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
sudo groupadd -f plugdev
sudo usermod -aG plugdev "$USER"
```

Then log out/in or restart WSL.

## Roadmap

- release packaging and CI artifacts
- real captured USB frame fixtures for parser tests
- JSONL and CSV streaming output
- PyO3 bindings as an optional package layer
- protocol research for MAC filters, LTK/passkey handling, and custom 2.4 GHz mode
- reconnect/watchdog support for long capture sessions

## License

Unlicense. See [`LICENSE`](LICENSE).
