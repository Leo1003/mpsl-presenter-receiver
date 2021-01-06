use crate::command::Commands;
use crate::hid_report::*;
use crate::{USB_HID_CURSOR, USB_HID_KBD, USB_HID_MOUSE};
use cortex_m::interrupt::free;

#[derive(Debug)]
pub struct App {
    mouse_pressed: MouseButtons,
    keys_pressed: [u8; 6],
    modifiers: KeyboardModifiers,
}

impl App {
    pub fn new() -> Self {
        Self {
            mouse_pressed: MouseButtons::empty(),
            keys_pressed: [0u8; 6],
            modifiers: KeyboardModifiers::empty(),
        }
    }

    pub fn process_cmd(&mut self, cmd: Commands) {
        match cmd {
            Commands::MouseDown(btn) => {
                self.mouse_pressed |= btn;
                send_mouse_report(0, 0, self.mouse_pressed);
            }
            Commands::MouseUp(btn) => {
                self.mouse_pressed -= btn;
                send_mouse_report(0, 0, self.mouse_pressed);
            }
            Commands::KeyDown(key) => {
                if KeyboardModifiers::is_modifier(key) {
                    self.modifiers |= KeyboardModifiers::from_keycode(key);
                } else {
                    self.add_key_pressed(key);
                }
                send_kbd_report(self.modifiers, &self.keys_pressed);
            }
            Commands::KeyUp(key) => {
                if KeyboardModifiers::is_modifier(key) {
                    self.modifiers -= KeyboardModifiers::from_keycode(key);
                } else {
                    self.remove_key_pressed(key);
                }
                send_kbd_report(self.modifiers, &self.keys_pressed);
            }
            Commands::AbsMove(x, y) => {
                send_cursor_report(x, y);
            }
            Commands::RelMove(x, y) => {
                send_mouse_report(x, y, self.mouse_pressed);
            }
            Commands::Wheel(w) => {
                send_wheel_report(w, self.mouse_pressed);
            }
        }
    }

    fn add_key_pressed(&mut self, key: u8) {
        for slot in &mut self.keys_pressed {
            if key == *slot {
                // Duplicate key
                return;
            }
            if *slot == 0x00 {
                *slot = key;
                return;
            }
        }
    }

    fn remove_key_pressed(&mut self, key: u8) {
        let mut idx = None;
        for (i, slot) in self.keys_pressed.iter().enumerate() {
            if key == *slot {
                // Key found
                idx = Some(i);
                break;
            }
            if *slot == 0x00 {
                // End of key slots
                return;
            }
        }
        if let Some(idx) = idx {
            if idx < 5 {
                self.keys_pressed.copy_within(idx + 1.., idx);
            }
            self.keys_pressed[5] = 0x00;
        }
    }
}

fn send_cursor_report(x: u16, y: u16) {
    free(|cs| {
        let mut usb_hid_cursor_ref = USB_HID_CURSOR.borrow(cs).borrow_mut();
        let usb_hid_cursor = usb_hid_cursor_ref.as_mut().unwrap();

        let report = CursorReport::with_position(x, y);

        debug!("Send report: {:?}", &report);

        if let Err(e) = usb_hid_cursor.push_input(&report) {
            error!("Cursor Report Error: {:?}", e);
        }
    });
}

fn send_mouse_report(x: i16, y: i16, btn: MouseButtons) {
    free(|cs| {
        let mut usb_hid_mouse_ref = USB_HID_MOUSE.borrow(cs).borrow_mut();
        let usb_hid_mouse = usb_hid_mouse_ref.as_mut().unwrap();

        let report = MouseReport {
            buttons: btn.bits(),
            x,
            y,
            wheel: 0,
        };

        debug!("Send report: {:?}", &report);

        if let Err(e) = usb_hid_mouse.push_input(&report) {
            error!("Mouse Report Error: {:?}", e);
        }
    });
}

fn send_wheel_report(wheel: i8, btn: MouseButtons) {
    free(|cs| {
        let mut usb_hid_mouse_ref = USB_HID_MOUSE.borrow(cs).borrow_mut();
        let usb_hid_mouse = usb_hid_mouse_ref.as_mut().unwrap();

        let report = MouseReport {
            buttons: btn.bits(),
            wheel,
            ..MouseReport::default()
        };

        debug!("Send report: {:?}", &report);

        if let Err(e) = usb_hid_mouse.push_input(&report) {
            error!("Wheel Report Error: {:?}", e);
        }
    });
}

fn send_kbd_report(modifiers: KeyboardModifiers, keys: &[u8; 6]) {
    free(|cs| {
        let mut usb_hid_kbd_ref = USB_HID_KBD.borrow(cs).borrow_mut();
        let usb_hid_kbd = usb_hid_kbd_ref.as_mut().unwrap();

        let report = KeyboardReport {
            modifier: modifiers.bits(),
            keycodes: *keys,
            leds: 0,
        };

        debug!("Send report: {:?}", &report);

        if let Err(e) = usb_hid_kbd.push_input(&report) {
            error!("Keyboard Report Error: {:?}", e);
        }
    });
}
