#[derive(Copy, Clone, Eq, PartialEq)]
pub enum EuAqiBand {
    Good,
    Fair,
    Moderate,
    Poor,
    VeryPoor,
    ExtremelyPoor,
}

pub const EU_PM25_GOOD_MAX: u16 = 5;
pub const EU_PM25_FAIR_MAX: u16 = 15;
pub const EU_PM25_MODERATE_MAX: u16 = 50;
pub const EU_PM25_POOR_MAX: u16 = 90;
pub const EU_PM25_VERY_POOR_MAX: u16 = 140;
pub const PM25_SCALE_MAX_UGM3: u16 = 180;

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

pub fn level_text_sv(pm25: Option<u16>) -> &'static str {
    match pm25.map(band_from_pm25) {
        Some(EuAqiBand::Good) => "GOD",
        Some(EuAqiBand::Fair) => "GANSKA GOD",
        Some(EuAqiBand::Moderate) => "MÅTTLIG",
        Some(EuAqiBand::Poor) => "DÅLIG",
        Some(EuAqiBand::VeryPoor) => "MYCKET DÅLIG",
        Some(EuAqiBand::ExtremelyPoor) => "YTTERST DÅLIG",
        None => "INGEN DATA",
    }
}

pub fn ratio_from_pm25(pm25: u16) -> f32 {
    pm25.min(PM25_SCALE_MAX_UGM3) as f32 / PM25_SCALE_MAX_UGM3 as f32
}
