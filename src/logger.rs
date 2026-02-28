use log::{LevelFilter, Log, Metadata, Record};

// ESP32-C3 USB_DEVICE (USB Serial/JTAG) peripheral.
// Base: 0x60043000. EP1_CONF register at offset 0x04.
// Bit 1 — SERIAL_IN_EP_DATA_FREE (RO): set when a USB host is connected and
// the serial-in endpoint FIFO has space. Clear when no cable is attached or
// when the host is not reading (buffer full). Used by esp-hal's UsbSerialJtag
// driver as the "is attached" check before attempting a write.
const EP1_CONF: *const u32 = 0x6004_3004 as *const u32;
const SERIAL_IN_EP_DATA_FREE: u32 = 1 << 1;

fn usb_connected() -> bool {
    // Safety: single-word read of a read-only memory-mapped register on ESP32-C3.
    unsafe { (core::ptr::read_volatile(EP1_CONF) & SERIAL_IN_EP_DATA_FREE) != 0 }
}

struct UsbLogger;

impl Log for UsbLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        usb_connected()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            esp_println::println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: UsbLogger = UsbLogger;

pub fn init(level: LevelFilter) {
    // Safety: called once at startup, before the main loop, on a single-core MCU.
    // No concurrent access is possible at this point.
    unsafe {
        log::set_logger_racy(&LOGGER).ok();
        log::set_max_level_racy(level);
    }
}
