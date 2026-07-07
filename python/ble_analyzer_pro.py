from __future__ import annotations

import ctypes
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Iterable, Optional


_ROOT = Path(__file__).resolve().parents[1]
_DEFAULT_LIB = _ROOT / "target" / "release" / "libble_analyzer_pro.so"


class BleAnalyzerError(RuntimeError):
    pass


@dataclass(frozen=True)
class DeviceInfo:
    bus: int
    address: int
    vendor_id: int
    product_id: int


@dataclass(frozen=True)
class Packet:
    rssi: int
    pkt_type: int
    direction: int
    channel_index: int
    rf_channel: int
    access_addr: int
    src_addr: str
    dst_addr: str
    pkt_index: int
    timestamp_us: int
    interval_us: int
    pdu: bytes

    @property
    def type_name(self) -> str:
        return {
            0x00: "ADV_IND",
            0x01: "ADV_DIRECT_IND",
            0x02: "ADV_NONCONN_IND",
            0x03: "SCAN_REQ",
            0x04: "SCAN_RSP",
            0x05: "CONNECT_REQ",
            0x06: "ADV_SCAN_IND",
            0x07: "AUX_SCAN_REQ",
            0x08: "AUX_CONNECT_REQ",
            0x09: "AUX_COMMON",
            0x0A: "AUX_ADV_IND",
            0x0B: "AUX_SCAN_RSP",
            0x0C: "AUX_SYNC_IND",
            0x0D: "AUX_CONNECT_RSP",
            0x0E: "AUX_CHAIN_IND",
        }.get(self.pkt_type, f"0x{self.pkt_type:02x}")


class _DeviceInfo(ctypes.Structure):
    _fields_ = [
        ("bus", ctypes.c_uint8),
        ("address", ctypes.c_uint8),
        ("vendor_id", ctypes.c_uint16),
        ("product_id", ctypes.c_uint16),
    ]


class _Packet(ctypes.Structure):
    _fields_ = [
        ("rssi", ctypes.c_int8),
        ("pkt_type", ctypes.c_uint8),
        ("direction", ctypes.c_uint8),
        ("channel_index", ctypes.c_uint8),
        ("rf_channel", ctypes.c_uint8),
        ("reserved0", ctypes.c_uint8 * 3),
        ("access_addr", ctypes.c_uint32),
        ("src_addr", ctypes.c_uint8 * 6),
        ("dst_addr", ctypes.c_uint8 * 6),
        ("pkt_index", ctypes.c_uint64),
        ("timestamp_us", ctypes.c_uint64),
        ("interval_us", ctypes.c_uint64),
        ("pdu_len", ctypes.c_size_t),
    ]


_CALLBACK = ctypes.CFUNCTYPE(
    ctypes.c_int,
    ctypes.POINTER(_Packet),
    ctypes.POINTER(ctypes.c_uint8),
    ctypes.c_size_t,
    ctypes.c_void_p,
)


class _CaptureConfig(ctypes.Structure):
    _fields_ = [
        ("pcap_path", ctypes.c_char_p),
        ("phy", ctypes.c_uint8),
        ("channel", ctypes.c_uint8),
        ("verbose", ctypes.c_uint8),
        ("reserved0", ctypes.c_uint8),
        ("duration_ms", ctypes.c_uint64),
        ("max_packets", ctypes.c_uint64),
        ("callback", _CALLBACK),
        ("user", ctypes.c_void_p),
    ]


class CaptureReport(ctypes.Structure):
    _fields_ = [
        ("devices_opened", ctypes.c_size_t),
        ("total_packets", ctypes.c_uint64),
        ("elapsed_ms", ctypes.c_uint64),
    ]


