#![no_std]
#![no_main]

use core::cell::RefCell;

mod air_quality;
mod bme280;
mod display;
mod logger;
mod pm_rolling;
mod pms5003;

use ::bme280::i2c::BME280;
use embedded_hal_bus::{i2c::RefCellDevice, spi::ExclusiveDevice};
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
use mipidsi::{
    interface::SpiInterface,
    models::ILI9341Rgb565,
    options::{ColorOrder, Orientation, Rotation},
    Builder,
};
use panic_halt as _;

use crate::{
    air_quality::aqi_pm25_equiv,
    bme280::{detect_bme_address, BmeReading},
    display::{clear_tft, render_tft, DisplayCache},
    pm_rolling::Pm24hRollingAverage,
    pms5003::{send_pms_command, PmsParser, PMS_ACTIVE_MODE_CMD, PMS_WAKE_CMD},
};

esp_bootloader_esp_idf::esp_app_desc!();

// ===== Timing =====
const BME_MEASURE_INTERVAL: Duration = Duration::from_secs(5);
const DISPLAY_REFRESH_INTERVAL: Duration = Duration::from_millis(250);
const DISPLAY_FORCE_REFRESH_INTERVAL: Duration = Duration::from_secs(60);
const PMS_STALE_TIMEOUT: Duration = Duration::from_secs(30);

