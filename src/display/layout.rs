use super::{
    font_height, TextStyleCfg, STYLE_CLIMATE_LABEL, STYLE_PARTICLE_LABEL, STYLE_PARTICLE_VALUE,
};

pub(super) const CLIMATE_VALUE_GAP_Y: i32 = 1;
const CLIMATE_VALUE_CLEAR_PAD_TOP: i32 = 2;
const CLIMATE_VALUE_CLEAR_PAD_BOTTOM: i32 = 0;
pub(super) const CLIMATE_TEMP_VALUE_CLEAR_W: u32 = 72;
pub(super) const CLIMATE_PRESSURE_VALUE_CLEAR_W: u32 = 92;
pub(super) const CLIMATE_HUM_VALUE_CLEAR_W: u32 = 40;

#[derive(Copy, Clone)]
pub(super) struct LabelCfg {
    pub x: i32,
    pub y: i32,
    pub text: &'static str,
    pub style: TextStyleCfg,
}

#[derive(Copy, Clone)]
pub(super) struct FieldCfg {
    pub x: i32,
    pub y: i32,
    pub clear_x: i32,
    pub clear_y: i32,
    pub clear_w: u32,
    pub clear_h: u32,
    pub style: TextStyleCfg,
}

// Static labels by function
pub(super) const LABEL_TEMP: LabelCfg = LabelCfg {
    x: 8,
    y: 2,
    text: "Temp",
    style: STYLE_CLIMATE_LABEL,
};
pub(super) const LABEL_RH: LabelCfg = LabelCfg {
    x: 198,
    y: 2,
    text: "RH",
    style: STYLE_CLIMATE_LABEL,
};
pub(super) const LABEL_PRESSURE: LabelCfg = LabelCfg {
    x: 96,
    y: 2,
    text: "Tryck",
    style: STYLE_CLIMATE_LABEL,
};
pub(super) const LABEL_PM1: LabelCfg = LabelCfg {
    x: 10,
    y: 198,
    text: "PM1.0",
    style: STYLE_PARTICLE_LABEL,
};
pub(super) const LABEL_PM25: LabelCfg = LabelCfg {
    x: 88,
    y: 198,
    text: "PM2.5",
    style: STYLE_PARTICLE_LABEL,
};
pub(super) const LABEL_PM10: LabelCfg = LabelCfg {
    x: 168,
    y: 198,
    text: "PM10",
    style: STYLE_PARTICLE_LABEL,
};
pub(super) const LABEL_PM03: LabelCfg = LabelCfg {
    x: 8,
    y: 262,
    text: "PM0.3",
    style: STYLE_PARTICLE_LABEL,
};
pub(super) const LABEL_PM05: LabelCfg = LabelCfg {
    x: 128,
    y: 262,
    text: "PM0.5",
    style: STYLE_PARTICLE_LABEL,
};

// Dynamic fields by function
pub(super) const FIELD_PM1: FieldCfg = FieldCfg {
    x: 8,
    y: 224,
    clear_x: 8,
    clear_y: 224,
    clear_w: 68,
    clear_h: 22,
    style: STYLE_PARTICLE_VALUE,
};
pub(super) const FIELD_PM25: FieldCfg = FieldCfg {
    x: 88,
    y: 224,
    clear_x: 88,
    clear_y: 224,
    clear_w: 68,
    clear_h: 22,
    style: STYLE_PARTICLE_VALUE,
};
pub(super) const FIELD_PM10: FieldCfg = FieldCfg {
    x: 168,
    y: 224,
    clear_x: 168,
    clear_y: 224,
    clear_w: 62,
    clear_h: 22,
    style: STYLE_PARTICLE_VALUE,
};
pub(super) const FIELD_PM03: FieldCfg = FieldCfg {
    x: 8,
    y: 286,
    clear_x: 8,
    clear_y: 286,
    clear_w: 104,
    clear_h: 22,
    style: STYLE_PARTICLE_VALUE,
};
pub(super) const FIELD_PM05: FieldCfg = FieldCfg {
    x: 128,
    y: 286,
    clear_x: 128,
    clear_y: 286,
    clear_w: 104,
    clear_h: 22,
    style: STYLE_PARTICLE_VALUE,
};

pub(super) fn climate_value_field(x: i32, clear_w: u32, style: TextStyleCfg) -> FieldCfg {
    let y = climate_value_y();
    let text_h = font_height(style.font);
    let clear_y = y - CLIMATE_VALUE_CLEAR_PAD_TOP;
    let clear_h =
        (text_h + CLIMATE_VALUE_CLEAR_PAD_TOP + CLIMATE_VALUE_CLEAR_PAD_BOTTOM).max(1) as u32;

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

fn climate_value_y() -> i32 {
    LABEL_TEMP.y + font_height(STYLE_CLIMATE_LABEL.font) + CLIMATE_VALUE_GAP_Y
}
