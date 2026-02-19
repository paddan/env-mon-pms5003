use core::fmt::Write as _;

use embedded_graphics::{
    geometry::AngleUnit,
    image::GetPixel,
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10, FONT_8X13_BOLD},
        MonoFont,
    },
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    primitives::{Arc, Circle, Line, PrimitiveStyle, Rectangle, Triangle},
};
use heapless::String;
use micromath::F32Ext;

use crate::{bme280::BmeReading, pms5003::Pms5003Reading};

pub type DisplayColor = Rgb565;

// ===== Appearance Config (edit here) =====
// Colors
const BG_COLOR: DisplayColor = DisplayColor::new(0, 0, 0);
const GRID_COLOR: DisplayColor = DisplayColor::new(8, 16, 8);
const TEXT_WHITE: DisplayColor = DisplayColor::new(31, 63, 31);
const TEXT_DIM: DisplayColor = DisplayColor::new(20, 38, 20);
const NEON_GREEN: DisplayColor = DisplayColor::new(0, 63, 12);
const BLUE: DisplayColor = DisplayColor::new(10, 34, 31);
const GREEN: DisplayColor = DisplayColor::new(0, 54, 8);
const YELLOW: DisplayColor = DisplayColor::new(31, 58, 0);
const ORANGE: DisplayColor = DisplayColor::new(31, 34, 0);
const RED: DisplayColor = DisplayColor::new(31, 4, 4);

// Fonts
#[derive(Copy, Clone)]
enum FontToken {
    Small,
    Medium,
    Large,
    Larger,
    Largest,
}

#[derive(Copy, Clone)]
struct TextStyleCfg {
    font: FontToken,
    color: DisplayColor,
}

// Typography by function
const STYLE_HEADER_TEXT: TextStyleCfg = TextStyleCfg {
    font: FontToken::Small,
    color: TEXT_WHITE,
};

const STYLE_CLIMATE_LABEL: TextStyleCfg = TextStyleCfg {
    font: FontToken::Small,
    color: TEXT_DIM,
};
const STYLE_CLIMATE_TEMP_VALUE: TextStyleCfg = TextStyleCfg {
    font: FontToken::Medium,
    color: NEON_GREEN,
};
const STYLE_CLIMATE_HUM_VALUE: TextStyleCfg = TextStyleCfg {
    font: FontToken::Medium,
    color: BLUE,
};
const STYLE_CLIMATE_PRESSURE_VALUE: TextStyleCfg = TextStyleCfg {
    font: FontToken::Medium,
    color: TEXT_DIM,
};

const STYLE_AQI_STATUS: TextStyleCfg = TextStyleCfg {
    font: FontToken::Medium,
    color: TEXT_DIM,
};

const STYLE_PARTICLE_LABEL: TextStyleCfg = TextStyleCfg {
    font: FontToken::Small,
    color: TEXT_WHITE,
};
const STYLE_PARTICLE_VALUE: TextStyleCfg = TextStyleCfg {
    font: FontToken::Larger,
    color: NEON_GREEN,
};

#[derive(Copy, Clone)]
struct LabelCfg {
    x: i32,
    y: i32,
    text: &'static str,
    style: TextStyleCfg,
}

#[derive(Copy, Clone)]
struct FieldCfg {
    x: i32,
    y: i32,
    clear_x: i32,
    clear_y: i32,
    clear_w: u32,
    clear_h: u32,
    style: TextStyleCfg,
}

// Static labels by function
const LABEL_TEMP: LabelCfg = LabelCfg {
    x: 8,
    y: 2,
    text: "Temp",
    style: STYLE_CLIMATE_LABEL,
};
const LABEL_RH: LabelCfg = LabelCfg {
    x: 198,
    y: 2,
    text: "RH",
    style: STYLE_CLIMATE_LABEL,
};
const LABEL_PRESSURE: LabelCfg = LabelCfg {
    x: 96,
    y: 2,
    text: "Tryck",
    style: STYLE_CLIMATE_LABEL,
};
const LABEL_PM1: LabelCfg = LabelCfg {
    x: 10,
    y: 198,
    text: "PM1.0",
    style: STYLE_PARTICLE_LABEL,
};
const LABEL_PM25: LabelCfg = LabelCfg {
    x: 88,
    y: 198,
    text: "PM2.5",
    style: STYLE_PARTICLE_LABEL,
};
const LABEL_PM10: LabelCfg = LabelCfg {
    x: 168,
    y: 198,
    text: "PM10",
    style: STYLE_PARTICLE_LABEL,
};
const LABEL_PM03: LabelCfg = LabelCfg {
    x: 12,
    y: 262,
    text: "PM0.3",
    style: STYLE_PARTICLE_LABEL,
};
const LABEL_PM05: LabelCfg = LabelCfg {
    x: 132,
    y: 262,
    text: "PM0.5",
    style: STYLE_PARTICLE_LABEL,
};

