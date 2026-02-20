use core::fmt::Write as _;

use embedded_graphics::{
    geometry::AngleUnit,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Arc, Circle, Line, PrimitiveStyle, Rectangle},
};
use heapless::String;
use micromath::F32Ext;

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
use text::{centered_status_text_pos, draw_text_aa, font_for, font_height, status_text_clear_rect};

use crate::{
    air_quality::{band_from_pm25, level_text_sv, ratio_from_pm25, EuAqiBand},
    bme280::BmeReading,
    pms5003::Pms5003Reading,
};

pub type DisplayColor = Rgb565;

// ===== Appearance Config (edit here) =====
// Colors
const BG_COLOR: DisplayColor = DisplayColor::new(0, 0, 0);
const GRID_COLOR: DisplayColor = DisplayColor::new(8, 16, 8);
const TEXT_WHITE: DisplayColor = DisplayColor::new(31, 63, 31);
const TEXT_DIM: DisplayColor = DisplayColor::new(20, 38, 20);
const NEON_GREEN: DisplayColor = DisplayColor::new(0, 63, 12);
const LIME: DisplayColor = DisplayColor::new(12, 63, 0);
const BLUE: DisplayColor = DisplayColor::new(10, 34, 31);
const GREEN: DisplayColor = DisplayColor::new(0, 54, 8);
const YELLOW: DisplayColor = DisplayColor::new(31, 58, 0);
const ORANGE: DisplayColor = DisplayColor::new(31, 34, 0);
const RED: DisplayColor = DisplayColor::new(31, 4, 4);
const DEEP_RED: DisplayColor = DisplayColor::new(23, 0, 0);

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

#[derive(Copy, Clone)]
enum U8g2FontToken {
    Small,
    Medium,
    Large,
    Larger,
}

impl From<FontToken> for U8g2FontToken {
    fn from(value: FontToken) -> Self {
        match value {
            FontToken::Small => U8g2FontToken::Small,
            FontToken::Medium => U8g2FontToken::Medium,
            FontToken::Large => U8g2FontToken::Large,
            FontToken::Larger => U8g2FontToken::Larger,
        }
    }
}

#[derive(Copy, Clone)]
enum ResolvedFont {
    U8g2(U8g2FontToken),
}

