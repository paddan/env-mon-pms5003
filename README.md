# ESP32-C3 Super Mini + PMS5003 + BME280 + 2.66in e-paper (Rust)

`no_std` ESP32-C3 firmware using:

- `PMS5003` over UART1 for PM readings
- `BME280` over I2C for temperature, humidity, pressure
- `Waveshare 2.66" e-paper` over SPI for on-device display

## Clear Wiring Diagram

ESP32-C3 Super Mini side:

| ESP32-C3 pin | Connects to                                  | Notes                           |
|:---------------|:-------------------------------------------|:--------------------------------|
| `5V`           | PMS5003 `VCC`                              | PMS5003 power (5V)              |
| `GND`          | PMS5003 `GND`, BME280 `GND`, e-paper `GND` | Shared ground                   |
| `GPIO4`        | PMS5003 `TXD`                              | UART1 RX                        |
| `GPIO5`        | PMS5003 `RXD`                              | UART1 TX (wake/active commands) |
| `3V3`          | BME280 `VIN/VCC`, e-paper `VCC`            | 3.3V rail                       |
| `GPIO6`        | BME280 `SDA`                               | I2C SDA                         |
| `GPIO7`        | BME280 `SCL`                               | I2C SCL                         |
| `GPIO10`       | e-paper `DIN`                              | SPI MOSI                        |
| `GPIO8`        | e-paper `CLK`                              | SPI SCK                         |
| `GPIO3`        | e-paper `CS`                               | SPI chip select                 |
| `GPIO2`        | e-paper `DC`                               | Data/command                    |
| `GPIO1`        | e-paper `RST`                              | Display reset                   |
| `GPIO0`        | e-paper `BUSY`                             | Display busy input              |

### PMS5003 8-pin connector mapping (sensor-side)

- `PIN1 VCC` -> ESP32-C3 Super Mini `5V`
- `PIN2 GND` -> ESP32-C3 Super Mini `GND`
- `PIN4 RX` -> ESP32-C3 Super Mini `GPIO5` (optional but recommended)
- `PIN5 TX` -> ESP32-C3 Super Mini `GPIO4`
- `PIN3 SET`, `PIN6 RESET`, `PIN7/8 NC` -> leave unconnected

### BME280 notes

- Firmware probes both `0x76` and `0x77` automatically.
- If your board is actually BMP280 (chip ID `0x58`), pressure/temp still work but humidity is not valid.

## What the firmware does

- Continuously reads and validates PMS5003 frames.
- Samples BME/BMP every 5 seconds.
- Refreshes the e-paper at most every 60 seconds when data changes:
  - PM1.0 / PM2.5 / PM10 (ATM)
  - Particle counts (0.3, 0.5um bins)
  - Temperature, humidity, pressure

## Toolchain setup (once)

```bash
cargo +stable install espup --locked
espup install
cargo +stable install espflash --locked
```

## Build

```bash
cargo build
```

## Flash and monitor

Use the upload script for your serial port:

```bash
./upload.sh
```