// Dynamic fields by function
const FIELD_HEADER: FieldCfg = FieldCfg {
    x: 8,
    y: 176,
    clear_x: 8,
    clear_y: 176,
    clear_w: 220,
    clear_h: 12,
    style: STYLE_HEADER_TEXT,
};
const FIELD_TEMP: FieldCfg = FieldCfg {
    x: 8,
    y: 15,
    clear_x: 8,
    clear_y: 15,
    clear_w: 72,
    clear_h: 14,
    style: STYLE_CLIMATE_TEMP_VALUE,
};
const FIELD_HUM: FieldCfg = FieldCfg {
    x: 198,
    y: 15,
    clear_x: 198,
    clear_y: 15,
    clear_w: 40,
    clear_h: 14,
    style: STYLE_CLIMATE_HUM_VALUE,
};
const FIELD_PRESSURE: FieldCfg = FieldCfg {
    x: 96,
    y: 15,
    clear_x: 96,
    clear_y: 15,
    clear_w: 92,
    clear_h: 14,
    style: STYLE_CLIMATE_PRESSURE_VALUE,
};
const FIELD_AQI: FieldCfg = FieldCfg {
    x: 0,
    y: 0,
    clear_x: 0,
    clear_y: 0,
    clear_w: 0,
    clear_h: 0,
    style: STYLE_AQI_STATUS,
};

const FIELD_PM1: FieldCfg = FieldCfg {
    x: 8,
    y: 224,
    clear_x: 8,
    clear_y: 224,
    clear_w: 68,
    clear_h: 22,
    style: STYLE_PARTICLE_VALUE,
};
const FIELD_PM25: FieldCfg = FieldCfg {
    x: 88,
    y: 228,
    clear_x: 88,
    clear_y: 228,
    clear_w: 68,
    clear_h: 22,
    style: STYLE_PARTICLE_VALUE,
};
const FIELD_PM10: FieldCfg = FieldCfg {
    x: 168,
    y: 224,
    clear_x: 168,
    clear_y: 224,
    clear_w: 62,
    clear_h: 22,
    style: STYLE_PARTICLE_VALUE,
};
const FIELD_PM03: FieldCfg = FieldCfg {
    x: 8,
    y: 286,
    clear_x: 8,
    clear_y: 286,
    clear_w: 104,
    clear_h: 22,
    style: STYLE_PARTICLE_VALUE,
};
const FIELD_PM05: FieldCfg = FieldCfg {
    x: 128,
    y: 286,
    clear_x: 128,
    clear_y: 286,
    clear_w: 104,
    clear_h: 22,
    style: STYLE_PARTICLE_VALUE,
};

// Gauge geometry and style
const GAUGE_CENTER: Point = Point::new(120, 154);
const GAUGE_DIAMETER: u32 = 170;

// Display orientation on this panel maps positive sweep to the desired upper AQI arc.
const GAUGE_START_DEG: f32 = 180.0;
const GAUGE_TOTAL_SWEEP_DEG: f32 = 180.0;

const GAUGE_BAND_OUTER_W: u32 = 18;
const GAUGE_BAND_EDGE_W: u32 = 14;
const GAUGE_BAND_FILL_W: u32 = 10;
const GAUGE_BAND_HIGHLIGHT_W: u32 = 6;

