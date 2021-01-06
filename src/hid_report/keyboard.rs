use usbd_hid::descriptor::{gen_hid_descriptor, generator_prelude::*};

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = KEYBOARD) = {
        (usage_page = KEYBOARD, usage_min = 0xE0, usage_max = 0xE7) = {
            #[packed_bits 8] #[item_settings data,variable,absolute] modifier=input;
        };
        (usage_page = LEDS, usage_min = 0x01, usage_max = 0x05) = {
            #[packed_bits 5] #[item_settings data,variable,absolute] leds=output;
        };
        (usage_page = KEYBOARD, usage_min = 0x00, usage_max = 0xA4) = {
            #[item_settings data,array,absolute] keycodes=input;
        };
    }
)]
pub struct KeyboardReport {
    pub modifier: u8,
    pub leds: u8,
    pub keycodes: [u8; 6],
}

bitflags! {
    #[derive(Default)]
    pub struct KeyboardModifiers: u8 {
        const L_CTRL =  0b00000001;
        const L_SHIFT = 0b00000010;
        const L_ALT =   0b00000100;
        const L_META =  0b00001000;
        const R_CTRL =  0b00010000;
        const R_SHIFT = 0b00100000;
        const R_ALT =   0b01000000;
        const R_META =  0b10000000;
    }
}

impl KeyboardModifiers {
    pub const fn is_modifier(code: u8) -> bool {
        (code >= 0xE0) && (code <= 0xE7)
    }

    pub const fn from_keycode(code: u8) -> Self {
        match code {
            0xE0 => KeyboardModifiers::L_CTRL,
            0xE1 => KeyboardModifiers::L_SHIFT,
            0xE2 => KeyboardModifiers::L_ALT,
            0xE3 => KeyboardModifiers::L_META,
            0xE4 => KeyboardModifiers::R_CTRL,
            0xE5 => KeyboardModifiers::R_SHIFT,
            0xE6 => KeyboardModifiers::R_ALT,
            0xE7 => KeyboardModifiers::R_META,
            _ => KeyboardModifiers::empty(),
        }
    }
}
