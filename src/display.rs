use core::fmt::Write as _;

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
};
use heapless::String;

mod gauge;
mod layout;
mod text;
use gauge::{draw_gauge_scale, update_status_if_changed};
use layout::{
    climate_value_field, FieldCfg, LabelCfg, CLIMATE_HUM_VALUE_CLEAR_W,
    CLIMATE_PRESSURE_VALUE_CLEAR_W, CLIMATE_TEMP_VALUE_CLEAR_W, FIELD_PM03, FIELD_PM05, FIELD_PM1,
    FIELD_PM10, FIELD_PM25, LABEL_PM03, LABEL_PM05, LABEL_PM1, LABEL_PM10, LABEL_PM25,
    LABEL_PRESSURE, LABEL_RH, LABEL_TEMP,
};
use text::{centered_status_text_pos, draw_text_aa, font_height, status_text_clear_rect};

use crate::{air_quality::level_text_sv, bme280::BmeReading, pms5003::Pms5003Reading};

pub type DisplayColor = Rgb565;

// ===== Appearance Config (edit here) =====
// Colors
const BG_COLOR: DisplayColor = DisplayColor::new(0, 0, 0);
const GRID_COLOR: DisplayColor = DisplayColor::new(24, 40, 24);
const TEXT_WHITE: DisplayColor = DisplayColor::new(24, 40, 24);
const NEON_GREEN: DisplayColor = DisplayColor::new(0, 63, 12);
const BLUE: DisplayColor = DisplayColor::new(10, 34, 31);
const YELLOW: DisplayColor = DisplayColor::new(31, 58, 0);

// Fonts
#[allow(dead_code)]
#[derive(Copy, Clone)]
enum FontToken {
    Small,
    Medium,
    Large,
    Larger,
}

#[derive(Copy, Clone)]
struct TextStyleCfg {
    font: FontToken,
    color: DisplayColor,
}

// Typography by function
const STYLE_CLIMATE_LABEL: TextStyleCfg = TextStyleCfg {
    font: FontToken::Medium,
    color: TEXT_WHITE,
};
const STYLE_CLIMATE_TEMP_VALUE: TextStyleCfg = TextStyleCfg {
    font: FontToken::Large,
    color: NEON_GREEN,
};
const STYLE_CLIMATE_HUM_VALUE: TextStyleCfg = TextStyleCfg {
    font: FontToken::Large,
    color: BLUE,
};
const STYLE_CLIMATE_PRESSURE_VALUE: TextStyleCfg = TextStyleCfg {
    font: FontToken::Large,
    color: YELLOW,
};
const STYLE_PARTICLE_LABEL: TextStyleCfg = TextStyleCfg {
    font: FontToken::Medium,
    color: TEXT_WHITE,
};
const STYLE_PARTICLE_VALUE: TextStyleCfg = TextStyleCfg {
    font: FontToken::Larger,
    color: NEON_GREEN,
};
// ===== End appearance config =====

pub struct DisplayCache {
    static_layout_drawn: bool,
    temp: String<16>,
    hum: String<16>,
    pressure: String<16>,
    status_text: String<26>,
    pm25: String<8>,
    pm1: String<8>,
    pm10: String<8>,
    pm03: String<8>,
    pm05: String<8>,
    status_pm25: Option<u16>,
}

impl DisplayCache {
    pub fn new() -> Self {
        Self {
            static_layout_drawn: false,
            temp: String::new(),
            hum: String::new(),
            pressure: String::new(),
            status_text: String::new(),
            pm25: String::new(),
            pm1: String::new(),
            pm10: String::new(),
            pm03: String::new(),
            pm05: String::new(),
            status_pm25: None,
        }
    }
}

pub fn clear_tft<D>(display: &mut D)
where
    D: DrawTarget<Color = DisplayColor>,
{
    clear_rect(
        display,
        Rectangle::new(Point::new(0, 0), display.bounding_box().size),
    );
}

pub fn render_tft<D>(
    display: &mut D,
    cache: &mut DisplayCache,
    pms: Option<Pms5003Reading>,
    aqi_pm: Option<u16>,
    bme: Option<BmeReading>,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    if !cache.static_layout_drawn {
        draw_static_layout(display);
        cache.static_layout_drawn = true;
    }

    draw_dynamic(display, cache, pms, aqi_pm, bme);
}

fn draw_static_layout<D>(display: &mut D)
where
    D: DrawTarget<Color = DisplayColor>,
{
    clear_rect(
        display,
        Rectangle::new(Point::new(0, 0), display.bounding_box().size),
    );

    let line = PrimitiveStyle::with_stroke(GRID_COLOR, 1);
    let _ = Line::new(Point::new(8, 192), Point::new(232, 192))
        .into_styled(line)
        .draw(display);
    let _ = Line::new(Point::new(8, 256), Point::new(232, 256))
        .into_styled(line)
        .draw(display);
    let _ = Line::new(Point::new(80, 192), Point::new(80, 256))
        .into_styled(line)
        .draw(display);
    let _ = Line::new(Point::new(160, 192), Point::new(160, 256))
        .into_styled(line)
        .draw(display);
    let _ = Line::new(Point::new(120, 256), Point::new(120, 318))
        .into_styled(line)
        .draw(display);

    draw_gauge_scale(display);

    draw_label(display, LABEL_TEMP);
    draw_label(display, LABEL_RH);
    draw_label(display, LABEL_PRESSURE);
    draw_label(display, LABEL_PM1);
    draw_label(display, LABEL_PM25);
    draw_label(display, LABEL_PM10);
    draw_label(display, LABEL_PM03);
    draw_label(display, LABEL_PM05);
}