const GAUGE_TICK_W: u32 = 8;
const GAUGE_TICK_SPAN_DEG: f32 = 3.0;
const GAUGE_MARKER_W: u32 = 8;
const GAUGE_MARKER_SPAN_DEG: f32 = 5.0;
const GAUGE_NEEDLE_INNER_R: i32 = 8;
const GAUGE_NEEDLE_OUTER_R: i32 = 70;
const GAUGE_NEEDLE_W: u32 = 4;
const GAUGE_NEEDLE_SHADOW_W: u32 = 6;
const GAUGE_NEEDLE_CLEAR_W: u32 = 8;
const GAUGE_NEEDLE_COLOR: DisplayColor = TEXT_WHITE;
const GAUGE_NEEDLE_SHADOW_COLOR: DisplayColor = DisplayColor::new(5, 12, 5);
const GAUGE_HUB_D: u32 = 10;
const GAUGE_HUB_CLEAR_D: u32 = 12;
const GAUGE_HUB_COLOR: DisplayColor = TEXT_WHITE;
const AQI_STATUS_GAP_Y: i32 = 10;
const AQI_STATUS_CLEAR_PAD_X: i32 = 4;
const AQI_STATUS_CLEAR_PAD_Y: i32 = 2;
const AQI_STATUS_MAX_CHARS: i32 = 16;
const GAUGE_ARROW_LEN: i32 = 11;
const GAUGE_ARROW_HALF_W: i32 = 6;
const GAUGE_ARROW_TIP_OFFSET: i32 = 3;
const GAUGE_ARROW_SHADOW_PAD: i32 = 1;
const GAUGE_ARROW_CLEAR_PAD: i32 = 2;

#[derive(Copy, Clone)]
struct GaugeSegmentCfg {
    sweep_deg: f32,
    color: DisplayColor,
}

const GAUGE_SEGMENTS: [GaugeSegmentCfg; 4] = [
    GaugeSegmentCfg {
        sweep_deg: 60.0,
        color: GREEN,
    },
    GaugeSegmentCfg {
        sweep_deg: 45.0,
        color: YELLOW,
    },
    GaugeSegmentCfg {
        sweep_deg: 45.0,
        color: ORANGE,
    },
    GaugeSegmentCfg {
        sweep_deg: 30.0,
        color: RED,
    },
];
// ===== End appearance config =====

pub struct DisplayCache {
    static_layout_drawn: bool,
    header: String<48>,
    temp: String<16>,
    hum: String<16>,
    pressure: String<16>,
    aqi: String<16>,
    pm25: String<8>,
    pm1: String<8>,
    pm10: String<8>,
    pm03: String<8>,
    pm05: String<8>,
    aqi_value: Option<u16>,
}

