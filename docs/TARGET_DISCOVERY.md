# Target Discovery SOP

This workflow helps identify which BLE advertiser corresponds to a physical
object when many devices are visible.

## English

1. Run a discovery scan:

   ```bash
   ble-analyzer-pro discover --duration-ms 15000 --sort rssi-change
   ```

2. Look for candidates with useful identity fields:

   - `kind=named`: has a local name.
   - `kind=manufacturer`: has manufacturer data but no name.
   - `kind=service`: has service UUID or service-data fields.
   - `kind=address-only`: only the advertiser address was visible.

3. Move the suspected device near and far from the analyzer.

   A strong RSSI delta, such as `20 dB` or more, is a practical signal that the
   candidate is physically the device you are moving.

4. Track the selected address live:

   ```bash
   ble-analyzer-pro discover --target AA:BB:CC:DD:EE:FF --duration-ms 30000
   ```

5. Capture only that advertiser:

   ```bash
   ble-analyzer-pro -v -w target.pcap --filter-addr AA:BB:CC:DD:EE:FF
   ```

Random BLE addresses can rotate. If the address changes, use the combination of
name, manufacturer data, service data, RSSI movement, and payload changes while
operating the device.

## 中文

1. 先做一次发现扫描：

   ```bash
   ble-analyzer-pro discover --duration-ms 15000 --sort rssi-change
   ```

2. 观察候选类型：

   - `kind=named`：广播里有设备名。
   - `kind=manufacturer`：有厂商数据，但没有名字。
   - `kind=service`：有 service UUID 或 service data。
   - `kind=address-only`：只看到广播地址。

3. 把疑似设备靠近分析仪、再拿远。

   如果 RSSI delta 明显变大，例如超过 `20 dB`，基本可以说明你移动的就是这个候选设备。

4. 锁定地址后实时追踪：

   ```bash
   ble-analyzer-pro discover --target AA:BB:CC:DD:EE:FF --duration-ms 30000
   ```

5. 只抓这个设备：

   ```bash
   ble-analyzer-pro -v -w target.pcap --filter-addr AA:BB:CC:DD:EE:FF
   ```

如果设备使用随机 BLE 地址，不要只依赖 MAC。要结合设备名、厂商数据、service
data、RSSI 变化，以及你操作设备时 payload 是否同步变化。