fn draw_dynamic<D>(
    display: &mut D,
    cache: &mut DisplayCache,
    pms: Option<Pms5003Reading>,
    aqi_pm: Option<u16>,
    bme: Option<BmeReading>,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    let mut temp_text: String<16> = String::new();
    let mut hum_text: String<16> = String::new();
    let mut pressure_text: String<16> = String::new();

    if let Some(reading) = bme {
        let sign = if reading.temperature_c_x10 < 0 {
            "-"
        } else {
            ""
        };
        let temp_abs = reading.temperature_c_x10.unsigned_abs();
        let pressure_hpa_x10 = reading.pressure_pa / 10;
        let humidity_pct = (reading.humidity_pct_x10 + 5) / 10;

        let _ = write!(
            temp_text,
            "{}{}.{:01} C",
            sign,
            temp_abs / 10,
            temp_abs % 10
        );
        let _ = write!(hum_text, "{} %", humidity_pct);
        let _ = write!(pressure_text, "{} hPa", pressure_hpa_x10 / 10);
    } else {
        let _ = temp_text.push_str("--.- C");
        let _ = hum_text.push_str("-- %");
        let _ = pressure_text.push_str("---- hPa");
    }

    let mut pm25_text: String<8> = String::new();
    let mut pm1_text: String<8> = String::new();
    let mut pm10_text: String<8> = String::new();
    let mut pm03_text: String<8> = String::new();
    let mut pm05_text: String<8> = String::new();
    if let Some(reading) = pms {
        let _ = write!(pm25_text, "{}", reading.pm2_5_atm);
        let _ = write!(pm1_text, "{}", reading.pm1_0_atm);
        let _ = write!(pm10_text, "{}", reading.pm10_atm);
        let _ = write!(pm03_text, "{}", reading.particles_0_3um);
        let _ = write!(pm05_text, "{}", reading.particles_0_5um);
    } else {
        let _ = pm25_text.push_str("---");
        let _ = pm1_text.push_str("---");
        let _ = pm10_text.push_str("---");
        let _ = pm03_text.push_str("---");
        let _ = pm05_text.push_str("---");
    }

    let field_temp = climate_value_field(
        LABEL_TEMP.x,
        CLIMATE_TEMP_VALUE_CLEAR_W,
        STYLE_CLIMATE_TEMP_VALUE,
    );
    let field_pressure = climate_value_field(
        LABEL_PRESSURE.x,
        CLIMATE_PRESSURE_VALUE_CLEAR_W,
        STYLE_CLIMATE_PRESSURE_VALUE,
    );
    let field_hum = climate_value_field(
        LABEL_RH.x,
        CLIMATE_HUM_VALUE_CLEAR_W,
        STYLE_CLIMATE_HUM_VALUE,
    );

    update_field_if_changed(display, &mut cache.temp, temp_text.as_str(), field_temp);
    update_field_if_changed(display, &mut cache.hum, hum_text.as_str(), field_hum);
    update_field_if_changed(
        display,
        &mut cache.pressure,
        pressure_text.as_str(),
        field_pressure,
    );
    let status_text = level_text_sv(aqi_pm);
    update_status_if_changed(display, cache, status_text, aqi_pm);

    update_field_if_changed(display, &mut cache.pm1, pm1_text.as_str(), FIELD_PM1);
    update_field_if_changed(display, &mut cache.pm25, pm25_text.as_str(), FIELD_PM25);
    update_field_if_changed(display, &mut cache.pm10, pm10_text.as_str(), FIELD_PM10);
    update_field_if_changed(display, &mut cache.pm03, pm03_text.as_str(), FIELD_PM03);
    update_field_if_changed(display, &mut cache.pm05, pm05_text.as_str(), FIELD_PM05);
}

fn draw_label<D>(display: &mut D, label: LabelCfg)
where
    D: DrawTarget<Color = DisplayColor>,
{
    draw_text_aa(
        display,
        Point::new(label.x, label.y),
        label.style.font,
        label.style.color,
        label.text,
    );
}

fn update_field_if_changed<D, const N: usize>(
    display: &mut D,
    previous: &mut String<N>,
    current: &str,
    field: FieldCfg,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    if previous.as_str() == current {
        return;
    }

    clear_rect(
        display,
        expand_rect(
            Rectangle::new(
                Point::new(field.clear_x, field.clear_y),
                Size::new(field.clear_w, field.clear_h),
            ),
            0,
        ),
    );

    draw_text_aa(
        display,
        Point::new(field.x, field.y),
        field.style.font,
        field.style.color,
        current,
    );

    previous.clear();
    let _ = previous.push_str(current);
}

fn clear_rect<D>(display: &mut D, area: Rectangle)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let _ = area
        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
        .draw(display);
}

fn expand_rect(area: Rectangle, pad: i32) -> Rectangle {
    let p = pad.max(0) as u32;
    Rectangle::new(
        Point::new(area.top_left.x - pad, area.top_left.y - pad),
        Size::new(
            area.size.width.saturating_add(p * 2),
            area.size.height.saturating_add(p * 2),
        ),
    )
}
