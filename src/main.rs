#![no_std]
#![no_main]

mod bme280;
mod display;
mod pms5003;

use ::bme280::i2c::BME280;
use embedded_hal_bus::spi::ExclusiveDevice;
use epd_waveshare::prelude::WaveshareDisplay;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    i2c::master::{Config as I2cConfig, I2c},
    main,
    spi::Mode as HalSpiMode,
    spi::master::{Config as SpiConfig, Spi},
    time::{Duration, Instant, Rate},
    uart::{Config as UartConfig, RxError, Uart},
};
use esp_println::println;
use panic_halt as _;

use crate::{
    bme280::{BmeReading, detect_bme_address},
    display::{Epd2in66b, init_display_buffer, refresh_epaper},
    pms5003::{PMS_ACTIVE_MODE_CMD, PMS_WAKE_CMD, PmsParser, write_all},
};

esp_bootloader_esp_idf::esp_app_desc!();

const BME_MEASURE_INTERVAL: Duration = Duration::from_secs(5);
const DISPLAY_REFRESH_INTERVAL: Duration = Duration::from_secs(60);

fn send_pms_command(
    uart: &mut Uart<'_, esp_hal::Blocking>,
    delay: &mut Delay,
    command: &[u8],
    command_name: &str,
) -> bool {
    for attempt in 1..=3 {
        match write_all(uart, command) {
            Ok(()) => {
                println!(
                    "PMS command sent: {} (attempt {}/3)",
                    command_name, attempt
                );
                return true;
            }
            Err(_) => {
                println!(
                    "PMS command failed: {} (attempt {}/3)",
                    command_name, attempt
                );
                delay.delay_millis(100);
            }
        }
    }
    false
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz);
    let peripherals = esp_hal::init(config);

    let mut delay = Delay::new();

    let uart_config = UartConfig::default().with_baudrate(9_600);
    let mut pms_uart = Uart::new(peripherals.UART2, uart_config)
        .expect("Failed to initialize UART2")
        .with_rx(peripherals.GPIO16)
        .with_tx(peripherals.GPIO17);

    println!("PMS5003 reader started");
    println!("PMS UART: UART2 RX=GPIO16 TX=GPIO17 @9600");

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
        .with_sda(peripherals.GPIO21)
        .with_scl(peripherals.GPIO22);

    let (bme_address, sensor_label) = match detect_bme_address(&mut i2c) {
        Some((address, chip_id)) => {
            println!(
                "BME/BMP sensor detected at 0x{:02X}, chip id 0x{:02X}",
                address, chip_id
            );
            let label = if chip_id == 0x60 {
                "BME280 active"
            } else {
                "BMP280 active (no humidity)"
            };
            (Some(address), Some(label))
        }
        None => {
            println!("No BME280/BMP280 detected on 0x76/0x77");
            (None, Some("No BME280/BMP280 detected"))
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
        .with_frequency(Rate::from_mhz(4))
        .with_mode(HalSpiMode::_0);
    let spi_bus = Spi::new(peripherals.SPI2, spi_config)
        .expect("Failed to initialize SPI2")
        .with_sck(peripherals.GPIO18)
        .with_mosi(peripherals.GPIO23);

    let epd_cs = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());
    let epd_dc = Output::new(peripherals.GPIO27, Level::Low, OutputConfig::default());
    let epd_rst = Output::new(peripherals.GPIO26, Level::High, OutputConfig::default());
    let epd_busy = Input::new(peripherals.GPIO25, InputConfig::default());

    let mut epd_spi =
        ExclusiveDevice::new_no_delay(spi_bus, epd_cs).expect("Failed to create SPI device");

    let mut epd = match Epd2in66b::new(&mut epd_spi, epd_busy, epd_dc, epd_rst, &mut delay, None)
    {
        Ok(epd) => {
            println!("2.66in e-paper initialized");
            Some(epd)
        }
        Err(_) => {
            println!("2.66in e-paper init failed");
            None
        }
    };

    let display = init_display_buffer();

    let mut initial_display_refresh_ok = false;
    if let Some(epd) = epd.as_mut() {
        if refresh_epaper(
            epd,
            &mut epd_spi,
            &mut delay,
            display,
            None,
            latest_bme,
            sensor_label,
        )
        .is_err()
        {
            println!("Initial e-paper refresh failed");
        } else {
            initial_display_refresh_ok = true;
        }
        if epd.sleep(&mut epd_spi, &mut delay).is_err() {
            println!("E-paper sleep failed");
        }
    }

    println!("Waiting for valid PMS5003 frames...");

    let mut rx_buf = [0u8; 64];
    let mut pms_parser = PmsParser::new();
    let mut no_data_polls = 0u16;

    let mut latest_pms = None;
    let mut last_bme_measure = Instant::now() - BME_MEASURE_INTERVAL;
    let mut last_display_refresh = if initial_display_refresh_ok {
        Instant::now()
    } else {
        Instant::now() - DISPLAY_REFRESH_INTERVAL
    };
    let mut display_dirty = !initial_display_refresh_ok;

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
                    if latest_pms.is_none() {
                        last_display_refresh = Instant::now() - DISPLAY_REFRESH_INTERVAL;
                    }
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
                        if latest_bme.is_none() {
                            last_display_refresh = Instant::now() - DISPLAY_REFRESH_INTERVAL;
                        }
                        latest_bme = Some(reading);
                        display_dirty = true;
                        let temp_sign = if reading.temperature_c_x10 < 0 { '-' } else { ' ' };
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
            if let Some(epd) = epd.as_mut() {
                if epd.wake_up(&mut epd_spi, &mut delay).is_err() {
                    println!("E-paper wake failed");
                } else if refresh_epaper(
                    epd,
                    &mut epd_spi,
                    &mut delay,
                    display,
                    latest_pms,
                    latest_bme,
                    sensor_label,
                )
                .is_err()
                {
                    println!("E-paper refresh failed");
                } else {
                    display_dirty = false;
                }
                if epd.sleep(&mut epd_spi, &mut delay).is_err() {
                    println!("E-paper sleep failed");
                }
            }
        }

        delay.delay_millis(5);
    }
}
