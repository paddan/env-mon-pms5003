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
    primitives::{Arc, Circle, Line, PrimitiveStyle, Rectangle},
};
use heapless::String;
use micromath::F32Ext;
use u8g2_fonts::{
    fonts,
    types::{FontColor, VerticalPosition},
    FontRenderer,
};

use crate::{bme280::BmeReading, pms5003::Pms5003Reading};

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
    Largest,
}

#[derive(Copy, Clone)]
struct TextStyleCfg {
    font: FontToken,
    color: DisplayColor,
}

#[derive(Copy, Clone)]
enum U8g2FontToken {
    Logisoso20,
    Logisoso24,
}

#[derive(Copy, Clone)]
enum ResolvedFont {
    Mono(&'static MonoFont<'static>),
    U8g2(U8g2FontToken),
}

// Typography by function
const STYLE_HEADER_TEXT: TextStyleCfg = TextStyleCfg {
    font: FontToken::Small,
    color: TEXT_DIM,
};

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
const CLIMATE_VALUE_GAP_Y: i32 = 3;
const CLIMATE_VALUE_CLEAR_PAD_Y: i32 = 1;
const CLIMATE_TEMP_VALUE_CLEAR_W: u32 = 72;
const CLIMATE_PRESSURE_VALUE_CLEAR_W: u32 = 92;
const CLIMATE_HUM_VALUE_CLEAR_W: u32 = 40;

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
    y: 178,
    clear_x: 8,
    clear_y: 178,
    clear_w: 220,
    clear_h: 12,
    style: STYLE_HEADER_TEXT,
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
    y: 224,
    clear_x: 88,
    clear_y: 224,
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
const GAUGE_POINTER_LENGTH_FACTOR: f32 = 1.1;
const GAUGE_POINTER_MAX_EXTRA_R_BASE: i32 = 120;
const GAUGE_RESTORE_SPAN_DEG: f32 = 7.0;
const GAUGE_NEEDLE_INNER_R_BASE: i32 = 8;
const GAUGE_NEEDLE_OUTER_R_BASE: i32 = 70;
const GAUGE_NEEDLE_W_BASE: i32 = 4;
const GAUGE_NEEDLE_SHADOW_W_BASE: i32 = 6;
const GAUGE_NEEDLE_CLEAR_W_BASE: i32 = 8;
const GAUGE_NEEDLE_COLOR: DisplayColor = TEXT_WHITE;
const GAUGE_NEEDLE_SHADOW_COLOR: DisplayColor = DisplayColor::new(5, 12, 5);
const GAUGE_HUB_D_BASE: u32 = 10;
const GAUGE_HUB_CLEAR_D_BASE: u32 = 12;
const GAUGE_HUB_COLOR: DisplayColor = TEXT_WHITE;
const STATUS_TEXT_GAP_Y: i32 = 20;
const STATUS_TEXT_CLEAR_PAD_X: i32 = 4;
const STATUS_TEXT_CLEAR_PAD_Y: i32 = 2;
const STATUS_TEXT_MAX_CHARS: i32 = 16;
const EU_PM25_GOOD_MAX: u16 = 5;
const EU_PM25_FAIR_MAX: u16 = 15;
const EU_PM25_MODERATE_MAX: u16 = 50;
const EU_PM25_POOR_MAX: u16 = 90;
const EU_PM25_VERY_POOR_MAX: u16 = 140;
const PM25_SCALE_MAX_UGM3: u16 = 180;
const GAUGE_ARROW_LEN_BASE: i32 = 11;
const GAUGE_ARROW_HALF_W_BASE: i32 = 6;
const GAUGE_ARROW_TIP_OFFSET_BASE: i32 = 3;
const GAUGE_ARROW_SHADOW_PAD_BASE: i32 = 1;
const GAUGE_ARROW_CLEAR_PAD_BASE: i32 = 2;

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
    header: String<48>,
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
            header: String::new(),
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
    _sensor_label: Option<&str>,
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

    let header = "ENV-MONITOR by JPL Design";
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

    update_field_if_changed(display, &mut cache.header, header, FIELD_HEADER);
    update_field_if_changed(display, &mut cache.temp, temp_text.as_str(), field_temp);
    update_field_if_changed(display, &mut cache.hum, hum_text.as_str(), field_hum);
    update_field_if_changed(
        display,
        &mut cache.pressure,
        pressure_text.as_str(),
        field_pressure,
    );
    let status_text = air_quality_level_text(status_pm25);
    update_status_if_changed(display, cache, status_text, status_pm25);

    update_field_if_changed(display, &mut cache.pm1, pm1_text.as_str(), FIELD_PM1);
    update_field_if_changed(display, &mut cache.pm25, pm25_text.as_str(), FIELD_PM25);
    update_field_if_changed(display, &mut cache.pm10, pm10_text.as_str(), FIELD_PM10);
    update_field_if_changed(display, &mut cache.pm03, pm03_text.as_str(), FIELD_PM03);
    update_field_if_changed(display, &mut cache.pm05, pm05_text.as_str(), FIELD_PM05);
}

