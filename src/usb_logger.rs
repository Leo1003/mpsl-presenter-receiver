use crate::USB_SER;
use core::fmt::{self, Write};
use cortex_m::{asm, interrupt::free};
use log::{LevelFilter, Log, Metadata, Record};
use usb_device::UsbError;

#[derive(Clone, Debug)]
pub struct UsbLogger;

impl UsbLogger {
    pub fn init(&'static self) {
        log::set_logger(self).unwrap();
        log::set_max_level(LevelFilter::Trace);
    }
}

impl Write for UsbLogger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        free(|cs| {
            let mut usb_ser_ref = USB_SER
                .borrow(cs)
                .try_borrow_mut()
                .map_err(|_| fmt::Error)?;
            let usb_ser = usb_ser_ref.as_mut().unwrap();

            for data in s.as_bytes().chunks(64) {
                if let Err(e) = usb_ser.write(data) {
                    if matches!(e, UsbError::WouldBlock) {
                        return Err(fmt::Error);
                    }
                    while usb_ser.flush().is_err() {
                        asm::nop();
                    }
                    // Retry again
                    usb_ser.write(data).ok();
                }
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl Log for UsbLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut logger = self.clone();
            if let Some(module_path) = record.module_path_static() {
                write!(
                    &mut logger,
                    "[{}] {}: {}\r\n",
                    record.level(),
                    module_path,
                    record.args()
                )
                .ok();
            } else {
                write!(&mut logger, "[{}] {}\r\n", record.level(), record.args()).ok();
            }
        }
    }

    fn flush(&self) {
        free(|cs| {
            let mut usb_ser_ref = USB_SER.borrow(cs).borrow_mut();
            let usb_ser = usb_ser_ref.as_mut().unwrap();

            usb_ser.flush().ok();
        });
    }
}
