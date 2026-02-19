use esp_hal::{
    delay::Delay,
    uart::{TxError, Uart},
    Blocking,
};
use esp_println::println;

const PMS_FRAME_SIZE: usize = 32;
const PMS_START_1: u8 = 0x42;
const PMS_START_2: u8 = 0x4D;
const PMS_EXPECTED_PAYLOAD_LEN: u16 = 28;

pub const PMS_WAKE_CMD: [u8; 7] = [0x42, 0x4D, 0xE4, 0x00, 0x01, 0x01, 0x74];
pub const PMS_ACTIVE_MODE_CMD: [u8; 7] = [0x42, 0x4D, 0xE1, 0x00, 0x01, 0x01, 0x71];

// Keep full PMS frame fields parsed even if UI currently shows only ATM PM values.
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Pms5003Reading {
    pub pm1_0_cf1: u16,
    pub pm2_5_cf1: u16,
    pub pm10_cf1: u16,
    pub pm1_0_atm: u16,
    pub pm2_5_atm: u16,
    pub pm10_atm: u16,
    pub particles_0_3um: u16,
    pub particles_0_5um: u16,
    pub particles_1_0um: u16,
    pub particles_2_5um: u16,
    pub particles_5_0um: u16,
    pub particles_10um: u16,
}

pub struct PmsParser {
    frame: [u8; PMS_FRAME_SIZE],
    frame_index: usize,
}

impl PmsParser {
    pub const fn new() -> Self {
        Self {
            frame: [0; PMS_FRAME_SIZE],
            frame_index: 0,
        }
    }

    pub fn process_chunk(&mut self, chunk: &[u8]) -> Option<Pms5003Reading> {
        let mut latest = None;

        for &byte in chunk {
            match self.frame_index {
                0 => {
                    if byte == PMS_START_1 {
                        self.frame[0] = byte;
                        self.frame_index = 1;
                    }
                }
                1 => {
                    if byte == PMS_START_2 {
                        self.frame[1] = byte;
                        self.frame_index = 2;
                    } else if byte == PMS_START_1 {
                        self.frame[0] = byte;
                        self.frame_index = 1;
                    } else {
                        self.frame_index = 0;
                    }
                }
                _ => {
                    self.frame[self.frame_index] = byte;
                    self.frame_index += 1;

                    if self.frame_index == PMS_FRAME_SIZE {
                        self.frame_index = 0;
                        if let Some(reading) = Pms5003Reading::from_frame(&self.frame) {
                            latest = Some(reading);
                        }
                    }
                }
            }
        }

        latest
    }
}

impl Pms5003Reading {
    fn from_frame(frame: &[u8; PMS_FRAME_SIZE]) -> Option<Self> {
        if frame[0] != PMS_START_1 || frame[1] != PMS_START_2 {
            return None;
        }

        if read_u16_be(frame, 2) != PMS_EXPECTED_PAYLOAD_LEN {
            return None;
        }

        let expected_checksum = read_u16_be(frame, 30);
        let calculated_checksum = frame[..30]
            .iter()
            .fold(0u16, |sum, byte| sum.wrapping_add(*byte as u16));

        if expected_checksum != calculated_checksum {
            return None;
        }

        Some(Self {
            pm1_0_cf1: read_u16_be(frame, 4),
            pm2_5_cf1: read_u16_be(frame, 6),
            pm10_cf1: read_u16_be(frame, 8),
            pm1_0_atm: read_u16_be(frame, 10),
            pm2_5_atm: read_u16_be(frame, 12),
            pm10_atm: read_u16_be(frame, 14),
            particles_0_3um: read_u16_be(frame, 16),
            particles_0_5um: read_u16_be(frame, 18),
            particles_1_0um: read_u16_be(frame, 20),
            particles_2_5um: read_u16_be(frame, 22),
            particles_5_0um: read_u16_be(frame, 24),
            particles_10um: read_u16_be(frame, 26),
        })
    }
}

fn read_u16_be(buf: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([buf[offset], buf[offset + 1]])
}

pub fn write_all(uart: &mut Uart<'_, Blocking>, buf: &[u8]) -> Result<(), TxError> {
    let mut written = 0;
    while written < buf.len() {
        written += uart.write(&buf[written..])?;
    }
    uart.flush()?;
    Ok(())
}

pub fn send_pms_command(
    uart: &mut Uart<'_, Blocking>,
    delay: &mut Delay,
    command: &[u8],
    command_name: &str,
) -> bool {
    for attempt in 1..=3 {
        match write_all(uart, command) {
            Ok(()) => {
                println!("PMS command sent: {} (attempt {}/3)", command_name, attempt);
                return true;
            }
            Err(_) => {
                println!(
                    "PMS command failed: {} (attempt {}/3)",
                    command_name, attempt
                );
                delay.delay_millis(100);
            }
        }
    }
    false
}
