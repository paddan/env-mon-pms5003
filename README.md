# ESP32-C3 Super Mini + PMS5003 + BME280 + 2.8in TFT (Rust)

`no_std` ESP32-C3 firmware using:

- `PMS5003` over UART1 for PM readings
- `BME280` over I2C for temperature, humidity, pressure
- `2.8" SPI TFT 240x320` (ILI9341-compatible) over SPI for on-device display

## Clear Wiring Diagram

ESP32-C3 Super Mini side:

`Left side:` `5V`, `GND`, `3V3`, `GPIO4`, `GPIO3`, `GPIO2`, `GPIO1`, `GPIO0`  
`Right side:` `GPIO5`, `GPIO6`, `GPIO7`, `GPIO8`, `GPIO9`, `GPIO10`, `GPIO20`, `GPIO21`

| ESP32-C3 pin | Connects to                              | Notes                           |
|:---------------|:---------------------------------------|:--------------------------------|
| `5V`           | PMS5003 `VCC`                          | PMS5003 power (5V)              |
| `GND`          | PMS5003 `GND`, BME280 `GND`, TFT `GND` | Shared ground                   |
| `GPIO2`        | PMS5003 `TXD`                          | UART1 RX                        |
| `GPIO3`        | PMS5003 `RXD`                          | UART1 TX (wake/active commands) |
| `3V3`          | BME280 `VIN/VCC`, TFT `VCC`            | 3.3V rail                       |
| `GPIO0`        | BME280 `SDA`                           | I2C SDA                         |
| `GPIO1`        | BME280 `SCL`                           | I2C SCL                         |
| `GPIO5`        | TFT `CS`                               | SPI chip select                 |
| `GPIO6`        | TFT `RESET`                            | Display reset                   |
| `GPIO7`        | TFT `DC`                               | Data/command                    |
| `GPIO8`        | TFT `SDI(MOSI)`                        | SPI MOSI                        |
| `GPIO9`        | TFT `SCK`                              | SPI SCK                         |
| `GPIO10`       | TFT `LED`                              | Backlight enable (active high)  |

### TFT connector order (module side)

Given your module pin order:

- `CS`
- `RESET`
- `DC`
- `SDI (MOSI)`
- `SCK`
- `LED`
- `SDO (MISO)`

Use this mapping:

- `CS` -> ESP `GPIO5`
- `RESET` -> ESP `GPIO6`
- `DC` -> ESP `GPIO7`
- `SDI (MOSI)` -> ESP `GPIO8`
- `SCK` -> ESP `GPIO9`
- `LED` -> ESP `GPIO10`
- `SDO (MISO)` -> not connected (not used by firmware)

### TFT module pins not used by firmware

- `SDO(MISO)` (display readback)
- `T_CLK`, `T_CS`, `T_DIN`, `T_DO`, `T_IRQ` (touch controller)
- SD-card pins (`SD_CS`, `SD_MOSI`, `SD_MISO`, `SD_SCK`)

### PMS5003 8-pin connector mapping (sensor-side)

- `PIN1 VCC` -> ESP32-C3 Super Mini `5V`
- `PIN2 GND` -> ESP32-C3 Super Mini `GND`
- `PIN4 RX` -> ESP32-C3 Super Mini `GPIO3` (optional but recommended)
- `PIN5 TX` -> ESP32-C3 Super Mini `GPIO2`
- `PIN3 SET`, `PIN6 RESET`, `PIN7/8 NC` -> leave unconnected

If your PMS cable has no markings:

- Use your known reference first: `PIN1 = 5V`, `PIN2 = GND`.
- Then count pins in the same direction from `PIN1/PIN2`:
  - `PIN3 = SET` (leave unconnected)
  - `PIN4 = RX` (connect to ESP `GPIO3`)
  - `PIN5 = TX` (connect to ESP `GPIO2`)
  - `PIN6 = RESET` (leave unconnected)
  - `PIN7 = NC` (leave unconnected)
  - `PIN8 = NC` (leave unconnected)

### BME280 notes

- Firmware probes both `0x76` and `0x77` automatically.
- If your board is actually BMP280 (chip ID `0x58`), pressure/temp still work but humidity is not valid.

## What the firmware does

- Continuously reads and validates PMS5003 frames.
- Samples BME/BMP every 5 seconds.
- Redraws the TFT at most every 2 seconds when data changes:
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