fn climate_value_y() -> i32 {
    let label_font = font_for(STYLE_CLIMATE_LABEL.font);
    LABEL_TEMP.y + font_height(label_font) + CLIMATE_VALUE_GAP_Y
}

fn climate_value_field(x: i32, clear_w: u32, style: TextStyleCfg) -> FieldCfg {
    let y = climate_value_y();
    let text_h = font_height(font_for(style.font));
    let clear_y = y - CLIMATE_VALUE_CLEAR_PAD_Y;
    let clear_h = (text_h + CLIMATE_VALUE_CLEAR_PAD_Y * 2).max(1) as u32;

    FieldCfg {
        x,
        y,
        clear_x: x,
        clear_y,
        clear_w,
        clear_h,
        style,
    }
}

fn draw_gauge_scale<D>(display: &mut D)
where
    D: DrawTarget<Color = DisplayColor>,
{
    draw_gauge_gradient_span(
        display,
        GAUGE_START_DEG,
        GAUGE_TOTAL_SWEEP_DEG,
        GAUGE_GRADIENT_STEP_DEG_STATIC,
    );
}

fn update_status_if_changed<D>(
    display: &mut D,
    cache: &mut DisplayCache,
    text: &str,
    pm25: Option<u16>,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    if cache.status_text.as_str() == text && cache.status_pm25 == pm25 {
        return;
    }

    if let Some(prev_pm25) = cache.status_pm25 {
        let ratio = status_ratio(prev_pm25);
        let angle = gauge_angle(ratio);
        erase_status_needle(display, angle);
        restore_gauge_slice(display, angle);
    }

    let value_style = TextStyleCfg {
        font: STYLE_STATUS_TEXT.font,
        color: pm25.map(status_color).unwrap_or(TEXT_DIM),
    };
    let font = font_for(value_style.font);
    clear_rect(display, status_text_clear_rect(font));

    let text_pos = centered_status_text_pos(font, text);
    draw_text_aa(display, text_pos, font, value_style.color, text);

    if let Some(current_pm25) = pm25 {
        let ratio = status_ratio(current_pm25);
        let angle = gauge_angle(ratio);
        draw_status_needle(display, angle);
    }

    cache.status_text.clear();
    let _ = cache.status_text.push_str(text);
    cache.status_pm25 = pm25;
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

fn draw_status_needle<D>(display: &mut D, angle_deg: f32)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let inner_r = gauge_scale_i32(GAUGE_NEEDLE_INNER_R_BASE);
    let outer_r = pointer_outer_radius(inner_r);
    let arrow_len = gauge_scale_i32(GAUGE_ARROW_LEN_BASE);
    let arrow_half_w = gauge_scale_i32(GAUGE_ARROW_HALF_W_BASE);
    let arrow_tip_offset = gauge_scale_i32(GAUGE_ARROW_TIP_OFFSET_BASE);
    let shadow_pad = gauge_scale_i32_nonzero(GAUGE_ARROW_SHADOW_PAD_BASE);

    let (start, shaft_end, tip, left, right) = needle_geometry(
        angle_deg,
        inner_r,
        outer_r,
        arrow_len,
        arrow_half_w,
        arrow_tip_offset,
    );
    let (_, _, shadow_tip, shadow_left, shadow_right) = needle_geometry(
        angle_deg,
        inner_r,
        outer_r,
        arrow_len + shadow_pad,
        arrow_half_w + shadow_pad,
        arrow_tip_offset + shadow_pad,
    );

    draw_capsule_aa(
        display,
        start,
        shaft_end,
        gauge_scale_i32_nonzero(GAUGE_NEEDLE_SHADOW_W_BASE) as f32,
        GAUGE_NEEDLE_SHADOW_COLOR,
    );
    fill_triangle_aa(
        display,
        shadow_tip,
        shadow_left,
        shadow_right,
        GAUGE_NEEDLE_SHADOW_COLOR,
    );

    draw_capsule_aa(
        display,
        start,
        shaft_end,
        gauge_scale_i32_nonzero(GAUGE_NEEDLE_W_BASE) as f32,
        GAUGE_NEEDLE_COLOR,
    );
    fill_triangle_aa(display, tip, left, right, GAUGE_NEEDLE_COLOR);

    let _ = Circle::with_center(GAUGE_CENTER, gauge_scale_u32_nonzero(GAUGE_HUB_D_BASE))
        .into_styled(PrimitiveStyle::with_fill(GAUGE_HUB_COLOR))
        .draw(display);
}

