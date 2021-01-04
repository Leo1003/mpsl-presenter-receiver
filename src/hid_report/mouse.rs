use usbd_hid::descriptor::{gen_hid_descriptor, generator_prelude::*};

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = MOUSE) = {
        (collection = PHYSICAL, usage = POINTER) = {
            (usage_page = BUTTON, usage_min = BUTTON_1, usage_max = BUTTON_3) = {
                #[packed_bits 3] #[item_settings data,variable,absolute] buttons=input;
            };
            (usage_page = GENERIC_DESKTOP,) = {
                (usage = X,) = {
                    #[item_settings data,variable,relative] x=input;
                };
                (usage = Y,) = {
                    #[item_settings data,variable,relative] y=input;
                };
                (usage = 0x38,) = {
                    #[item_settings data,variable,relative] wheel=input;
                };
            };
        };
    }
)]
#[derive(Default)]
pub struct MouseReport {
    pub buttons: u8,
    pub x: i16,
    pub y: i16,
    pub wheel: i8,
}

#[allow(dead_code)]
impl MouseReport {
    pub fn with_buttons(buttons: MouseButtons) -> Self {
        Self {
            buttons: buttons.bits(),
            ..Self::default()
        }
    }

    pub fn with_position(x: i16, y: i16) -> Self {
        Self {
            x,
            y,
            ..Self::default()
        }
    }

    pub fn with_wheel(wheel: i8) -> Self {
        Self {
            wheel,
            ..Self::default()
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct MouseButtons: u8 {
        const L_BUTTON = 0b00000001;
        const R_BUTTON = 0b00000010;
        const M_BUTTON = 0b00000100;
    }
}
