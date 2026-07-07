#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))

from ble_analyzer_pro import BleAnalyzer


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("-w", "--write", default="python-capture.pcap")
    parser.add_argument("--duration-ms", type=int, default=5000)
    parser.add_argument("--max-packets", type=int, default=20)
    args = parser.parse_args()

    analyzer = BleAnalyzer()
    print("lib version:", analyzer.version)
    print("devices:", analyzer.list_devices())

    seen = 0

    def on_packet(pkt):
        nonlocal seen
        seen += 1
        print(
            f"{seen:04d} ch{pkt.channel_index:02d}/rf{pkt.rf_channel:02d} "
            f"{pkt.type_name:<16} rssi={pkt.rssi:4d} addr={pkt.src_addr} "
            f"pdu={pkt.pdu[:12].hex()}"
        )
        return seen < args.max_packets

    report = analyzer.capture(
        pcap_path=args.write,
        duration_ms=args.duration_ms,
        max_packets=args.max_packets,
        on_packet=on_packet,
    )
    print(
        f"done: packets={report.total_packets} "
        f"devices={report.devices_opened} elapsed_ms={report.elapsed_ms}"
    )


if __name__ == "__main__":
    main()
