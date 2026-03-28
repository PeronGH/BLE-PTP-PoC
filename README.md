# BLE Precision Touchpad PoC

Proof of concept: an ESP32-S3 advertising as a **BLE HOGP HID device** with a **Windows Precision Touchpad (PTP)** descriptor. Windows recognises it as a native precision touchpad — no driver installation required on the host.

## Result

**Confirmed working.** Windows 11 pairs with the ESP32-S3 over BLE, reads the PTP feature reports, and sets Input Mode = 3 (PTP collection mode). The cursor moves from hardcoded touch reports.

```
I ble_ptp_poc::ble_hid: HIDD: feature SET report_id=0x04 data=[03]
I ble_ptp_poc::ble_hid: *** Windows set Input Mode = 3 (PTP) — precision touchpad confirmed ***
```

This proves PTP over BLE HOGP is viable. This PoC was built to de-risk the [esp32-universal-control](https://github.com/PeronGH/esp32-universal-control) project, which will be developed separately.

## What it does

1. Initialises the ESP32-S3 BLE stack (Bluedroid, BLE-only)
2. Creates a HOGP HID device with a 5-finger PTP report descriptor
3. Pre-loads feature reports: device capabilities, PTPHQA certification blob
4. Advertises as "ESP32 UC PTP"
5. On connect, sends a repeating single-finger horizontal sweep

## Hardware

- ESP32-S3 dev board (any board with BLE, ~$5)
- USB-C cable to a Mac/Linux host for flashing
- A Windows 11 machine for BLE pairing and PTP verification

## Building and flashing

Requires the [ESP32 Rust toolchain](https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html) (`espup`, `espflash`, `ldproxy`).

```sh
cargo run
```

This builds, flashes, and opens the serial monitor. Pair "ESP32 UC PTP" from Windows Bluetooth settings.

## PTP descriptor

The HID report descriptor is translated byte-for-byte from [imbushuo/mac-precision-touchpad](https://github.com/imbushuo/mac-precision-touchpad) (`WellspringT2.h`). It contains two top-level collections:

| TLC | Purpose | Report IDs |
|-----|---------|-----------|
| Digitizer / Touch Pad | 5-finger multitouch input + feature reports | 0x05 (input), 0x07 (device caps), 0x08 (PTPHQA cert) |
| Digitizer / Configuration | Input mode and function switch | 0x04 (input mode), 0x06 (function switch) |

## Known issues

- `BTM_BleWriteAdvData, Partial data write into ADV` — the 128-bit service UUID in advertising data exceeds the 31-byte BLE advertising limit. Cosmetic; does not affect pairing.
- `BT_GATT: attribute value too long, truncated to 20` — the 256-byte PTPHQA cert blob is truncated by the default BLE ATT MTU. Windows still accepts it.
- No reconnect-after-disconnect logic beyond restarting advertising.

## License

See [LICENSE](LICENSE).