// Typography by function
const STYLE_CLIMATE_LABEL: TextStyleCfg = TextStyleCfg {
    font: FontToken::Medium,
    color: TEXT_DIM,
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

const STYLE_STATUS_TEXT: TextStyleCfg = TextStyleCfg {
    font: FontToken::Medium,
    color: TEXT_DIM,
};

const STYLE_PARTICLE_LABEL: TextStyleCfg = TextStyleCfg {
    font: FontToken::Medium,
    color: TEXT_DIM,
};
const STYLE_PARTICLE_VALUE: TextStyleCfg = TextStyleCfg {
    font: FontToken::Larger,
    color: NEON_GREEN,
};

// Gauge geometry and style
const GAUGE_CENTER: Point = Point::new(120, 154);
const GAUGE_DIAMETER: u32 = 190;

// Display orientation on this panel maps positive sweep to the desired upper status arc.
const GAUGE_START_DEG: f32 = 180.0;
const GAUGE_TOTAL_SWEEP_DEG: f32 = 180.0;

const GAUGE_BAND_OUTER_W: u32 = 18;
const GAUGE_BAND_EDGE_W: u32 = 14;
const GAUGE_BAND_FILL_W: u32 = 10;
const GAUGE_BAND_HIGHLIGHT_W: u32 = 6;
const GAUGE_GRADIENT_STEP_DEG_STATIC: f32 = 4.0;
const GAUGE_GRADIENT_STEP_DEG_RESTORE: f32 = 3.0;
const GAUGE_COLOR_BLEND_SPAN_DEG: f32 = 30.0;

const GAUGE_REF_DIAMETER: i32 = 190;
const GAUGE_POINTER_LENGTH_FACTOR: f32 = 1.2;
const GAUGE_POINTER_MAX_EXTRA_R_BASE: i32 = 120;
const GAUGE_RESTORE_SPAN_DEG: f32 = 7.0;
const GAUGE_NEEDLE_INNER_R_BASE: i32 = 2;
const GAUGE_NEEDLE_OUTER_R_BASE: i32 = 70;
const GAUGE_NEEDLE_W_BASE: i32 = 4;
const GAUGE_NEEDLE_SHADOW_W_BASE: i32 = 6;
const GAUGE_NEEDLE_CLEAR_W_BASE: i32 = 8;
const GAUGE_NEEDLE_COLOR: DisplayColor = TEXT_WHITE;
const GAUGE_NEEDLE_SHADOW_COLOR: DisplayColor = DisplayColor::new(5, 12, 5);
const GAUGE_HUB_D_BASE: u32 = 10;
const GAUGE_HUB_CLEAR_D_BASE: u32 = 12;
const GAUGE_HUB_COLOR: DisplayColor = TEXT_WHITE;
const STATUS_TEXT_GAP_Y: i32 = 16;
const STATUS_TEXT_CLEAR_PAD_X: i32 = 2;
const STATUS_TEXT_CLEAR_PAD_Y: i32 = 0;
const STATUS_TEXT_MAX_CHARS: i32 = 15;
const GAUGE_ARROW_LEN_BASE: i32 = 11;
const GAUGE_ARROW_HALF_W_BASE: i32 = 6;
const GAUGE_ARROW_TIP_OFFSET_BASE: i32 = 3;
const GAUGE_ARROW_SHADOW_PAD_BASE: i32 = 1;
const GAUGE_ARROW_CLEAR_PAD_BASE: i32 = 2;
const GAUGE_NEEDLE_FAST_MODE: bool = true;
const GAUGE_NEEDLE_MIN_REDRAW_DEG: f32 = 1.0;

#[derive(Copy, Clone)]
struct GaugeSegmentCfg {
    sweep_deg: f32,
    color: DisplayColor,
}

const GAUGE_SEGMENTS: [GaugeSegmentCfg; 6] = [
    GaugeSegmentCfg {
        sweep_deg: 5.0,
        color: GREEN,
    },
    GaugeSegmentCfg {
        sweep_deg: 10.0,
        color: LIME,
    },
    GaugeSegmentCfg {
        sweep_deg: 35.0,
        color: YELLOW,
    },
    GaugeSegmentCfg {
        sweep_deg: 40.0,
        color: ORANGE,
    },
    GaugeSegmentCfg {
        sweep_deg: 50.0,
        color: RED,
    },
    GaugeSegmentCfg {
        sweep_deg: 40.0,
        color: DEEP_RED,
    },
];
// ===== End appearance config =====

pub struct DisplayCache {
    static_layout_drawn: bool,
    temp: String<16>,
    hum: String<16>,
    pressure: String<16>,
    status_text: String<16>,
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
    bme: Option<BmeReading>,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    if !cache.static_layout_drawn {
        draw_static_layout(display);
        cache.static_layout_drawn = true;
    }

    draw_dynamic(display, cache, pms, bme);
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
    let mut status_pm25 = None;

    if let Some(reading) = pms {
        let _ = write!(pm25_text, "{}", reading.pm2_5_atm);
        let _ = write!(pm1_text, "{}", reading.pm1_0_atm);
        let _ = write!(pm10_text, "{}", reading.pm10_atm);
        let _ = write!(pm03_text, "{}", reading.particles_0_3um);
        let _ = write!(pm05_text, "{}", reading.particles_0_5um);

        status_pm25 = Some(reading.pm2_5_atm);
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
    let status_text = level_text_sv(status_pm25);
    update_status_if_changed(display, cache, status_text, status_pm25);

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
        font_for(label.style.font),
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
        font_for(field.style.font),
        field.style.color,
        current,
    );

    previous.clear();
    let _ = previous.push_str(current);
}

fn draw_pixel_safe<D>(display: &mut D, p: Point, color: DisplayColor)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let bounds = display.bounding_box();
    let right = bounds.top_left.x + bounds.size.width as i32;
    let bottom = bounds.top_left.y + bounds.size.height as i32;

    if p.x < bounds.top_left.x || p.y < bounds.top_left.y || p.x >= right || p.y >= bottom {
        return;
    }

    let _ = Pixel(p, color).draw(display);
}

fn scale_color(color: DisplayColor, alpha: u8) -> DisplayColor {
    let a = alpha as u16;
    DisplayColor::new(
        ((color.r() as u16 * a) / 255) as u8,
        ((color.g() as u16 * a) / 255) as u8,
        ((color.b() as u16 * a) / 255) as u8,
    )
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

fn darken(color: DisplayColor, amount: u8) -> DisplayColor {
    let g_amount = amount.saturating_mul(2);
    DisplayColor::new(
        color.r().saturating_sub(amount),
        color.g().saturating_sub(g_amount),
        color.b().saturating_sub(amount),
    )
}

fn brighten(color: DisplayColor, amount: u8) -> DisplayColor {
    let g_amount = amount.saturating_mul(2);
    DisplayColor::new(
        color.r().saturating_add(amount).min(31),
        color.g().saturating_add(g_amount).min(63),
        color.b().saturating_add(amount).min(31),
    )
}

fn status_color(pm25: u16) -> DisplayColor {
    match band_from_pm25(pm25) {
        EuAqiBand::Good => GREEN,
        EuAqiBand::Fair => LIME,
        EuAqiBand::Moderate => YELLOW,
        EuAqiBand::Poor => ORANGE,
        EuAqiBand::VeryPoor => RED,
        EuAqiBand::ExtremelyPoor => DEEP_RED,
    }
}
