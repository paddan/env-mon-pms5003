use ::bme280::Measurements;
use esp_hal::{
    Blocking,
    i2c::master::I2c,
};

#[derive(Clone, Copy)]
pub struct BmeReading {
    pub temperature_c_x10: i16,
    pub humidity_pct_x10: u16,
    pub pressure_pa: u32,
}

impl BmeReading {
    pub fn from_measurements<E>(measurements: &Measurements<E>) -> Self {
        let temperature_c_x10 = float_to_i16_tenths(measurements.temperature);
        let humidity_pct_x10 = float_to_u16_tenths(measurements.humidity);
        let pressure_pa = float_to_u32(measurements.pressure);

        Self {
            temperature_c_x10,
            humidity_pct_x10,
            pressure_pa,
        }
    }
}

pub fn detect_bme_address(i2c: &mut I2c<'_, Blocking>) -> Option<(u8, u8)> {
    let mut chip_id = [0u8; 1];

    for address in [0x76u8, 0x77u8] {
        if i2c.write_read(address, &[0xD0], &mut chip_id).is_ok()
            && (chip_id[0] == 0x60 || chip_id[0] == 0x58)
        {
            return Some((address, chip_id[0]));
        }
    }

    None
}

fn float_to_i16_tenths(value: f32) -> i16 {
    let scaled = value * 10.0;
    if scaled.is_sign_negative() {
        (scaled - 0.5) as i16
    } else {
        (scaled + 0.5) as i16
    }
}

fn float_to_u16_tenths(value: f32) -> u16 {
    if value.is_nan() || value <= 0.0 {
        0
    } else {
        (value * 10.0 + 0.5) as u16
    }
}

fn float_to_u32(value: f32) -> u32 {
    if value.is_nan() || value <= 0.0 {
        0
    } else {
        (value + 0.5) as u32
    }
}
