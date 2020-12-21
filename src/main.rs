#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_semihosting as _;
use stm32l4xx_hal::otg_fs::{UsbBus, USB};
use stm32l4xx_hal::prelude::*;
use stm32l4xx_hal::rcc::{PllConfig, PllDivider, PllSource};
use stm32l4xx_hal::stm32::Peripherals;
use usb_device::prelude::*;

static mut EP_MEMORY: [u32; 320] = [0; 320];

fn enable_crs() {
    use stm32l4xx_hal::stm32::{CRS, RCC};

    let rcc = unsafe { &(*RCC::ptr()) };
    rcc.apb1enr1.modify(|_, w| w.crsen().set_bit());
    let crs = unsafe { &(*CRS::ptr()) };
    // Initialize clock recovery
    // Set autotrim enabled.
    crs.cr.modify(|_, w| w.autotrimen().set_bit());
    // Enable CR
    crs.cr.modify(|_, w| w.cen().set_bit());
}

/// Enables VddUSB power supply
fn enable_usb_pwr() {
    use stm32l4xx_hal::stm32::{PWR, RCC};

    // Enable PWR peripheral
    let rcc = unsafe { &(*RCC::ptr()) };
    rcc.apb1enr1.modify(|_, w| w.pwren().set_bit());

    // Enable VddUSB
    let pwr = unsafe { &(*PWR::ptr()) };
    pwr.cr2.modify(|_, w| w.usv().set_bit());
}

fn enable_pllq_48mhz() {
    use stm32l4xx_hal::stm32::RCC;

    let rcc = unsafe { &(*RCC::ptr()) };
    // PllQ = PllDivider::Div2
    rcc.pllcfgr.modify(|_, w| unsafe { w.pllq().bits(0x00) });
    // Enable PllQ
    rcc.pllcfgr.modify(|_, w| w.pllqen().set_bit());
    // Attach 48MHz clock onto PllQ
    rcc.ccipr.modify(|_, w| unsafe { w.clk48sel().bits(0x02) });
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    let clocks = rcc
        .cfgr
        .pll_source(PllSource::HSI16)
        .sysclk_with_pll(48.mhz(), PllConfig::new(2, 12, PllDivider::Div2))
        .hclk(48.mhz())
        .pclk1(24.mhz())
        .pclk2(24.mhz())
        .freeze(&mut flash.acr, &mut pwr);

    enable_pllq_48mhz();

    enable_crs();

    // disable Vddusb power isolation
    enable_usb_pwr();

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);

    let usb = USB {
        usb_global: dp.OTG_FS_GLOBAL,
        usb_device: dp.OTG_FS_DEVICE,
        usb_pwrclk: dp.OTG_FS_PWRCLK,
        pin_dm: gpioa
            .pa11
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
            .into_af10(&mut gpioa.moder, &mut gpioa.afrh),
        pin_dp: gpioa
            .pa12
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
            .into_af10(&mut gpioa.moder, &mut gpioa.afrh),
        hclk: clocks.hclk(),
    };
    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });

    loop {
        // your code goes here
    }
}
