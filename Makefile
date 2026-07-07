.PHONY: all fmt lint test build release pycheck check package clean list capture py-live analyze

PCAP ?= /tmp/ble-analyzer-pro-rs.pcap
DURATION_MS ?= 3000
MAX_PACKETS ?= 20
TARGET ?= $(shell rustc -vV | sed -n 's/^host: //p')
PACKAGE ?= ble-analyzer-pro-rs-$(TARGET)

all: check release

fmt:
	cargo fmt

lint:
	cargo clippy --all-targets --locked -- -D warnings

test:
	cargo test --all-targets --locked

build:
	cargo build

release:
	cargo build --release --locked

pycheck:
	python3 -m py_compile python/ble_analyzer_pro.py examples/python_live.py examples/analyze_pcap.py

check: fmt lint test pycheck

package:
	cargo build --release --locked --target $(TARGET)
	scripts/package-release.sh $(TARGET) $(PACKAGE)

clean:
	cargo clean
	rm -f *.pcap *.pcapng

list: release
	./target/release/ble-analyzer-pro --list

capture: release
	./target/release/ble-analyzer-pro -v -w $(PCAP) --duration-ms $(DURATION_MS)

py-live: release
	PYTHONPATH=python python3 examples/python_live.py --duration-ms $(DURATION_MS) --max-packets $(MAX_PACKETS) -w $(PCAP)

analyze:
	python3 examples/analyze_pcap.py $(PCAP)
