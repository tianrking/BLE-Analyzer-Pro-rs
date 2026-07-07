# Architecture

The project is organized around one Rust capture core. Every public surface
uses that same core instead of duplicating protocol logic.

```text
WCH USB devices
  -> src/device.rs      enumerate/open/claim MCU interfaces
  -> src/protocol.rs    vendor commands, bulk reads, WCH frame decode
  -> src/packet.rs      typed packet model and formatting
  -> src/capture.rs     multi-MCU capture loop and stop conditions
  -> src/pcap.rs        Wireshark-compatible pcap writer
      -> src/main.rs    CLI
      -> src/ffi.rs     C ABI
          -> python/    ctypes wrapper
```

## Design Rules

- Protocol parsing lives in `src/protocol.rs`.
- Long-running orchestration lives in `src/capture.rs`.
- Output formats do not own USB state.
- Python never touches raw USB; it calls the Rust ABI.
- The C ABI owns no borrowed data after a callback returns.
- Unsupported vendor protocol features are explicit and documented instead of
  being represented as fake knobs.

## Capture Flow

1. Create a `rusb::Context`.
2. Enumerate `VID 1a86 / PID 8009` devices.
3. Open each MCU and claim interface 0.
4. Send the verified initialization sequence:
   - `AA 84 ... "BLEAnalyzer&IAP"`
   - `AA 81 ... phy/channel/config`
   - `AA A1 00 00`
5. Assign channels:
   - auto mode with three MCUs: `37 / 38 / 39`
   - pinned mode: all MCUs use the requested channel
6. Drain each MCU buffer with a short timeout.
7. When all MCUs are idle, poll with a longer timeout.
8. For each decoded packet:
   - optionally write pcap
   - call the Rust/FFI/Python packet callback
   - stop cleanly on callback false, max packet count, duration, or Ctrl+C

## ABI Boundary

The ABI exports plain C structs:

- `WchRsDeviceInfo`
- `WchRsPacket`
- `WchRsCaptureConfig`
- `WchRsCaptureReport`

The callback receives:

- a pointer to packet metadata
- a pointer to raw BLE LL PDU bytes
- PDU length
- opaque user pointer

The packet and PDU pointers are valid only during the callback. Callers that
need data later must copy it.

## Why `ctypes` First?

PyO3 is useful for a polished Python package, but the first stable integration
layer should be the C ABI:

- it works with any CPython version available in WSL
- it can be reused by C, C++, Zig, Go FFI, and other runtimes
- it avoids wheel/package work while the USB protocol is still evolving
- the ABI can stay stable even if the internal Rust API changes

PyO3 can be added later as a convenience layer on top of the same Rust core.

## Error Handling

Rust APIs return `Result<T, Error>`. The C ABI returns `0` for success and `-1`
for failure, with `wch_rs_last_error()` holding the last error message.

Callback stop is not an error. It is a normal way for Python or C callers to end
a capture after seeing enough packets.

## PCAP Notes

The hardware validates and strips the real on-air CRC before USB delivery. The
pcap writer appends zero CRC bytes and sets checksum-inspected/checksum-valid
flags in the BLE pseudo-header so Wireshark decodes packets correctly without
inventing a fake CRC.

## Future Architecture Hooks

The code is ready for:

- richer output sinks such as JSONL or CSV
- packet fixture tests using captured USB frames
- a reconnect/watchdog layer around `run_capture`
- PyO3 bindings that call the existing Rust API
- protocol-specific modules once MAC filters, LTK/passkey, and 2.4 GHz mode are
  proven from USB traces
