#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import subprocess
from collections import Counter


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("pcap")
    args = parser.parse_args()

    cmd = [
        "tshark",
        "-r",
        args.pcap,
        "-T",
        "fields",
        "-e",
        "btle_rf.channel",
        "-e",
        "btle_rf.signal_dbm",
        "-e",
        "btle.advertising_header.pdu_type",
        "-e",
        "btle.advertising_address",
        "-e",
        "btcommon.eir_ad.entry.device_name",
        "-E",
        "separator=,",
        "-E",
        "quote=d",
    ]
    proc = subprocess.run(cmd, check=True, text=True, stdout=subprocess.PIPE)

    channels = Counter()
    addresses = Counter()
    names = Counter()
    rssis = []

    for row in csv.reader(proc.stdout.splitlines()):
        if len(row) < 5:
            continue
        channel, rssi, _ptype, addr, name = row[:5]
        if channel:
            channels[channel] += 1
        if addr:
            addresses[addr] += 1
        if name:
            names[(addr, name)] += 1
        if rssi:
            try:
                rssis.append(int(rssi))
            except ValueError:
                pass

    print("channels")
    for item, count in channels.most_common():
        print(f"  {item:>2}: {count}")

    if rssis:
        print(f"rssi avg={sum(rssis)/len(rssis):.1f} min={min(rssis)} max={max(rssis)}")

    print("top addresses")
    for item, count in addresses.most_common(20):
        print(f"  {count:5d} {item}")

    print("named devices")
    for (addr, name), count in names.most_common(20):
        print(f"  {count:5d} {addr} {name}")


if __name__ == "__main__":
    main()
