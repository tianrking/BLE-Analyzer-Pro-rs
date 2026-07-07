<div align="center">

# BLE Analyzer Pro RS

**面向 WCH / 沁恒 BLE Analyzer Pro 的 Rust 原生抓包栈。**

简体中文 | [English](README.md)

[![CI](https://github.com/tianrking/BLE-Analyzer-Pro-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/tianrking/BLE-Analyzer-Pro-rs/actions/workflows/ci.yml)
[![Release Binaries](https://github.com/tianrking/BLE-Analyzer-Pro-rs/actions/workflows/release.yml/badge.svg)](https://github.com/tianrking/BLE-Analyzer-Pro-rs/actions/workflows/release.yml)
[![Rust](https://img.shields.io/badge/Rust-1.95%2B-f74c00?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-ctypes-3776AB?logo=python&logoColor=white)](python/ble_analyzer_pro.py)
[![C ABI](https://img.shields.io/badge/C%20ABI-stable-00599C?logo=c&logoColor=white)](include/ble_analyzer_pro.h)
[![libusb](https://img.shields.io/badge/libusb-1.0-4B8BBE)](https://libusb.info/)
[![Wireshark](https://img.shields.io/badge/Wireshark-PCAP-1679A7?logo=wireshark&logoColor=white)](https://www.wireshark.org/)
[![WSL2](https://img.shields.io/badge/WSL2-usbipd--win-4D4D4D?logo=windows&logoColor=white)](https://github.com/dorssel/usbipd-win)
[![License](https://img.shields.io/badge/License-Unlicense-lightgrey.svg)](LICENSE)

![标签](https://img.shields.io/badge/BLE-广播抓包-00B894)
![标签](https://img.shields.io/badge/Linux-原生采集-2D3436)
![标签](https://img.shields.io/badge/CH582F-逆向协议-6C5CE7)
![标签](https://img.shields.io/badge/pcap-linktype%20256-0984E3)
![标签](https://img.shields.io/badge/Python-自动化-FDCB6E)
![标签](https://img.shields.io/badge/Linux-x86__64%20%7C%20arm64-00A86B)
![标签](https://img.shields.io/badge/macOS-Intel%20%7C%20Apple%20Silicon-111111?logo=apple&logoColor=white)
![标签](https://img.shields.io/badge/Windows-x86__64-0078D4?logo=windows&logoColor=white)

</div>

## 项目简介

`BLE Analyzer Pro RS` 是一个独立的 Rust 实现，用于在 Linux / WSL2 下驱动
**WCH BLE Analyzer Pro**。这个硬件由一个 WCH USB Hub 和三颗 CH582F BLE MCU
组成，可以同时覆盖 BLE 广播信道。

这个仓库的目标不是把脚本堆起来，而是做成一个可长期维护的抓包内核：

- Rust 内核负责 USB I/O、协议解析和 pcap 输出
- CLI 负责直接抓包
- C ABI 负责给其他语言调用
- Python `ctypes` 包装层负责自动化、统计、仪表盘和上层分析
- Wireshark / tshark 负责后处理和协议视图

原始 C 驱动仍然是协议参考；本项目是面向长期维护的 Rust 工程化版本。

## 当前能力

| 模块 | 状态 | 说明 |
| --- | --- | --- |
| Linux / WSL2 设备枚举 | 已完成 | 通过 libusb 找到三颗 `1a86:8009` MCU。 |
| BLE 广播抓包 | 已完成 | 真机验证过 37、38、39 三个广播信道。 |
| PCAP 输出 | 已完成 | Wireshark 可直接打开，BLE LL RF，linktype 256。 |
| CLI | 已完成 | 支持 list、capture、verbose、duration、max-packets。 |
| 目标发现 | 已完成 | 按身份字段和 RSSI 变化排序候选，然后实时追踪目标。 |
| 软件地址过滤 | 已完成 | 通过 `--filter-addr` 只抓一个选定广播设备。 |
| Python 调用 | 已完成 | 通过 `ctypes` 调用原生 `.so`。 |
| C ABI | 已完成 | 提供稳定头文件和 shared library。 |
| CI | 已完成 | rustfmt、clippy、测试、release build、Python 语法检查。 |
| Release 产物 | 已完成 | Linux x86_64、Linux arm64、macOS Intel、macOS Apple Silicon、Windows x86_64。 |
| MAC filter / LTK / 2.4G | 不宣称完成 | 需要先拿到可靠的厂商 USB trace，不能靠猜。 |

严格功能表见 [`docs/FEATURE_MATRIX.md`](docs/FEATURE_MATRIX.md)。
架构说明见 [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)。

## 目录结构

```text
src/protocol.rs          USB 命令、bulk 读取、WCH 帧解析
src/device.rs            libusb 枚举、打开设备、claim interface
src/packet.rs            抓包数据结构和格式化
src/ad.rs                BLE advertising data 解析
src/discovery.rs         目标发现和 RSSI 聚合
src/pcap.rs              Wireshark 兼容 pcap writer
src/capture.rs           多 MCU 抓包调度
src/ffi.rs               C ABI
src/main.rs              CLI
python/                  Python ctypes 包装
examples/                Python 实时抓包和 pcap 分析示例
include/                 C 头文件
docs/                    架构与功能矩阵
scripts/                 本地 WSL 辅助脚本
.github/workflows/       CI 和多平台 release binary 自动化
```

## 硬件模型

设备在 USB 上表现为一个 hub 加三颗 MCU：

```text
VID:PID    作用
1a86:8091  WCH CH334 USB Hub
1a86:8009  CH582F BLE Analyzer MCU，总共三颗
```

WSL / Linux 下正常挂载后应该能看到：

```bash
lsusb | grep 1a86
```

示例输出：

```text
Bus 001 Device 032: ID 1a86:8009 QinHeng Electronics ble analyzer
Bus 001 Device 033: ID 1a86:8009 QinHeng Electronics ble analyzer
Bus 001 Device 034: ID 1a86:8009 QinHeng Electronics ble analyzer
```

## 依赖

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

WSL2 下使用 USB 设备需要 Windows 侧安装 `usbipd-win`。

## Linux USB 权限

在原生 Linux 主机上，安装仓库自带的 udev 规则后，普通用户就可以访问 WCH
MCU 设备和 hub：

```bash
sudo cp 99-wch-ble-analyzer.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
sudo groupadd -f plugdev
sudo usermod -aG plugdev "$USER"
```

然后拔插一次设备、重新登录，或者重启 WSL。规则覆盖：

```text
1a86:8009  WCH BLE Analyzer MCU
1a86:8091  WCH CH334 hub
```

这一步解决的是 Linux 内部的设备权限。WSL2 还需要 Windows 侧通过
`usbipd-win` 把 USB 设备透传进来。

## 平台支持

| 平台 | 构建产物 | 运行状态 |
| --- | --- | --- |
| Linux x86_64 | `ble-analyzer-pro-rs-linux-x86_64.tar.gz` | 已在 WSL2/Linux 真机抓包验证。 |
| Linux arm64 | `ble-analyzer-pro-rs-linux-aarch64.tar.gz` | 预期可用，需要目标设备 smoke test。 |
| macOS Intel | `ble-analyzer-pro-rs-macos-x86_64.tar.gz` | 可构建，用于集成；暂不宣称真机抓包已验证。 |
| macOS Apple Silicon | `ble-analyzer-pro-rs-macos-aarch64.tar.gz` | 可构建，用于集成；暂不宣称真机抓包已验证。 |
| Windows x86_64 | `ble-analyzer-pro-rs-windows-x86_64.zip` | 通过 vcpkg libusb 构建；直接 USB 抓包还需要驱动验证。 |

项目的 Rust 内核和 ABI 层按跨平台设计；当前已经可靠验证的生产路径仍然是
Linux / WSL2，因为 USB 驱动绑定和设备行为都在这个路径上经过真机测试。

发布流程见 [`docs/RELEASING.md`](docs/RELEASING.md)。

## WSL2 USB 挂载

在 Windows PowerShell 中查看：

```powershell
wsl -l -v
usbipd list -u
```

如果三颗 `1a86:8009` 可见但还没共享，管理员 PowerShell 里先 bind 一次。
把 `<BUSID_N>` 换成 `usbipd list -u` 里显示的 busid：

```powershell
usbipd bind --busid <BUSID_1> --force
usbipd bind --busid <BUSID_2> --force
usbipd bind --busid <BUSID_3> --force
```

每次 WSL 里看不到设备时重新 attach。把 `<YourDistro>` 换成 `wsl -l -v`
里显示的发行版名字，比如 `Ubuntu`、`Ubuntu-24.04`、`Ubuntu-26.04`：

```powershell
usbipd attach --wsl <YourDistro> --busid <BUSID_1>
usbipd attach --wsl <YourDistro> --busid <BUSID_2>
usbipd attach --wsl <YourDistro> --busid <BUSID_3>
```

busid 可能会变化。以 `usbipd list -u` 为准，找三条 `1a86:8009`。本开发机
当时的发行版名是 `Ubuntu-26.04`，busid 是 `3-1`、`3-3`、`3-4`；这些只是
示例，不是项目要求。

本仓库也带了一个本机辅助脚本：

```powershell
.\scripts\attach-wsl.ps1 -Distro <YourDistro> -BusIds <BUSID_1>,<BUSID_2>,<BUSID_3>
```

## 构建

```bash
cd ~/BLE-Analyzer-pro-rs
cargo build --release
```

输出文件：

```text
target/release/ble-analyzer-pro
target/release/libble_analyzer_pro.so
```

常用开发命令：

```bash
make check      # rustfmt、clippy、测试、Python 语法检查
make release    # 构建优化版 CLI 和 shared library
make package    # 为当前 Rust host target 生成本地 tar.gz 包
make list       # 列出已挂载的 analyzer MCU
make capture    # 短时间 verbose 抓包
make discover   # 排序 BLE 广播候选设备
make track ADDR=AA:BB:CC:DD:EE:FF
make py-live    # Python ctypes 实时抓包示例
```

## CLI 使用

列出设备：

```bash
./target/release/ble-analyzer-pro --list
```

抓包到 pcap：

```bash
mkdir -p ~/captures
./target/release/ble-analyzer-pro -w ~/captures/ble.pcap
```

抓包并实时打印：

```bash
./target/release/ble-analyzer-pro -v -w ~/captures/ble.pcap
```

短时间抓包：

```bash
./target/release/ble-analyzer-pro -v -w /tmp/ble-rs-test.pcap --duration-ms 3000
```

常用参数：

```text
--list                 列出 WCH analyzer MCU
-v, --verbose          抓包时打印 decoded packet
-w, --write FILE       写出 Wireshark 兼容 pcap
-p, --phy N            PHY 值 1..4，默认 1
-c, --channel N        BLE 信道 0..39；0 表示自动分配 37/38/39
--filter-addr ADDR     只打印/写出匹配某个 BLE 地址的包
--duration-ms N        N 毫秒后停止
--max-packets N        N 个包后停止
--quiet-init           不打印 USB 初始化日志
```

长时间抓包按 `Ctrl+C` 停止。

## 目标发现 SOP

内置 `discover` 工作流就是把“全量扫描 -> 找候选 -> 靠近/远离看 RSSI ->
锁定目标 -> 过滤抓包”做进 CLI。

先给附近广播设备排序：

```bash
./target/release/ble-analyzer-pro discover --duration-ms 15000 --sort rssi-change
```

常用排序：

```text
rssi-change    RSSI 近/远变化最大的候选排前面
strongest      信号最强的候选排前面
packets        广播最频繁的候选排前面
name           按设备名聚合
manufacturer   按厂商数据聚合
kind           按 named / manufacturer / service / address-only 分组
```

表格会显示地址、类型、包数、last/avg/min/max RSSI、RSSI delta、设备名、
manufacturer ID、service 字段和 PDU 类型。带 `*` 的行表示 RSSI 变化超过阈值。

选中地址后，移动设备并实时观察：

```bash
./target/release/ble-analyzer-pro discover \
  --target AA:BB:CC:DD:EE:FF \
  --duration-ms 30000
```

只抓这个目标设备：

```bash
./target/release/ble-analyzer-pro -v \
  -w ~/captures/target.pcap \
  --filter-addr AA:BB:CC:DD:EE:FF
```

这是软件过滤。硬件仍然接收完整 BLE 广播流，Rust 管线会在打印和写 pcap 前过滤。
如果设备会随机换 BLE 地址，不要只看 MAC，要结合 RSSI 变化、设备名、厂商数据、
service data，以及操作设备时 payload 是否同步变化。

更多说明见 [`docs/TARGET_DISCOVERY.md`](docs/TARGET_DISCOVERY.md)。

## 分析 pcap

文件摘要：

```bash
capinfos ~/captures/ble.pcap
```

查看前几包：

```bash
tshark -r ~/captures/ble.pcap -c 20
```

导出常用 BLE 字段：

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

查找带名称的 BLE 设备：

```bash
tshark -r ~/captures/ble.pcap \
  -Y 'btcommon.eir_ad.entry.device_name' \
  -T fields \
  -e btle.advertising_address \
  -e btcommon.eir_ad.entry.device_name \
| sort | uniq -c | sort -nr
```

查找广播最频繁的地址：

```bash
tshark -r ~/captures/ble.pcap \
  -Y 'btle.advertising_address' \
  -T fields -e btle.advertising_address \
| sort | uniq -c | sort -nr | head -20
```

查看 Manufacturer company ID：

```bash
tshark -r ~/captures/ble.pcap \
  -Y 'btcommon.eir_ad.entry.company_id' \
  -T fields \
  -e btle.advertising_address \
  -e btcommon.eir_ad.entry.company_id \
| sort | uniq -c | sort -nr | head -30
```

Wireshark 说明：

- pcap 封装是 `Bluetooth Low Energy Link Layer RF`。
- `btle_rf.channel` 是物理 RF 信道。BLE 广播逻辑信道映射为
  `37 -> 0`、`38 -> 12`、`39 -> 39`。
- 硬件会校验并剥离真实空中 CRC。本项目写入零 CRC 字节，并在伪头中标记
  checksum-inspected/checksum-valid，让 Wireshark 正常解码。

## Python 调用

先构建 shared library：

```bash
cargo build --release
```

运行实时示例：

```bash
PYTHONPATH=python python3 examples/python_live.py \
  --duration-ms 5000 \
  --max-packets 20 \
  -w /tmp/python-capture.pcap
```

在自己的脚本中调用：

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

Python wrapper 默认加载：

```text
target/release/libble_analyzer_pro.so
```

也可以手动指定：

```bash
export BLE_ANALYZER_PRO_LIB=/path/to/libble_analyzer_pro.so
```

## C ABI

头文件：

```text
include/ble_analyzer_pro.h
```

导出函数：

```c
const char *wch_rs_version(void);
const char *wch_rs_last_error(void);
int wch_rs_find_devices(WchRsDeviceInfo *out, size_t capacity);
int wch_rs_capture_blocking(const WchRsCaptureConfig *cfg,
                            WchRsCaptureReport *report_out);
```

回调返回值约定：

```text
0       继续抓包
非 0    停止抓包
```

回调中的 packet metadata 和 PDU 指针只在回调期间有效，需要长期保存时请复制。

## 排障

`--list` 没有设备：

```bash
lsusb | grep 1a86
```

如果 WSL 看不到三条 `1a86:8009`，回 Windows 查：

```powershell
usbipd list -u
```

如果状态是 `Shared (forced)` 而不是 `Attached`，说明 Windows 已共享但还没挂进
WSL。停止抓包或设备重新枚举后经常会出现这个状态，重新 attach 即可：

```powershell
usbipd attach --wsl <YourDistro> --busid <BUSID_1>
usbipd attach --wsl <YourDistro> --busid <BUSID_2>
usbipd attach --wsl <YourDistro> --busid <BUSID_3>
```

然后在 WSL 里确认：

```bash
lsusb | grep 1a86
./target/release/ble-analyzer-pro --list
```

如果原始 C 驱动报：

```text
No WCH BLE Analyzer MCUs found (VID 0x1A86 / PID 0x8009).
```

处理方式一样：先确认 `lsusb` 能看到三颗 `1a86:8009`。C 版和 Rust 版都走
libusb，都要求设备已经 attach 到 WSL。

Linux 下权限不足，表示 USB 设备已经可见，但当前用户没有权限打开：

```bash
sudo cp 99-wch-ble-analyzer.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
sudo groupadd -f plugdev
sudo usermod -aG plugdev "$USER"
```

然后重新登录或重启 WSL。

## 路线图

- release 包和 CI artifact
- 真实 USB frame fixture 测试
- JSONL / CSV 流式输出
- 可选 PyO3 包装层
- MAC filter、LTK/passkey、custom 2.4 GHz 协议补完
- 长时间抓包的 reconnect/watchdog 支持

## License

Unlicense。见 [`LICENSE`](LICENSE)。