impl DisplayCache {
    pub fn new() -> Self {
        Self {
            static_layout_drawn: false,
            header: String::new(),
            temp: String::new(),
            hum: String::new(),
            pressure: String::new(),
            aqi: String::new(),
            pm25: String::new(),
            pm1: String::new(),
            pm10: String::new(),
            pm03: String::new(),
            pm05: String::new(),
            aqi_value: None,
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
    sensor_label: Option<&str>,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    if !cache.static_layout_drawn {
        draw_static_layout(display);
        cache.static_layout_drawn = true;
    }

    draw_dynamic(display, cache, pms, bme, sensor_label);
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
    sensor_label: Option<&str>,
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
    let mut aqi_value = None;

    if let Some(reading) = pms {
        let _ = write!(pm25_text, "{:03}", reading.pm2_5_atm);
        let _ = write!(pm1_text, "{:03}", reading.pm1_0_atm);
        let _ = write!(pm10_text, "{:03}", reading.pm10_atm);
        let _ = write!(pm03_text, "{}", reading.particles_0_3um);
        let _ = write!(pm05_text, "{}", reading.particles_0_5um);

        aqi_value = Some(aqi_from_pm25(reading.pm2_5_atm));
    } else {
        let _ = pm25_text.push_str("---");
        let _ = pm1_text.push_str("---");
        let _ = pm10_text.push_str("---");
        let _ = pm03_text.push_str("---");
        let _ = pm05_text.push_str("---");
    }

    let header = sensor_label.unwrap_or("ENV MONITOR");

    update_field_if_changed(display, &mut cache.header, header, FIELD_HEADER);
    update_field_if_changed(display, &mut cache.temp, temp_text.as_str(), FIELD_TEMP);
    update_field_if_changed(display, &mut cache.hum, hum_text.as_str(), FIELD_HUM);
    update_field_if_changed(
        display,
        &mut cache.pressure,
        pressure_text.as_str(),
        FIELD_PRESSURE,
    );
    let aqi_status = aqi_level_text(aqi_value);
    update_aqi_if_changed(display, cache, aqi_status, aqi_value);

    update_field_if_changed(display, &mut cache.pm1, pm1_text.as_str(), FIELD_PM1);
    update_field_if_changed(display, &mut cache.pm25, pm25_text.as_str(), FIELD_PM25);
    update_field_if_changed(display, &mut cache.pm10, pm10_text.as_str(), FIELD_PM10);
    update_field_if_changed(display, &mut cache.pm03, pm03_text.as_str(), FIELD_PM03);
    update_field_if_changed(display, &mut cache.pm05, pm05_text.as_str(), FIELD_PM05);
}

fn draw_gauge_scale<D>(display: &mut D)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let mut angle = GAUGE_START_DEG;

    for (idx, segment) in GAUGE_SEGMENTS.iter().enumerate() {
        let sweep = segment.sweep_deg.copysign(GAUGE_TOTAL_SWEEP_DEG);
        draw_arc_band(display, angle, sweep, segment.color);
        angle += sweep;

        if idx < GAUGE_SEGMENTS.len() - 1 {
            draw_tick(display, angle);
        }
    }
}

fn update_aqi_if_changed<D>(
    display: &mut D,
    cache: &mut DisplayCache,
    text: &str,
    value: Option<u16>,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    if cache.aqi.as_str() == text && cache.aqi_value == value {
        return;
    }

    // Remove previous marker by restoring the underlying gauge highlight color.
    if let Some(prev) = cache.aqi_value {
        let ratio = aqi_ratio(prev);
        let angle = gauge_angle(ratio);
        erase_aqi_needle(display, angle);
        draw_aqi_marker(display, angle, gauge_marker_bg_color(ratio));
    }

    let value_style = TextStyleCfg {
        font: FIELD_AQI.style.font,
        color: value.map(aqi_color).unwrap_or(TEXT_DIM),
    };
    let font = font_for(value_style.font);
    clear_rect(display, aqi_status_clear_rect(font));

    let text_pos = centered_aqi_status_pos(font, text);
    draw_text_aa(display, text_pos, font, value_style.color, text);

    if let Some(aqi) = value {
        let ratio = aqi_ratio(aqi);
        let angle = gauge_angle(ratio);
        draw_aqi_marker(display, angle, TEXT_WHITE);
        draw_aqi_needle(display, angle);
    }

    cache.aqi.clear();
    let _ = cache.aqi.push_str(text);
    cache.aqi_value = value;
}

fn draw_arc_band<D>(display: &mut D, start_deg: f32, sweep_deg: f32, color: DisplayColor)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let outer = darken(color, 8);
    let edge = darken(color, 5);
    let fill = color;
    let highlight = brighten(color, 4);

    let arc = Arc::with_center(
        GAUGE_CENTER,
        GAUGE_DIAMETER,
        start_deg.deg(),
        sweep_deg.deg(),
    );
    let _ = arc
        .into_styled(PrimitiveStyle::with_stroke(outer, GAUGE_BAND_OUTER_W))
        .draw(display);

    let arc = Arc::with_center(
        GAUGE_CENTER,
        GAUGE_DIAMETER,
        start_deg.deg(),
        sweep_deg.deg(),
    );
    let _ = arc
        .into_styled(PrimitiveStyle::with_stroke(edge, GAUGE_BAND_EDGE_W))
        .draw(display);

    let arc = Arc::with_center(
        GAUGE_CENTER,
        GAUGE_DIAMETER,
        start_deg.deg(),
        sweep_deg.deg(),
    );
    let _ = arc
        .into_styled(PrimitiveStyle::with_stroke(fill, GAUGE_BAND_FILL_W))
        .draw(display);

    let arc = Arc::with_center(
        GAUGE_CENTER,
        GAUGE_DIAMETER,
        start_deg.deg(),
        sweep_deg.deg(),
    );
    let _ = arc
        .into_styled(PrimitiveStyle::with_stroke(
            highlight,
            GAUGE_BAND_HIGHLIGHT_W,
        ))
        .draw(display);
}

fn draw_tick<D>(display: &mut D, angle_deg: f32)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let dir = if GAUGE_TOTAL_SWEEP_DEG >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let half = (GAUGE_TICK_SPAN_DEG * 0.5) * dir;

    let _ = Arc::with_center(
        GAUGE_CENTER,
        GAUGE_DIAMETER,
        (angle_deg + half).deg(),
        (-GAUGE_TICK_SPAN_DEG * dir).deg(),
    )
    .into_styled(PrimitiveStyle::with_stroke(TEXT_WHITE, GAUGE_TICK_W))
    .draw(display);
}

