# FireBeetle ESP32 + PMS5003 + BME280 + 2.66in e-paper (Rust)

`no_std` ESP32 firmware using:

- `PMS5003` over UART2 for PM readings
- `BME280` over I2C for temperature, humidity, pressure
- `Waveshare 2.66" e-paper` over SPI for on-device display

## Clear Wiring Diagram

FireBeetle ESP32 side:

| FireBeetle pin | Connects to                                | Notes                           |
|:---------------|:-------------------------------------------|:--------------------------------|
| `5V`           | PMS5003 `VCC`                              | PMS5003 power (5V)              |
| `GND`          | PMS5003 `GND`, BME280 `GND`, e-paper `GND` | Shared ground                   |
| `GPIO16`       | PMS5003 `TXD`                              | UART2 RX                        |
| `GPIO17`       | PMS5003 `RXD`                              | UART2 TX (wake/active commands) |
| `3V3`          | BME280 `VIN/VCC`, e-paper `VCC`            | 3.3V rail                       |
| `GPIO21`       | BME280 `SDA`                               | I2C SDA                         |
| `GPIO22`       | BME280 `SCL`                               | I2C SCL                         |
| `GPIO23`       | e-paper `DIN`                              | SPI MOSI                        |
| `GPIO18`       | e-paper `CLK`                              | SPI SCK                         |
| `GPIO5`        | e-paper `CS`                               | SPI chip select                 |
| `GPIO27`       | e-paper `DC`                               | Data/command                    |
| `GPIO26`       | e-paper `RST`                              | Display reset                   |
| `GPIO25`       | e-paper `BUSY`                             | Display busy input              |

### PMS5003 8-pin connector mapping (sensor-side)

- `PIN1 VCC` -> FireBeetle `5V`
- `PIN2 GND` -> FireBeetle `GND`
- `PIN4 RX` -> FireBeetle `GPIO17` (optional but recommended)
- `PIN5 TX` -> FireBeetle `GPIO16`
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

Your shell already sources `export-esp.sh` from `.zshrc`.

## Build

```bash
cd /Users/patrik/projects/firebeetle-pms5003
cargo build
```

## Flash and monitor

Use the upload script for your serial port:

```bash
cd /Users/patrik/projects/firebeetle-pms5003
./upload.sh
```