fn erase_status_needle<D>(display: &mut D, angle_deg: f32)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let inner_r = gauge_scale_i32(GAUGE_NEEDLE_INNER_R_BASE);
    let outer_r = pointer_outer_radius(inner_r);
    let clear_pad = gauge_scale_i32_nonzero(GAUGE_ARROW_CLEAR_PAD_BASE);

    let (start, _, tip, left, right) = needle_geometry(
        angle_deg,
        inner_r,
        outer_r,
        gauge_scale_i32(GAUGE_ARROW_LEN_BASE) + clear_pad,
        gauge_scale_i32(GAUGE_ARROW_HALF_W_BASE) + clear_pad,
        gauge_scale_i32(GAUGE_ARROW_TIP_OFFSET_BASE) + clear_pad,
    );

    draw_capsule_aa(
        display,
        start,
        tip,
        gauge_scale_i32_nonzero(GAUGE_NEEDLE_CLEAR_W_BASE) as f32,
        BG_COLOR,
    );

    fill_triangle_aa(display, tip, left, right, BG_COLOR);

    let _ = Circle::with_center(
        GAUGE_CENTER,
        gauge_scale_u32_nonzero(GAUGE_HUB_CLEAR_D_BASE),
    )
    .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
    .draw(display);
}

fn status_ratio(pm25: u16) -> f32 {
    pm25.min(PM25_SCALE_MAX_UGM3) as f32 / PM25_SCALE_MAX_UGM3 as f32
}

fn gauge_angle(ratio: f32) -> f32 {
    GAUGE_START_DEG + ratio.clamp(0.0, 1.0) * GAUGE_TOTAL_SWEEP_DEG
}

fn gauge_scale_i32(base: i32) -> i32 {
    ((base * GAUGE_DIAMETER as i32) + (GAUGE_REF_DIAMETER / 2)) / GAUGE_REF_DIAMETER
}

fn gauge_scale_i32_nonzero(base: i32) -> i32 {
    gauge_scale_i32(base).max(1)
}

fn gauge_scale_u32_nonzero(base: u32) -> u32 {
    let scaled =
        ((base as i32 * GAUGE_DIAMETER as i32) + (GAUGE_REF_DIAMETER / 2)) / GAUGE_REF_DIAMETER;
    scaled.max(1) as u32
}

