# Feature Matrix

This table is intentionally strict. A feature is marked complete only when it is
implemented in Rust and has been exercised against the real WCH BLE Analyzer Pro
hardware on Linux/WSL.

| Area | Status | Notes |
| --- | --- | --- |
| Enumerate `1a86:8009` MCUs | Complete | Uses `rusb` and opens all visible MCU devices. |
| Non-root Linux access | Complete | `99-wch-ble-analyzer.rules` included. |
| WSL2 USB workflow | Documented | `usbipd-win` commands and helper script included. |
| BLE advertising capture | Complete | Real-device tested on channels 37/38/39. |
| Three-MCU auto channel assignment | Complete | Auto mode assigns 37, 38, 39 when all three MCUs are present. |
| Per-packet metadata | Complete | Timestamp, RSSI, channel, PDU type, source/destination addresses. |
| Raw BLE LL PDU callback | Complete | Exposed to Rust and C ABI/Python callers. |
| PCAP output | Complete | Wireshark-compatible BLE LL with pseudo-header, linktype 256. |
| CLI | Complete | List, capture, pcap output, verbose output, duration, max packets. |
| C ABI | Complete | Stable C header and shared library for external callers. |
| Python wrapper | Complete | `ctypes` wrapper plus live capture example. |
| CI | Complete | rustfmt, clippy, tests, release build, Python syntax check. |
| Unit tests | Partial | Decoder and channel mapping covered. More real fixtures should be added. |
| PHY 1M | Complete | Default path used in smoke tests. |
| PHY 2M / Coded | Experimental | Command field exists, but needs real capture validation. |
| MAC filter | Not implemented | Need USB command evidence from vendor app before implementing. |
| LTK/passkey decryption | Not implemented | Need protocol evidence and pairing/session model. |
| Custom 2.4 GHz mode | Not implemented | Need USB command evidence from vendor app. |
| Firmware upload/state recovery | Not implemented | Current path assumes firmware-present state `0x33/0x32`. |
| Long soak / reconnect | Not complete | Short real-device smoke tests pass; long capture watchdog is future work. |

## Why Not Claim Full Feature Parity Yet?

The known, verified command sequence is enough for Linux BLE advertising capture.
Other options shown by the original C CLI are not fully documented by verified
USB traces yet. Guessing those payload fields would create a project that looks
complete but fails under real hardware. This repo keeps unsupported features
explicit until the vendor app traffic or firmware behavior is proven.