def load_library(path: Optional[os.PathLike[str] | str] = None) -> ctypes.CDLL:
    lib_path = Path(path or os.environ.get("BLE_ANALYZER_PRO_LIB", _DEFAULT_LIB))
    if not lib_path.exists():
        raise BleAnalyzerError(
            f"shared library not found: {lib_path}; run `cargo build --release` first"
        )

    lib = ctypes.CDLL(str(lib_path))
    lib.wch_rs_version.restype = ctypes.c_char_p
    lib.wch_rs_last_error.restype = ctypes.c_char_p
    lib.wch_rs_find_devices.argtypes = [ctypes.POINTER(_DeviceInfo), ctypes.c_size_t]
    lib.wch_rs_find_devices.restype = ctypes.c_int
    lib.wch_rs_capture_blocking.argtypes = [
        ctypes.POINTER(_CaptureConfig),
        ctypes.POINTER(CaptureReport),
    ]
    lib.wch_rs_capture_blocking.restype = ctypes.c_int
    return lib


class BleAnalyzer:
    def __init__(self, library_path: Optional[os.PathLike[str] | str] = None):
        self.lib = load_library(library_path)

    @property
    def version(self) -> str:
        return self.lib.wch_rs_version().decode("utf-8")

    def list_devices(self, capacity: int = 8) -> list[DeviceInfo]:
        arr = (_DeviceInfo * capacity)()
        rc = self.lib.wch_rs_find_devices(arr, capacity)
        if rc < 0:
            raise BleAnalyzerError(self._last_error())
        count = min(rc, capacity)
        return [
            DeviceInfo(arr[i].bus, arr[i].address, arr[i].vendor_id, arr[i].product_id)
            for i in range(count)
        ]

    def capture(
        self,
        *,
        pcap_path: Optional[os.PathLike[str] | str] = None,
        duration_ms: int = 0,
        max_packets: int = 0,
        phy: int = 1,
        channel: int = 0,
        verbose: bool = False,
        on_packet: Optional[Callable[[Packet], bool | None]] = None,
    ) -> CaptureReport:
        path_bytes = None
        if pcap_path is not None:
            path_bytes = os.fsencode(os.fspath(pcap_path))

        callback_ref = _CALLBACK(0)
        if on_packet is not None:

            @_CALLBACK
            def callback(pkt_ptr, pdu_ptr, pdu_len, user):
                del user
                pkt = _packet_from_ffi(pkt_ptr.contents, pdu_ptr, pdu_len)
                keep_going = on_packet(pkt)
                return 0 if keep_going is not False else 1

            callback_ref = callback

        cfg = _CaptureConfig(
            path_bytes,
            phy,
            channel,
            1 if verbose else 0,
            0,
            duration_ms,
            max_packets,
            callback_ref,
            None,
        )
        report = CaptureReport()
        rc = self.lib.wch_rs_capture_blocking(ctypes.byref(cfg), ctypes.byref(report))
        if rc != 0:
            raise BleAnalyzerError(self._last_error())
        return report

    def _last_error(self) -> str:
        raw = self.lib.wch_rs_last_error()
        return raw.decode("utf-8", errors="replace") if raw else "unknown error"


def _packet_from_ffi(pkt: _Packet, pdu_ptr, pdu_len: int) -> Packet:
    pdu = ctypes.string_at(pdu_ptr, pdu_len) if pdu_ptr and pdu_len else b""
    return Packet(
        rssi=pkt.rssi,
        pkt_type=pkt.pkt_type,
        direction=pkt.direction,
        channel_index=pkt.channel_index,
        rf_channel=pkt.rf_channel,
        access_addr=pkt.access_addr,
        src_addr=_format_mac(pkt.src_addr),
        dst_addr=_format_mac(pkt.dst_addr),
        pkt_index=pkt.pkt_index,
        timestamp_us=pkt.timestamp_us,
        interval_us=pkt.interval_us,
        pdu=pdu,
    )


def _format_mac(addr: Iterable[int]) -> str:
    parts = list(addr)
    return ":".join(f"{b:02X}" for b in reversed(parts))