#[main]
fn main() -> ! {
    // ===== Hardware init =====
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    logger::init(log::LevelFilter::Info);

    let mut delay = Delay::new();

    // ===== Pin map =====
    let pms_rx = peripherals.GPIO2;
    let pms_tx = peripherals.GPIO3;
    let bme_sda = peripherals.GPIO0;
    let bme_scl = peripherals.GPIO1;
    let tft_sck = peripherals.GPIO9;
    let tft_mosi = peripherals.GPIO8;
    let tft_cs = peripherals.GPIO5;
    let tft_rst = peripherals.GPIO6;
    let tft_dc = peripherals.GPIO7;
    let tft_led = peripherals.GPIO10;

    let uart_config = UartConfig::default().with_baudrate(9_600);
    let mut pms_uart = Uart::new(peripherals.UART1, uart_config)
        .expect("Failed to initialize UART1")
        .with_rx(pms_rx)
        .with_tx(pms_tx);

    let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(100));
    let i2c = I2c::new(peripherals.I2C0, i2c_config)
        .expect("Failed to initialize I2C0")
        .with_sda(bme_sda)
        .with_scl(bme_scl);
    let i2c_bus = RefCell::new(i2c);

    let spi_config = SpiConfig::default()
        .with_frequency(Rate::from_mhz(40))
        .with_mode(HalSpiMode::_0);
    let spi_bus = Spi::new(peripherals.SPI2, spi_config)
        .expect("Failed to initialize SPI2")
        .with_sck(tft_sck)
        .with_mosi(tft_mosi);

    let tft_cs = Output::new(tft_cs, Level::High, OutputConfig::default());
    let tft_rst = Output::new(tft_rst, Level::High, OutputConfig::default());
    let tft_dc = Output::new(tft_dc, Level::Low, OutputConfig::default());
    let _tft_led = Output::new(tft_led, Level::High, OutputConfig::default());

    let tft_spi =
        ExclusiveDevice::new_no_delay(spi_bus, tft_cs).expect("Failed to create SPI device");
    let mut tft_buf = [0u8; 2048];
    let tft_di = SpiInterface::new(tft_spi, tft_dc, &mut tft_buf);

    let mut tft = match Builder::new(ILI9341Rgb565, tft_di)
        .reset_pin(tft_rst)
        .display_size(240, 320)
        .orientation(
            Orientation::new()
                .rotate(Rotation::Deg180)
                .flip_horizontal(),
        )
        .color_order(ColorOrder::Bgr)
        .init(&mut delay)
    {
        Ok(display) => {
            log::info!("2.8in TFT initialized (ILI9341)");
            Some(display)
        }
        Err(err) => {
            log::warn!("2.8in TFT init failed: {:?}", err);
            None
        }
    };

    // ===== Sensor startup =====
    log::info!("PMS5003 reader started");
    log::info!("PMS UART: UART1 @9600");

    let wake_ok = send_pms_command(&mut pms_uart, &mut delay, &PMS_WAKE_CMD, "wake");
    delay.delay_millis(1500);
    let active_ok = send_pms_command(
        &mut pms_uart,
        &mut delay,
        &PMS_ACTIVE_MODE_CMD,
        "active mode",
    );
    if !wake_ok || !active_ok {
        log::warn!("Continuing without confirmed PMS mode setup");
    }
    delay.delay_millis(200);

    let mut i2c_probe = RefCellDevice::new(&i2c_bus);
    let bme_address = match detect_bme_address(&mut i2c_probe) {
        Some(address) => {
            log::info!("BME280 detected at 0x{:02X} (chip id 0x60)", address);
            Some(address)
        }
        None => {
            log::warn!("BME280 not detected on 0x76/0x77.");
            None
        }
    };

    let mut bme = bme_address.map(|address| BME280::new(RefCellDevice::new(&i2c_bus), address));

    let mut latest_bme = None;
    if let Some(sensor) = bme.as_mut() {
        if sensor.init(&mut delay).is_ok() {
            if let Ok(measurements) = sensor.measure(&mut delay) {
                latest_bme = Some(BmeReading::from_measurements(&measurements));
            }
        } else {
            log::warn!("BME280 init failed");
        }
    }

    let mut display_cache = DisplayCache::new();
    if let Some(display) = tft.as_mut() {
        clear_tft(display);
        render_tft(display, &mut display_cache, None, None, latest_bme);
    }

    log::info!("Waiting for valid PMS5003 frames...");

    // ===== Loop state =====
    let mut rx_buf = [0u8; 64];
    let mut pms_parser = PmsParser::new();
    let mut no_data_polls = 0u16;

    let mut latest_pms = None;
    let mut latest_aqi_pm = None;
    let mut pm_24h_avg = Pm24hRollingAverage::new();
    let mut last_bme_measure = Instant::now() - BME_MEASURE_INTERVAL;
    let mut last_display_refresh = Instant::now() - DISPLAY_REFRESH_INTERVAL;
    let mut last_full_refresh = Instant::now();
    let mut last_pms_frame: Option<Instant> = None;
    let mut display_dirty = true;

    // ===== Main loop =====
    loop {
        // Poll PMS5003
        match pms_uart.read_buffered(&mut rx_buf) {
            Ok(0) => {
                no_data_polls = no_data_polls.saturating_add(1);
                if no_data_polls >= 1000 {
                    log::warn!("No UART bytes for ~5s. Check PMS TX, GND, and pin mapping.");
                    no_data_polls = 0;
                }
            }
            Ok(count) => {
                no_data_polls = 0;
                if let Some(reading) = pms_parser.process_chunk(&rx_buf[..count]) {
                    let avg = pm_24h_avg.update(
                        reading.pm1_0_atm,
                        reading.pm2_5_atm,
                        reading.pm10_atm,
                        Instant::now(),
                    );
                    latest_pms = Some(reading);
                    last_pms_frame = Some(Instant::now());
                    let aqi_pm = aqi_pm25_equiv(avg.pm2_5, avg.pm10);
                    latest_aqi_pm = Some(aqi_pm);
                    display_dirty = true;
                    log::info!(
                        "ATM ug/m3 (24h glidande): PM1.0={} PM2.5={} PM10={}",
                        avg.pm1_0,
                        avg.pm2_5,
                        avg.pm10
                    );
                    log::info!("AQI PM-underlag (24h PM2.5-ekvivalent): {}", aqi_pm);
                    log::info!(
                        "ATM ug/m3 (rå): PM1.0={} PM2.5={} PM10={}",
                        reading.pm1_0_atm,
                        reading.pm2_5_atm,
                        reading.pm10_atm
                    );
                    log::info!(
                        "CF1 ug/m3: PM1.0={} PM2.5={} PM10={}",
                        reading.pm1_0_cf1,
                        reading.pm2_5_cf1,
                        reading.pm10_cf1
                    );
                }
            }
            Err(err) => {
                no_data_polls = 0;
                match err {
                    RxError::FifoOverflowed => log::warn!("UART RX FIFO overflow"),
                    RxError::GlitchOccurred => log::warn!("UART RX glitch detected"),
                    RxError::FrameFormatViolated => log::warn!("UART RX framing error"),
                    RxError::ParityMismatch => log::warn!("UART RX parity error"),
                    _ => log::warn!("UART RX error"),
                }
            }
        }

        // Poll BME280
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
                        log::info!(
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
                        log::warn!("BME280 measure failed");
                    }
                }
            }
        }

        // Clear stale PMS data if sensor has been silent for too long
        if let Some(t) = last_pms_frame {
            if t.elapsed() >= PMS_STALE_TIMEOUT {
                latest_pms = None;
                latest_aqi_pm = None;
                last_pms_frame = None;
                display_dirty = true;
                log::warn!("PMS5003 silent for 30s — clearing stale readings");
            }
        }

        // Periodic forced field refresh — recovers from silent draw failures without
        // a full screen clear. Resets text caches so all fields are redrawn next render.
        if last_full_refresh.elapsed() >= DISPLAY_FORCE_REFRESH_INTERVAL {
            display_cache.reset_dynamic();
            last_full_refresh = Instant::now();
            display_dirty = true;
        }

        // Refresh display
        if display_dirty && last_display_refresh.elapsed() >= DISPLAY_REFRESH_INTERVAL {
            last_display_refresh = Instant::now();
            if let Some(display) = tft.as_mut() {
                render_tft(
                    display,
                    &mut display_cache,
                    latest_pms,
                    latest_aqi_pm,
                    latest_bme,
                );
                display_dirty = false;
            }
        }

        delay.delay_millis(5);
    }
}
