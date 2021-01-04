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
