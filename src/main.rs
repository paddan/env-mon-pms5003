#![no_std]
#![no_main]

mod air_quality;
mod bme280;
mod display;
mod pms5003;

use ::bme280::i2c::BME280;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    i2c::master::{Config as I2cConfig, I2c},
    main,
    spi::master::{Config as SpiConfig, Spi},
    spi::Mode as HalSpiMode,
    time::{Duration, Instant, Rate},
    uart::{Config as UartConfig, RxError, Uart},
};
use esp_println::println;
use mipidsi::{
    interface::SpiInterface,
    models::ILI9341Rgb565,
    options::{ColorOrder, Orientation},
    Builder,
};
use panic_halt as _;

use crate::{
    bme280::{detect_bme_address, BmeReading},
    display::{clear_tft, render_tft, DisplayCache},
    pms5003::{send_pms_command, PmsParser, PMS_ACTIVE_MODE_CMD, PMS_WAKE_CMD},
};

esp_bootloader_esp_idf::esp_app_desc!();

const BME_MEASURE_INTERVAL: Duration = Duration::from_secs(5);
const DISPLAY_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz);
    let peripherals = esp_hal::init(config);

    let mut delay = Delay::new();

    let uart_config = UartConfig::default().with_baudrate(9_600);
    let mut pms_uart = Uart::new(peripherals.UART1, uart_config)
        .expect("Failed to initialize UART1")
        .with_rx(peripherals.GPIO4)
        .with_tx(peripherals.GPIO5);

    println!("PMS5003 reader started");
    println!("PMS UART: UART1 RX=GPIO4 TX=GPIO5 @9600");

    let wake_ok = send_pms_command(&mut pms_uart, &mut delay, &PMS_WAKE_CMD, "wake");
    delay.delay_millis(1500);
    let active_ok = send_pms_command(
        &mut pms_uart,
        &mut delay,
        &PMS_ACTIVE_MODE_CMD,
        "active mode",
    );
    if !wake_ok || !active_ok {
        println!("Continuing without confirmed PMS mode setup");
    }
    delay.delay_millis(200);

    let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(100));
    let mut i2c = I2c::new(peripherals.I2C0, i2c_config)
        .expect("Failed to initialize I2C0")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);

    let bme_address = match detect_bme_address(&mut i2c) {
        Some((address, chip_id)) => {
            println!(
                "BME/BMP sensor detected at 0x{:02X}, chip id 0x{:02X}",
                address, chip_id
            );
            Some(address)
        }
        None => {
            println!("No BME280/BMP280 detected on 0x76/0x77");
            None
        }
    };

    let mut bme = bme_address.map(|address| BME280::new(i2c, address));

    let mut latest_bme = None;
    if let Some(sensor) = bme.as_mut() {
        if sensor.init(&mut delay).is_ok() {
            if let Ok(measurements) = sensor.measure(&mut delay) {
                latest_bme = Some(BmeReading::from_measurements(&measurements));
            }
        } else {
            println!("BME/BMP init failed");
        }
    }

    let spi_config = SpiConfig::default()
        .with_frequency(Rate::from_mhz(8))
        .with_mode(HalSpiMode::_0);
    let spi_bus = Spi::new(peripherals.SPI2, spi_config)
        .expect("Failed to initialize SPI2")
        .with_sck(peripherals.GPIO8)
        .with_mosi(peripherals.GPIO10);

    let tft_cs = Output::new(peripherals.GPIO3, Level::High, OutputConfig::default());
    let tft_dc = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let tft_rst = Output::new(peripherals.GPIO1, Level::High, OutputConfig::default());
    let _tft_led = Output::new(peripherals.GPIO0, Level::High, OutputConfig::default());

    let tft_spi =
        ExclusiveDevice::new_no_delay(spi_bus, tft_cs).expect("Failed to create SPI device");
    let mut tft_buf = [0u8; 512];
    let tft_di = SpiInterface::new(tft_spi, tft_dc, &mut tft_buf);

    let mut tft = match Builder::new(ILI9341Rgb565, tft_di)
        .reset_pin(tft_rst)
        .display_size(240, 320)
        .orientation(Orientation::new().flip_horizontal())
        .color_order(ColorOrder::Bgr)
        .init(&mut delay)
    {
        Ok(display) => {
            println!("2.8in TFT initialized (ILI9341)");
            Some(display)
        }
        Err(err) => {
            println!("2.8in TFT init failed: {:?}", err);
            None
        }
    };
    let mut display_cache = DisplayCache::new();

    if let Some(display) = tft.as_mut() {
        clear_tft(display);
        render_tft(display, &mut display_cache, None, latest_bme);
    }

    println!("Waiting for valid PMS5003 frames...");

    let mut rx_buf = [0u8; 64];
    let mut pms_parser = PmsParser::new();
    let mut no_data_polls = 0u16;

    let mut latest_pms = None;
    let mut last_bme_measure = Instant::now() - BME_MEASURE_INTERVAL;
    let mut last_display_refresh = Instant::now() - DISPLAY_REFRESH_INTERVAL;
    let mut display_dirty = true;

    loop {
        match pms_uart.read_buffered(&mut rx_buf) {
            Ok(0) => {
                no_data_polls = no_data_polls.saturating_add(1);
                if no_data_polls >= 1000 {
                    println!("No UART bytes for ~5s. Check PMS TX, GND, and pin mapping.");
                    no_data_polls = 0;
                }
            }
            Ok(count) => {
                no_data_polls = 0;
                if let Some(reading) = pms_parser.process_chunk(&rx_buf[..count]) {
                    latest_pms = Some(reading);
                    display_dirty = true;
                    println!(
                        "ATM ug/m3: PM1.0={} PM2.5={} PM10={}",
                        reading.pm1_0_atm, reading.pm2_5_atm, reading.pm10_atm
                    );
                    println!(
                        "CF1 ug/m3: PM1.0={} PM2.5={} PM10={}",
                        reading.pm1_0_cf1, reading.pm2_5_cf1, reading.pm10_cf1
                    );
                }
            }
            Err(err) => {
                no_data_polls = 0;
                match err {
                    RxError::FifoOverflowed => println!("UART RX FIFO overflow"),
                    RxError::GlitchOccurred => println!("UART RX glitch detected"),
                    RxError::FrameFormatViolated => println!("UART RX framing error"),
                    RxError::ParityMismatch => println!("UART RX parity error"),
                    _ => println!("UART RX error"),
                }
            }
        }

        if last_bme_measure.elapsed() >= BME_MEASURE_INTERVAL {
            last_bme_measure = Instant::now();
            if let Some(sensor) = bme.as_mut() {
                match sensor.measure(&mut delay) {
                    Ok(measurements) => {
                        let reading = BmeReading::from_measurements(&measurements);
                        latest_bme = Some(reading);
                        display_dirty = true;
                        let temp_sign = if reading.temperature_c_x10 < 0 {
                            "-"
                        } else {
                            ""
                        };
                        let temp_abs = reading.temperature_c_x10.unsigned_abs();
                        println!(
                            "BME: T={}{}.{:01}C RH={}.{:01}% P={}Pa",
                            temp_sign,
                            temp_abs / 10,
                            temp_abs % 10,
                            reading.humidity_pct_x10 / 10,
                            reading.humidity_pct_x10 % 10,
                            reading.pressure_pa
                        );
                    }
                    Err(_) => {
                        println!("BME/BMP measure failed");
                    }
                }
            }
        }

        if display_dirty && last_display_refresh.elapsed() >= DISPLAY_REFRESH_INTERVAL {
            last_display_refresh = Instant::now();
            if let Some(display) = tft.as_mut() {
                render_tft(display, &mut display_cache, latest_pms, latest_bme);
                display_dirty = false;
            }
        }

        delay.delay_millis(5);
    }
}