fn pointer_outer_radius(inner_r: i32) -> i32 {
    let gauge_r = (GAUGE_DIAMETER as i32) / 2;
    let min_outer = inner_r + 1;
    let base_outer = gauge_scale_i32(GAUGE_NEEDLE_OUTER_R_BASE);
    let desired_outer = round_to_i32(base_outer as f32 * GAUGE_POINTER_LENGTH_FACTOR.max(0.1));
    let max_outer = gauge_r + gauge_scale_i32(GAUGE_POINTER_MAX_EXTRA_R_BASE);

    desired_outer.clamp(min_outer, max_outer.max(min_outer))
}

fn restore_gauge_slice<D>(display: &mut D, angle_deg: f32)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let dir = if GAUGE_TOTAL_SWEEP_DEG >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let sweep = GAUGE_RESTORE_SPAN_DEG * dir;
    let start = angle_deg - (sweep * 0.5);
    draw_gauge_gradient_span(display, start, sweep, GAUGE_GRADIENT_STEP_DEG_RESTORE);
}

fn angle_to_ratio(angle_deg: f32) -> f32 {
    if GAUGE_TOTAL_SWEEP_DEG.abs() < f32::EPSILON {
        0.0
    } else {
        ((angle_deg - GAUGE_START_DEG) / GAUGE_TOTAL_SWEEP_DEG).clamp(0.0, 1.0)
    }
}

fn draw_gauge_gradient_span<D>(display: &mut D, start_deg: f32, sweep_deg: f32, step_deg: f32)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let total_sweep = sweep_deg.abs();
    if total_sweep <= f32::EPSILON {
        return;
    }

    let dir = if sweep_deg >= 0.0 { 1.0 } else { -1.0 };
    let step = step_deg.max(0.25);
    let mut walked = 0.0f32;

    while walked < total_sweep {
        let chunk = (total_sweep - walked).min(step);
        let chunk_start = start_deg + walked * dir;
        let chunk_mid = chunk_start + (chunk * 0.5) * dir;
        let color = gauge_gradient_color(angle_to_ratio(chunk_mid));
        draw_arc_band(display, chunk_start, chunk * dir, color);
        walked += chunk;
    }
}

fn polar_point(center: Point, radius: i32, angle_deg: f32) -> Point {
    let rad = angle_deg * (core::f32::consts::PI / 180.0);
    let x = center.x + round_to_i32((radius as f32) * rad.cos());
    let y = center.y + round_to_i32((radius as f32) * rad.sin());
    Point::new(x, y)
}

fn needle_geometry(
    angle_deg: f32,
    inner_r: i32,
    outer_r: i32,
    arrow_len: i32,
    arrow_half_w: i32,
    arrow_tip_offset: i32,
) -> (Point, Point, Point, Point, Point) {
    let start = polar_point(GAUGE_CENTER, inner_r, angle_deg);
    let base = polar_point(GAUGE_CENTER, outer_r - arrow_len, angle_deg);
    let tip = polar_point(GAUGE_CENTER, outer_r + arrow_tip_offset, angle_deg);
    let left = polar_point(base, arrow_half_w, angle_deg + 90.0);
    let right = polar_point(base, arrow_half_w, angle_deg - 90.0);
    (start, base, tip, left, right)
}

fn draw_capsule_aa<D>(display: &mut D, start: Point, end: Point, width: f32, color: DisplayColor)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let half_w = (width * 0.5).max(0.5);
    let pad = round_to_i32(half_w + 1.0);
    let min_x = start.x.min(end.x) - pad;
    let max_x = start.x.max(end.x) + pad;
    let min_y = start.y.min(end.y) - pad;
    let max_y = start.y.max(end.y) + pad;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let alpha = capsule_alpha(x as f32 + 0.5, y as f32 + 0.5, start, end, half_w);
            if alpha <= 0.0 {
                continue;
            }

            let a = round_to_i32((alpha * 255.0).clamp(0.0, 255.0)) as u8;
            draw_pixel_safe(display, Point::new(x, y), scale_color(color, a));
        }
    }
}

fn capsule_alpha(px: f32, py: f32, start: Point, end: Point, half_w: f32) -> f32 {
    let dist = point_segment_distance(px, py, start, end);
    (half_w + 0.5 - dist).clamp(0.0, 1.0)
}

