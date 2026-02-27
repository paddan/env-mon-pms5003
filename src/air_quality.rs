#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum EuAqiBand {
    Good,
    Fair,
    Moderate,
    Poor,
    VeryPoor,
    ExtremelyPoor,
}

// EU thresholds for PM2.5 (µg/m³, 24 h average)
pub const EU_PM25_GOOD_MAX: u16 = 5;
pub const EU_PM25_FAIR_MAX: u16 = 15;
pub const EU_PM25_MODERATE_MAX: u16 = 50;
pub const EU_PM25_POOR_MAX: u16 = 90;
pub const EU_PM25_VERY_POOR_MAX: u16 = 140;
pub const PM25_SCALE_MAX_UGM3: u16 = 180;

// EU thresholds for PM10 (µg/m³, 24 h average) — ~2× the PM2.5 thresholds
pub const EU_PM10_GOOD_MAX: u16 = 10;
pub const EU_PM10_FAIR_MAX: u16 = 25;
pub const EU_PM10_MODERATE_MAX: u16 = 90;
pub const EU_PM10_POOR_MAX: u16 = 180;
pub const EU_PM10_VERY_POOR_MAX: u16 = 280;

pub fn band_from_pm25(pm25: u16) -> EuAqiBand {
    if pm25 <= EU_PM25_GOOD_MAX {
        EuAqiBand::Good
    } else if pm25 <= EU_PM25_FAIR_MAX {
        EuAqiBand::Fair
    } else if pm25 <= EU_PM25_MODERATE_MAX {
        EuAqiBand::Moderate
    } else if pm25 <= EU_PM25_POOR_MAX {
        EuAqiBand::Poor
    } else if pm25 <= EU_PM25_VERY_POOR_MAX {
        EuAqiBand::VeryPoor
    } else {
        EuAqiBand::ExtremelyPoor
    }
}

pub fn band_from_pm10(pm10: u16) -> EuAqiBand {
    if pm10 <= EU_PM10_GOOD_MAX {
        EuAqiBand::Good
    } else if pm10 <= EU_PM10_FAIR_MAX {
        EuAqiBand::Fair
    } else if pm10 <= EU_PM10_MODERATE_MAX {
        EuAqiBand::Moderate
    } else if pm10 <= EU_PM10_POOR_MAX {
        EuAqiBand::Poor
    } else if pm10 <= EU_PM10_VERY_POOR_MAX {
        EuAqiBand::VeryPoor
    } else {
        EuAqiBand::ExtremelyPoor
    }
}

/// Midpoint of each band on the PM2.5 scale, used to position the gauge needle
/// when PM10 is the driving metric.
fn band_pm25_midpoint(band: EuAqiBand) -> u16 {
    match band {
        EuAqiBand::Good => 2,
        EuAqiBand::Fair => 10,
        EuAqiBand::Moderate => 32,
        EuAqiBand::Poor => 70,
        EuAqiBand::VeryPoor => 115,
        EuAqiBand::ExtremelyPoor => 160,
    }
}

/// Returns a PM2.5-equivalent value for gauge display, reflecting the worse of
/// PM2.5 and PM10 each assessed against their own EU thresholds.
/// When PM2.5 drives the band the raw PM2.5 value is returned (precise needle
/// positioning); when PM10 drives the band a representative PM2.5 midpoint for
/// that band is returned instead.
pub fn aqi_pm25_equiv(pm2_5: u16, pm10: u16) -> u16 {
    let band_25 = band_from_pm25(pm2_5);
    let band_10 = band_from_pm10(pm10);
    if band_10 > band_25 {
        band_pm25_midpoint(band_10)
    } else {
        pm2_5
    }
}

pub fn level_text_sv(pm25: Option<u16>) -> &'static str {
    match pm25.map(band_from_pm25) {
        Some(EuAqiBand::Good) => "GOD LUFTKVALITET",
        Some(EuAqiBand::Fair) => "GANSKA GOD LUFTKVALITET",
        Some(EuAqiBand::Moderate) => "MÅTTLIGT GOD LUFTKVALITET",
        Some(EuAqiBand::Poor) => "DÅLIG LUFTKVALITET",
        Some(EuAqiBand::VeryPoor) => "MYCKET DÅLIG LUFTKVALITET",
        Some(EuAqiBand::ExtremelyPoor) => "EXTREMT DÅLIG LUFTKVALITET",
        None => "INGEN DATA",
    }
}

pub fn ratio_from_pm25(pm25: u16) -> f32 {
    pm25.min(PM25_SCALE_MAX_UGM3) as f32 / PM25_SCALE_MAX_UGM3 as f32
}
