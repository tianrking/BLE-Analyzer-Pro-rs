#ifndef BLE_ANALYZER_PRO_H
#define BLE_ANALYZER_PRO_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    uint8_t bus;
    uint8_t address;
    uint16_t vendor_id;
    uint16_t product_id;
} WchRsDeviceInfo;

typedef struct {
    int8_t rssi;
    uint8_t pkt_type;
    uint8_t direction;
    uint8_t channel_index;
    uint8_t rf_channel;
    uint8_t reserved0[3];
    uint32_t access_addr;
    uint8_t src_addr[6];
    uint8_t dst_addr[6];
    uint64_t pkt_index;
    uint64_t timestamp_us;
    uint64_t interval_us;
    size_t pdu_len;
} WchRsPacket;

typedef int (*WchRsPacketCallback)(
    const WchRsPacket *packet,
    const uint8_t *pdu,
    size_t pdu_len,
    void *user);

typedef struct {
    const char *pcap_path;
    uint8_t phy;
    uint8_t channel;
    uint8_t verbose;
    uint8_t reserved0;
    uint64_t duration_ms;
    uint64_t max_packets;
    WchRsPacketCallback callback;
    void *user;
} WchRsCaptureConfig;

typedef struct {
    size_t devices_opened;
    uint64_t total_packets;
    uint64_t elapsed_ms;
} WchRsCaptureReport;

const char *wch_rs_version(void);
const char *wch_rs_last_error(void);
int wch_rs_find_devices(WchRsDeviceInfo *out, size_t capacity);
int wch_rs_capture_blocking(const WchRsCaptureConfig *cfg, WchRsCaptureReport *report_out);

#ifdef __cplusplus
}
#endif

#endif