fn draw_aqi_marker<D>(display: &mut D, angle_deg: f32, color: DisplayColor)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let dir = if GAUGE_TOTAL_SWEEP_DEG >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let half = (GAUGE_MARKER_SPAN_DEG * 0.5) * dir;

    let _ = Arc::with_center(
        GAUGE_CENTER,
        GAUGE_DIAMETER,
        (angle_deg + half).deg(),
        (-GAUGE_MARKER_SPAN_DEG * dir).deg(),
    )
    .into_styled(PrimitiveStyle::with_stroke(color, GAUGE_MARKER_W))
    .draw(display);
}

fn draw_aqi_needle<D>(display: &mut D, angle_deg: f32)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let (start, shaft_end, tip, left, right) = needle_geometry(
        angle_deg,
        GAUGE_ARROW_LEN,
        GAUGE_ARROW_HALF_W,
        GAUGE_ARROW_TIP_OFFSET,
    );
    let (_, _, shadow_tip, shadow_left, shadow_right) = needle_geometry(
        angle_deg,
        GAUGE_ARROW_LEN + GAUGE_ARROW_SHADOW_PAD,
        GAUGE_ARROW_HALF_W + GAUGE_ARROW_SHADOW_PAD,
        GAUGE_ARROW_TIP_OFFSET + GAUGE_ARROW_SHADOW_PAD,
    );

    let _ = Line::new(start, shaft_end)
        .into_styled(PrimitiveStyle::with_stroke(
            GAUGE_NEEDLE_SHADOW_COLOR,
            GAUGE_NEEDLE_SHADOW_W,
        ))
        .draw(display);

    let _ = Triangle::new(shadow_tip, shadow_left, shadow_right)
        .into_styled(PrimitiveStyle::with_fill(GAUGE_NEEDLE_SHADOW_COLOR))
        .draw(display);

    let _ = Line::new(start, shaft_end)
        .into_styled(PrimitiveStyle::with_stroke(
            GAUGE_NEEDLE_COLOR,
            GAUGE_NEEDLE_W,
        ))
        .draw(display);

    let _ = Triangle::new(tip, left, right)
        .into_styled(PrimitiveStyle::with_fill(GAUGE_NEEDLE_COLOR))
        .draw(display);

    let _ = Circle::with_center(GAUGE_CENTER, GAUGE_HUB_D)
        .into_styled(PrimitiveStyle::with_fill(GAUGE_HUB_COLOR))
        .draw(display);
}

fn erase_aqi_needle<D>(display: &mut D, angle_deg: f32)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let (start, _, tip, left, right) = needle_geometry(
        angle_deg,
        GAUGE_ARROW_LEN + GAUGE_ARROW_CLEAR_PAD,
        GAUGE_ARROW_HALF_W + GAUGE_ARROW_CLEAR_PAD,
        GAUGE_ARROW_TIP_OFFSET + GAUGE_ARROW_CLEAR_PAD,
    );

    let _ = Line::new(start, tip)
        .into_styled(PrimitiveStyle::with_stroke(BG_COLOR, GAUGE_NEEDLE_CLEAR_W))
        .draw(display);

    let _ = Triangle::new(tip, left, right)
        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
        .draw(display);

    let _ = Circle::with_center(GAUGE_CENTER, GAUGE_HUB_CLEAR_D)
        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
        .draw(display);
}

fn aqi_ratio(aqi: u16) -> f32 {
    aqi.min(300) as f32 / 300.0
}

fn gauge_angle(ratio: f32) -> f32 {
    GAUGE_START_DEG + ratio.clamp(0.0, 1.0) * GAUGE_TOTAL_SWEEP_DEG
}

fn polar_point(center: Point, radius: i32, angle_deg: f32) -> Point {
    let rad = angle_deg * (core::f32::consts::PI / 180.0);
    let x = center.x + round_to_i32((radius as f32) * rad.cos());
    let y = center.y + round_to_i32((radius as f32) * rad.sin());
    Point::new(x, y)
}

fn needle_geometry(
    angle_deg: f32,
    arrow_len: i32,
    arrow_half_w: i32,
    arrow_tip_offset: i32,
) -> (Point, Point, Point, Point, Point) {
    let start = polar_point(GAUGE_CENTER, GAUGE_NEEDLE_INNER_R, angle_deg);
    let base = polar_point(GAUGE_CENTER, GAUGE_NEEDLE_OUTER_R - arrow_len, angle_deg);
    let tip = polar_point(
        GAUGE_CENTER,
        GAUGE_NEEDLE_OUTER_R + arrow_tip_offset,
        angle_deg,
    );
    let left = polar_point(base, arrow_half_w, angle_deg + 90.0);
    let right = polar_point(base, arrow_half_w, angle_deg - 90.0);
    (start, base, tip, left, right)
}

