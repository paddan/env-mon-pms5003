use core::fmt::Write as _;

use embedded_graphics::{
    mono_font::{
        MonoTextStyleBuilder,
        ascii::{FONT_6X10, FONT_7X13_BOLD},
    },
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use embedded_hal::{digital::InputPin, digital::OutputPin, spi::SpiDevice};
pub use epd_waveshare::epd2in66b::{Display2in66b, Epd2in66b};
use epd_waveshare::prelude::{DisplayRotation, TriColor, WaveshareDisplay};
use esp_hal::delay::Delay;
use heapless::String;
use static_cell::StaticCell;

use crate::{bme280::BmeReading, pms5003::Pms5003Reading};

static DISPLAY: StaticCell<Display2in66b> = StaticCell::new();

pub fn init_display_buffer() -> &'static mut Display2in66b {
    let display = DISPLAY.init(Display2in66b::default());
    display.set_rotation(DisplayRotation::Rotate270);
    display
}

pub fn refresh_epaper<SPI, BUSY, DC, RST>(
    epd: &mut Epd2in66b<SPI, BUSY, DC, RST, Delay>,
    spi: &mut SPI,
    delay: &mut Delay,
    display: &mut Display2in66b,
    pms: Option<Pms5003Reading>,
    bme: Option<BmeReading>,
    sensor_label: Option<&str>,
) -> Result<(), SPI::Error>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    draw_display(display, pms, bme, sensor_label);
    epd.update_frame(spi, display.bw_buffer(), delay)?;
    epd.display_frame(spi, delay)
}

fn draw_display(
    display: &mut Display2in66b,
    pms: Option<Pms5003Reading>,
    bme: Option<BmeReading>,
    sensor_label: Option<&str>,
) {
    let clear = PrimitiveStyle::with_fill(TriColor::White);
    let _ = Rectangle::new(Point::new(0, 0), display.bounding_box().size)
        .into_styled(clear)
        .draw(display);

    let mut pm1_text: String<6> = String::new();
    let mut pm25_text: String<6> = String::new();
    let mut pm10_text: String<6> = String::new();
    let mut pm03_text: String<8> = String::new();
    let mut pm05_text: String<8> = String::new();
    if let Some(reading) = pms {
        let _ = write!(pm1_text, "{}", reading.pm1_0_atm);
        let _ = write!(pm25_text, "{}", reading.pm2_5_atm);
        let _ = write!(pm10_text, "{}", reading.pm10_atm);
        let _ = write!(pm03_text, "{}", reading.particles_0_3um);
        let _ = write!(pm05_text, "{}", reading.particles_0_5um);
    } else {
        let _ = pm1_text.push_str("--");
        let _ = pm25_text.push_str("--");
        let _ = pm10_text.push_str("--");
        let _ = pm03_text.push_str("--");
        let _ = pm05_text.push_str("--");
    }

    let mut temp_text: String<12> = String::new();
    let mut hum_text: String<12> = String::new();
    let mut pressure_text: String<14> = String::new();
    if let Some(reading) = bme {
        let temp_sign = if reading.temperature_c_x10 < 0 { '-' } else { ' ' };
        let temp_abs = reading.temperature_c_x10.unsigned_abs();
        let pressure_hpa_x10 = reading.pressure_pa / 10;
        let _ = write!(temp_text, "{}{}.{:01} C", temp_sign, temp_abs / 10, temp_abs % 10);
        let _ = write!(
            hum_text,
            "{}.{:01} %",
            reading.humidity_pct_x10 / 10,
            reading.humidity_pct_x10 % 10
        );
        let _ = write!(
            pressure_text,
            "{}.{:01} hPa",
            pressure_hpa_x10 / 10,
            pressure_hpa_x10 % 10
        );
    } else {
        let _ = temp_text.push_str("--.- C");
        let _ = hum_text.push_str("--.- %");
        let _ = pressure_text.push_str("----.- hPa");
    }

    draw_line_big(
        display,
        4,
        20,
        TriColor::Black,
        format_args!("{:>5}: {}", "PM0.3", pm03_text),
    );
    draw_line_big(
        display,
        4,
        38,
        TriColor::Black,
        format_args!("{:>5}: {}", "PM0.5", pm05_text),
    );
    draw_line_big(
        display,
        4,
        56,
        TriColor::Black,
        format_args!("{:>5}: {}", "PM1", pm1_text),
    );
    draw_line_big(
        display,
        4,
        74,
        TriColor::Black,
        format_args!("{:>5}: {}", "PM2.5", pm25_text),
    );
    draw_line_big(
        display,
        4,
        92,
        TriColor::Black,
        format_args!("{:>5}: {}", "PM10", pm10_text),
    );

    draw_line_big(
        display,
        96,
        16,
        TriColor::Black,
        format_args!("{:>13}: {}", "Temperatur", temp_text),
    );
    draw_line_big(
        display,
        96,
        38,
        TriColor::Black,
        format_args!("{:>13}: {}", "Luftfuktighet", hum_text),
    );
    draw_line_big(
        display,
        96,
        56,
        TriColor::Black,
        format_args!("{:>13}: {}", "Lufttryck", pressure_text),
    );
    if let Some(sensor_label) = sensor_label {
        draw_line_small(
            display,
            4,
            136,
            TriColor::Black,
            format_args!("{}", sensor_label),
        );
    }
}

fn draw_line_big(
    display: &mut Display2in66b,
    x: i32,
    y: i32,
    color: TriColor,
    args: core::fmt::Arguments<'_>,
) {
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_7X13_BOLD)
        .text_color(color)
        .build();

    let mut line: String<64> = String::new();
    let _ = line.write_fmt(args);
    let _ = Text::new(line.as_str(), Point::new(x, y), text_style).draw(display);
}

fn draw_line_small(
    display: &mut Display2in66b,
    x: i32,
    y: i32,
    color: TriColor,
    args: core::fmt::Arguments<'_>,
) {
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(color)
        .build();

    let mut line: String<96> = String::new();
    let _ = line.write_fmt(args);
    let _ = Text::new(line.as_str(), Point::new(x, y), text_style).draw(display);
}
