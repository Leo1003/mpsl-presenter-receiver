use crate::USB_SER;
use core::fmt;
use cortex_m::interrupt::free;

pub struct SerialWriter;

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        free(|cs| {
            let mut usb_ser_ref = USB_SER.borrow(cs).try_borrow_mut().map_err(|_| fmt::Error)?;
            let usb_ser = usb_ser_ref.as_mut().unwrap();

            usb_ser.write(s.as_bytes()).map_err(|_| fmt::Error)
        })?;
        Ok(())
    }
}