fn round_to_i32(v: f32) -> i32 {
    if v >= 0.0 {
        (v + 0.5) as i32
    } else {
        (v - 0.5) as i32
    }
}

fn gauge_marker_bg_color(ratio: f32) -> DisplayColor {
    let mut accum = 0.0f32;
    let total = GAUGE_SEGMENTS
        .iter()
        .fold(0.0f32, |sum, seg| sum + seg.sweep_deg.abs())
        .max(1.0);
    let target = ratio.clamp(0.0, 1.0) * total;

    for seg in GAUGE_SEGMENTS {
        accum += seg.sweep_deg.abs();
        if target <= accum {
            return brighten(seg.color, 4);
        }
    }

    brighten(RED, 4)
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
            1,
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

fn font_for(font: FontToken) -> &'static MonoFont<'static> {
    match font {
        FontToken::Small => &FONT_6X10,
        FontToken::Medium => &FONT_8X13_BOLD,
        FontToken::Large => &FONT_10X20,
        FontToken::Larger => &FONT_10X20,
        FontToken::Largest => &FONT_10X20,
    }
}

fn text_width(font: &MonoFont<'_>, text: &str) -> i32 {
    let count = text.chars().count() as i32;
    if count <= 0 {
        return 0;
    }

    let glyph_w = font.character_size.width as i32;
    let spacing = font.character_spacing as i32;
    count * glyph_w + (count - 1) * spacing
}

fn centered_aqi_status_pos(font: &MonoFont<'_>, text: &str) -> Point {
    let w = text_width(font, text);
    let x = GAUGE_CENTER.x - (w / 2);
    let y = GAUGE_CENTER.y - font.character_size.height as i32 - AQI_STATUS_GAP_Y;
    Point::new(x, y)
}

fn aqi_status_clear_rect(font: &MonoFont<'_>) -> Rectangle {
    let max_chars = AQI_STATUS_MAX_CHARS.max(1);
    let glyph_w = font.character_size.width as i32;
    let spacing = font.character_spacing as i32;
    let text_w = max_chars * glyph_w + (max_chars - 1) * spacing;
    let text_h = font.character_size.height as i32;

    let w = (text_w + AQI_STATUS_CLEAR_PAD_X * 2).max(0) as u32;
    let h = (text_h + AQI_STATUS_CLEAR_PAD_Y * 2).max(0) as u32;
    let x = GAUGE_CENTER.x - (text_w / 2) - AQI_STATUS_CLEAR_PAD_X;
    let y = GAUGE_CENTER.y - text_h - AQI_STATUS_GAP_Y - AQI_STATUS_CLEAR_PAD_Y;

    Rectangle::new(Point::new(x, y), Size::new(w, h))
}

fn draw_text_aa<D>(
    display: &mut D,
    pos: Point,
    font: &MonoFont<'_>,
    color: DisplayColor,
    text: &str,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    let mut cursor = pos;

    for ch in text.chars() {
        if ch == ' ' {
            cursor.x += font.character_size.width as i32 + font.character_spacing as i32;
            continue;
        }

        let (base, accent) = decompose_swedish_char(ch);
        draw_glyph_aa(display, font, base, cursor, color);
        if accent != AccentMark::None {
            draw_accent_mark(display, font, cursor, color, accent);
        }
        cursor.x += font.character_size.width as i32 + font.character_spacing as i32;
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum AccentMark {
    None,
    Diaeresis,
    Ring,
}

fn decompose_swedish_char(ch: char) -> (char, AccentMark) {
    match ch {
        'Å' => ('A', AccentMark::Ring),
        'Ä' => ('A', AccentMark::Diaeresis),
        'Ö' => ('O', AccentMark::Diaeresis),
        'å' => ('a', AccentMark::Ring),
        'ä' => ('a', AccentMark::Diaeresis),
        'ö' => ('o', AccentMark::Diaeresis),
        _ => (ch, AccentMark::None),
    }
}

fn draw_accent_mark<D>(
    display: &mut D,
    font: &MonoFont<'_>,
    origin: Point,
    color: DisplayColor,
    accent: AccentMark,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    let w = font.character_size.width as i32;
    let y = origin.y - 2;
    if y < 0 {
        return;
    }

    match accent {
        AccentMark::None => {}
        AccentMark::Diaeresis => {
            draw_pixel_safe(display, Point::new(origin.x + (w / 3), y), color);
            draw_pixel_safe(display, Point::new(origin.x + ((w * 2) / 3), y), color);
        }
        AccentMark::Ring => {
            let cx = origin.x + (w / 2);
            draw_pixel_safe(display, Point::new(cx, y - 1), color);
            draw_pixel_safe(display, Point::new(cx - 1, y), color);
            draw_pixel_safe(display, Point::new(cx + 1, y), color);
            draw_pixel_safe(display, Point::new(cx, y + 1), color);
        }
    }
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

fn draw_glyph_aa<D>(
    display: &mut D,
    font: &MonoFont<'_>,
    ch: char,
    origin: Point,
    color: DisplayColor,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    let w = font.character_size.width as i32;
    let h = font.character_size.height as i32;
    let bounds = display.bounding_box();
    let right = bounds.top_left.x + bounds.size.width as i32;
    let bottom = bounds.top_left.y + bounds.size.height as i32;

    for y in -1..=h {
        for x in -1..=w {
            let alpha = glyph_alpha(font, ch, x, y);
            if alpha == 0 {
                continue;
            }

            let p = Point::new(origin.x + x, origin.y + y);
            if p.x < bounds.top_left.x || p.y < bounds.top_left.y || p.x >= right || p.y >= bottom {
                continue;
            }

            let c = scale_color(color, alpha);
            let _ = Pixel(p, c).draw(display);
        }
    }
}

fn glyph_alpha(font: &MonoFont<'_>, ch: char, x: i32, y: i32) -> u8 {
    if glyph_bit(font, ch, x, y) {
        return 255;
    }

    let mut min_d2 = i32::MAX;
    for ny in (y - 2)..=(y + 2) {
        for nx in (x - 2)..=(x + 2) {
            if glyph_bit(font, ch, nx, ny) {
                let dx = nx - x;
                let dy = ny - y;
                let d2 = dx * dx + dy * dy;
                if d2 < min_d2 {
                    min_d2 = d2;
                }
            }
        }
    }

    // Tight AA to avoid halo.
    match min_d2 {
        1 => 84,
        2 => 52,
        _ => 0,
    }
}

fn glyph_bit(font: &MonoFont<'_>, ch: char, x: i32, y: i32) -> bool {
    if x < 0
        || y < 0
        || x >= font.character_size.width as i32
        || y >= font.character_size.height as i32
    {
        return false;
    }

    let glyphs_per_row = font.image.size().width / font.character_size.width;
    let glyph_index = font.glyph_mapping.index(ch) as u32;
    let row = glyph_index / glyphs_per_row;
    let col = glyph_index - row * glyphs_per_row;

    let px = (col * font.character_size.width + x as u32) as i32;
    let py = (row * font.character_size.height + y as u32) as i32;

    matches!(font.image.pixel(Point::new(px, py)), Some(BinaryColor::On))
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

fn aqi_color(aqi: u16) -> DisplayColor {
    match aqi {
        0..=50 => GREEN,
        51..=100 => YELLOW,
        101..=150 => ORANGE,
        _ => RED,
    }
}

fn aqi_level_text(aqi: Option<u16>) -> &'static str {
    match aqi {
        Some(0..=50) => "BRA",
        Some(51..=100) => "OK",
        Some(101..=150) => "SÄMRE",
        Some(151..=200) => "DÅLIG!",
        Some(201..=300) => "MYCKET DÅLIGT",
        Some(_) => "FARLIG NIVÅ!",
        None => "INGEN DATA",
    }
}

fn aqi_from_pm25(pm25: u16) -> u16 {
    let c = pm25 as u32;

    let (cl, ch, il, ih) = if c <= 12 {
        (0, 12, 0, 50)
    } else if c <= 35 {
        (13, 35, 51, 100)
    } else if c <= 55 {
        (36, 55, 101, 150)
    } else if c <= 150 {
        (56, 150, 151, 200)
    } else if c <= 250 {
        (151, 250, 201, 300)
    } else {
        (251, 500, 301, 500)
    };

    let aqi = ((ih - il) * (c.saturating_sub(cl)) / (ch - cl)) + il;
    aqi.min(500) as u16
}
