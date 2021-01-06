use super::MouseButtons;
use usbd_hid::descriptor::{gen_hid_descriptor, generator_prelude::*};

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = MOUSE) = {
        (collection = PHYSICAL, usage = POINTER) = {
            (usage_page = BUTTON, usage_min = BUTTON_1, usage_max = BUTTON_3) = {
                #[packed_bits 3] #[item_settings data,variable,absolute] buttons=input;
            };
            (usage_page = GENERIC_DESKTOP,) = {
                (usage = X,) = {
                    #[item_settings data,variable,absolute] x=input;
                };
                (usage = Y,) = {
                    #[item_settings data,variable,absolute] y=input;
                };
            };
        };
    }
)]
#[derive(Default)]
pub struct CursorReport {
    pub buttons: u8,
    pub x: u16,
    pub y: u16,
}

#[allow(dead_code)]
impl CursorReport {
    pub fn with_buttons(buttons: MouseButtons) -> Self {
        Self {
            buttons: buttons.bits(),
            ..Self::default()
        }
    }

    pub fn with_position(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            ..Self::default()
        }
    }
}