fn point_segment_distance(px: f32, py: f32, start: Point, end: Point) -> f32 {
    let ax = start.x as f32;
    let ay = start.y as f32;
    let bx = end.x as f32;
    let by = end.y as f32;
    let vx = bx - ax;
    let vy = by - ay;
    let len2 = vx * vx + vy * vy;

    if len2 <= 0.0001 {
        let dx = px - ax;
        let dy = py - ay;
        return (dx * dx + dy * dy).sqrt();
    }

    let t = (((px - ax) * vx + (py - ay) * vy) / len2).clamp(0.0, 1.0);
    let cx = ax + t * vx;
    let cy = ay + t * vy;
    let dx = px - cx;
    let dy = py - cy;
    (dx * dx + dy * dy).sqrt()
}

fn fill_triangle_aa<D>(display: &mut D, a: Point, b: Point, c: Point, color: DisplayColor)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let min_x = a.x.min(b.x).min(c.x);
    let max_x = a.x.max(b.x).max(c.x);
    let min_y = a.y.min(b.y).min(c.y);
    let max_y = a.y.max(b.y).max(c.y);
    let sample_offsets = [0.25_f32, 0.75_f32];

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let mut covered = 0u8;
            for sy in sample_offsets {
                for sx in sample_offsets {
                    if point_in_triangle(x as f32 + sx, y as f32 + sy, a, b, c) {
                        covered += 1;
                    }
                }
            }

            if covered == 0 {
                continue;
            }

            let alpha = (covered as u16 * 255 / 4) as u8;
            draw_pixel_safe(display, Point::new(x, y), scale_color(color, alpha));
        }
    }
}

fn point_in_triangle(px: f32, py: f32, a: Point, b: Point, c: Point) -> bool {
    let w0 = edge_fn(b, c, px, py);
    let w1 = edge_fn(c, a, px, py);
    let w2 = edge_fn(a, b, px, py);
    let has_neg = w0 < 0.0 || w1 < 0.0 || w2 < 0.0;
    let has_pos = w0 > 0.0 || w1 > 0.0 || w2 > 0.0;
    !(has_neg && has_pos)
}

