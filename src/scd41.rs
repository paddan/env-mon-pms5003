use embedded_hal::{delay::DelayNs, i2c::I2c};
use scd4x::{types::SensorData, Error, Scd4x};

pub const CO2_OUTDOOR_REFERENCE_PPM: u16 = 425;

#[derive(Clone, Copy)]
pub struct Scd41Reading {
    pub co2_ppm: u16,
    pub temperature_c_x10: i16,
    pub humidity_pct_x10: u16,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Co2Category {
    CatI,
    CatII,
    CatIIIorIV,
    AboveStandard,
}

impl Scd41Reading {
    pub fn from_sensor_data(data: SensorData) -> Self {
        Self {
            co2_ppm: data.co2,
            temperature_c_x10: float_to_i16_tenths(data.temperature),
            humidity_pct_x10: float_to_u16_tenths(data.humidity),
        }
    }
}

pub fn co2_category(co2_ppm: u16) -> Co2Category {
    let relative_ppm = co2_ppm.saturating_sub(CO2_OUTDOOR_REFERENCE_PPM);

    if relative_ppm <= 550 {
        Co2Category::CatI
    } else if relative_ppm <= 800 {
        Co2Category::CatII
    } else if relative_ppm <= 1350 {
        Co2Category::CatIIIorIV
    } else {
        Co2Category::AboveStandard
    }
}

pub fn co2_category_label_sv(co2_ppm: Option<u16>) -> &'static str {
    match co2_ppm.map(co2_category) {
        Some(Co2Category::CatI) => "Kat I",
        Some(Co2Category::CatII) => "Kat II",
        Some(Co2Category::CatIIIorIV) => "Kat III-IV",
        Some(Co2Category::AboveStandard) => "Over std",
        None => "--",
    }
}

pub fn init_scd41<I2C, D, E>(sensor: &mut Scd4x<I2C, D>) -> Result<u64, Error<E>>
where
    I2C: I2c<Error = E>,
    D: DelayNs,
{
    let serial = sensor.serial_number()?;
    sensor.start_periodic_measurement()?;
    Ok(serial)
}

pub fn read_measurement_if_ready<I2C, D, E>(
    sensor: &mut Scd4x<I2C, D>,
) -> Result<Option<Scd41Reading>, Error<E>>
where
    I2C: I2c<Error = E>,
    D: DelayNs,
{
    if !sensor.data_ready_status()? {
        return Ok(None);
    }

    let measurement = sensor.measurement()?;
    Ok(Some(Scd41Reading::from_sensor_data(measurement)))
}

pub fn apply_pressure_compensation<I2C, D, E>(
    sensor: &mut Scd4x<I2C, D>,
    pressure_pa: u32,
) -> Result<(), Error<E>>
where
    I2C: I2c<Error = E>,
    D: DelayNs,
{
    let pressure_hpa = ((pressure_pa + 50) / 100).min(u16::MAX as u32) as u16;
    sensor.set_ambient_pressure(pressure_hpa)
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