fn edge_fn(a: Point, b: Point, px: f32, py: f32) -> f32 {
    let ax = a.x as f32;
    let ay = a.y as f32;
    let bx = b.x as f32;
    let by = b.y as f32;
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

fn round_to_i32(v: f32) -> i32 {
    if v >= 0.0 {
        (v + 0.5) as i32
    } else {
        (v - 0.5) as i32
    }
}

fn gauge_gradient_color(ratio: f32) -> DisplayColor {
    let mut accum = 0.0f32;
    let total = gauge_total_span_deg();
    let target = ratio.clamp(0.0, 1.0) * total;
    let blend_half = (GAUGE_COLOR_BLEND_SPAN_DEG * 0.5).max(0.1);

    for (idx, seg) in GAUGE_SEGMENTS.iter().enumerate() {
        let seg_span = seg.sweep_deg.abs();
        let seg_end = accum + seg_span;

        if idx + 1 < GAUGE_SEGMENTS.len() {
            let blend_start = (seg_end - blend_half).max(accum);
            let blend_end = (seg_end + blend_half).min(total);

            if target >= blend_start && target <= blend_end {
                let next = GAUGE_SEGMENTS[idx + 1].color;
                let denom = (blend_end - blend_start).max(0.001);
                let t = ((target - blend_start) / denom).clamp(0.0, 1.0);
                return lerp_color(seg.color, next, t);
            }
        }

        if target <= seg_end {
            return seg.color;
        }

        accum = seg_end;
    }

    RED
}

fn gauge_total_span_deg() -> f32 {
    GAUGE_SEGMENTS
        .iter()
        .fold(0.0f32, |sum, seg| sum + seg.sweep_deg.abs())
        .max(1.0)
}

fn lerp_color(a: DisplayColor, b: DisplayColor, t: f32) -> DisplayColor {
    let clamped_t = t.clamp(0.0, 1.0);
    DisplayColor::new(
        lerp_u5(a.r(), b.r(), clamped_t),
        lerp_u6(a.g(), b.g(), clamped_t),
        lerp_u5(a.b(), b.b(), clamped_t),
    )
}

fn lerp_u5(a: u8, b: u8, t: f32) -> u8 {
    round_to_i32(a as f32 + (b as f32 - a as f32) * t).clamp(0, 31) as u8
}

fn lerp_u6(a: u8, b: u8, t: f32) -> u8 {
    round_to_i32(a as f32 + (b as f32 - a as f32) * t).clamp(0, 63) as u8
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

fn font_for(font: FontToken) -> ResolvedFont {
    match font {
        FontToken::Small => ResolvedFont::Mono(&FONT_6X10),
        FontToken::Medium => ResolvedFont::Mono(&FONT_8X13_BOLD),
        FontToken::Large => ResolvedFont::Mono(&FONT_10X20),
        FontToken::Larger => ResolvedFont::U8g2(U8g2FontToken::Logisoso20),
        FontToken::Largest => ResolvedFont::U8g2(U8g2FontToken::Logisoso24),
    }
}

fn text_width(font: ResolvedFont, text: &str) -> i32 {
    match font {
        ResolvedFont::Mono(mono) => {
            let count = text.chars().count() as i32;
            if count <= 0 {
                return 0;
            }

            let glyph_w = mono.character_size.width as i32;
            let spacing = mono.character_spacing as i32;
            count * glyph_w + (count - 1) * spacing
        }
        ResolvedFont::U8g2(face) => u8g2_text_width(face, text),
    }
}

fn centered_status_text_pos(font: ResolvedFont, text: &str) -> Point {
    let w = text_width(font, text);
    let x = GAUGE_CENTER.x - (w / 2);
    let y = GAUGE_CENTER.y - font_height(font) - STATUS_TEXT_GAP_Y;
    Point::new(x, y)
}

fn status_text_clear_rect(font: ResolvedFont) -> Rectangle {
    let max_chars = STATUS_TEXT_MAX_CHARS.max(1);
    let glyph_w = text_width(font, "0");
    let spacing = match font {
        ResolvedFont::Mono(mono) => mono.character_spacing as i32,
        ResolvedFont::U8g2(_) => 1,
    };
    let text_w = max_chars * glyph_w + (max_chars - 1) * spacing;
    let text_h = font_height(font);

    let w = (text_w + STATUS_TEXT_CLEAR_PAD_X * 2).max(0) as u32;
    let h = (text_h + STATUS_TEXT_CLEAR_PAD_Y * 2).max(0) as u32;
    let x = GAUGE_CENTER.x - (text_w / 2) - STATUS_TEXT_CLEAR_PAD_X;
    let y = GAUGE_CENTER.y - text_h - STATUS_TEXT_GAP_Y - STATUS_TEXT_CLEAR_PAD_Y;

    Rectangle::new(Point::new(x, y), Size::new(w, h))
}

fn draw_text_aa<D>(display: &mut D, pos: Point, font: ResolvedFont, color: DisplayColor, text: &str)
where
    D: DrawTarget<Color = DisplayColor>,
{
    match font {
        ResolvedFont::Mono(mono) => draw_text_mono_aa(display, pos, mono, color, text),
        ResolvedFont::U8g2(face) => draw_text_u8g2(display, pos, face, color, text),
    }
}

fn draw_text_mono_aa<D>(
    display: &mut D,
    pos: Point,
    font: &MonoFont<'_>,
    color: DisplayColor,
    text: &str,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    let mut cursor = pos;
    let advance = font.character_size.width as i32 + font.character_spacing as i32;

    for ch in text.chars() {
        if ch == ' ' {
            cursor.x += advance;
            continue;
        }

        let (base, accent) = decompose_swedish_char(ch);
        draw_glyph_aa(display, font, base, cursor, color);
        if accent != AccentMark::None {
            draw_accent_mark_mono(display, font, cursor, color, accent);
        }
        cursor.x += advance;
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

fn draw_accent_mark_mono<D>(
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

fn draw_text_u8g2<D>(
    display: &mut D,
    pos: Point,
    font: U8g2FontToken,
    color: DisplayColor,
    text: &str,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    match font {
        U8g2FontToken::Logisoso20 => {
            let renderer = FontRenderer::new::<fonts::u8g2_font_logisoso20_tr>()
                .with_ignore_unknown_chars(true);
            let _ = renderer.render(
                text_ascii_fallback(text).as_str(),
                pos,
                VerticalPosition::Top,
                FontColor::Transparent(color),
                display,
            );
        }
        U8g2FontToken::Logisoso24 => {
            let renderer = FontRenderer::new::<fonts::u8g2_font_logisoso24_tr>()
                .with_ignore_unknown_chars(true);
            let _ = renderer.render(
                text_ascii_fallback(text).as_str(),
                pos,
                VerticalPosition::Top,
                FontColor::Transparent(color),
                display,
            );
        }
    }
}

fn u8g2_text_width(font: U8g2FontToken, text: &str) -> i32 {
    let fallback = text_ascii_fallback(text);
    let dims = match font {
        U8g2FontToken::Logisoso20 => FontRenderer::new::<fonts::u8g2_font_logisoso20_tr>()
            .with_ignore_unknown_chars(true)
            .get_rendered_dimensions(fallback.as_str(), Point::zero(), VerticalPosition::Top),
        U8g2FontToken::Logisoso24 => FontRenderer::new::<fonts::u8g2_font_logisoso24_tr>()
            .with_ignore_unknown_chars(true)
            .get_rendered_dimensions(fallback.as_str(), Point::zero(), VerticalPosition::Top),
    };

    dims.map(|d| d.advance.x.max(0)).unwrap_or(0)
}

fn u8g2_font_height(font: U8g2FontToken) -> i32 {
    match font {
        U8g2FontToken::Logisoso20 => {
            FontRenderer::new::<fonts::u8g2_font_logisoso20_tr>().get_default_line_height() as i32
        }
        U8g2FontToken::Logisoso24 => {
            FontRenderer::new::<fonts::u8g2_font_logisoso24_tr>().get_default_line_height() as i32
        }
    }
}

fn font_height(font: ResolvedFont) -> i32 {
    match font {
        ResolvedFont::Mono(mono) => mono.character_size.height as i32,
        ResolvedFont::U8g2(face) => u8g2_font_height(face),
    }
}

fn text_ascii_fallback(text: &str) -> String<64> {
    let mut out: String<64> = String::new();
    for ch in text.chars() {
        let mapped = match ch {
            'Å' | 'Ä' => 'A',
            'Ö' => 'O',
            'å' | 'ä' => 'a',
            'ö' => 'o',
            _ => ch,
        };
        let _ = out.push(mapped);
    }
    out
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

fn status_color(pm25: u16) -> DisplayColor {
    if pm25 <= EU_PM25_GOOD_MAX {
        GREEN
    } else if pm25 <= EU_PM25_FAIR_MAX {
        LIME
    } else if pm25 <= EU_PM25_MODERATE_MAX {
        YELLOW
    } else if pm25 <= EU_PM25_POOR_MAX {
        ORANGE
    } else if pm25 <= EU_PM25_VERY_POOR_MAX {
        RED
    } else {
        DEEP_RED
    }
}

fn air_quality_level_text(pm25: Option<u16>) -> &'static str {
    match pm25 {
        Some(v) if v <= EU_PM25_GOOD_MAX => "BRA NIVÅ",
        Some(v) if v <= EU_PM25_FAIR_MAX => "GODTAGBAR",
        Some(v) if v <= EU_PM25_MODERATE_MAX => "MÅTTLIG",
        Some(v) if v <= EU_PM25_POOR_MAX => "DÅLIG",
        Some(v) if v <= EU_PM25_VERY_POOR_MAX => "MYCKET DÅLIG",
        Some(_) => "EXTREMT DÅLIG",
        None => "INGEN DATA",
    }
}
