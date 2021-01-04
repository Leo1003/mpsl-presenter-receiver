#![no_std]
#![no_main]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate cortex_m_rt;
#[macro_use]
extern crate log;

use core::cell::RefCell;

use cortex_m::interrupt::{free, Mutex};
use cortex_m::peripheral::NVIC;
use embedded_nrf24l01::{setup::*, Configuration, CrcMode, DataRate, NRF24L01};
use hid_report::{CursorReport, MouseReport};
use nrf24_mode::{NRF24Device, NRF24Mode};
use panic_semihosting as _;
use stm32l4xx_hal::{
    interrupt,
    otg_fs::{UsbBus, USB},
    prelude::*,
    rcc::{PllConfig, PllDivider, PllSource},
    spi::Spi,
    stm32::{Interrupt, Peripherals},
};
use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::hid_class::HIDClass;
use usbd_serial::SerialPort;

use usb_logger::UsbLogger;

const NRF24_ADDRESS: &[u8; 5] = b"\x2f\xa6\x37\x89\x73";
const NRF24_CHANNEL: u8 = 82;

static mut EP_MEMORY: [u32; 320] = [0; 320];

type MutexCell<T> = Mutex<RefCell<Option<T>>>;
type UsbType = UsbBus<USB>;

static mut USB_BUS: Option<UsbBusAllocator<UsbType>> = None;
static USB_DEV: MutexCell<UsbDevice<UsbType>> = Mutex::new(RefCell::new(None));
static USB_HID_CURSOR: MutexCell<HIDClass<UsbType>> = Mutex::new(RefCell::new(None));
static USB_HID_MOUSE: MutexCell<HIDClass<UsbType>> = Mutex::new(RefCell::new(None));
static USB_SER: MutexCell<SerialPort<UsbType>> = Mutex::new(RefCell::new(None));
static NRF24: MutexCell<NRF24Mode<NRF24Device>> = Mutex::new(RefCell::new(None));

mod hid_report;
mod nrf24_mode;
mod usb_logger;

static USB_LOGGER: UsbLogger = UsbLogger;

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

    // Set HCLK to 48MHz
    let clocks = rcc
        .cfgr
        .pll_source(PllSource::HSI16)
        .sysclk_with_pll(48.mhz(), PllConfig::new(2, 12, PllDivider::Div2))
        .hclk(48.mhz())
        .pclk1(24.mhz())
        .pclk2(24.mhz())
        .freeze(&mut flash.acr, &mut pwr);
    // Output 48MHz to USB clock source
    enable_pllq_48mhz();

    enable_crs();

    // disable Vddusb power isolation
    enable_usb_pwr();

    // Setup basic GPIO
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);
    let usr_btn = gpioc
        .pc13
        .into_pull_up_input(&mut gpioc.moder, &mut gpioc.pupdr);

    // Setup USB
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
    free(|_| unsafe {
        // Safety: Interrupt-free section
        USB_BUS = Some(usb_bus);
    });

    let mut action = CursorReport::with_position(2048, 950);

    free(|cs| {
        // Safety: Interrupt-free section
        USB_SER
            .borrow(cs)
            .replace(Some(SerialPort::new(unsafe { USB_BUS.as_ref().unwrap() })));
        USB_HID_CURSOR.borrow(cs).replace(Some(HIDClass::new(
            unsafe { USB_BUS.as_ref().unwrap() },
            CursorReport::desc(),
            100,
        )));
        /*
        USB_HID_MOUSE.borrow(cs).replace(Some(HIDClass::new(
            unsafe { USB_BUS.as_ref().unwrap() },
            MouseReport::desc(),
            100,
        )));*/
        USB_DEV.borrow(cs).replace(Some(
            UsbDeviceBuilder::new(
                unsafe { USB_BUS.as_ref().unwrap() },
                UsbVidPid(0x16c0, 0x0487),
            )
            .manufacturer("Leo")
            .product("Smart presenter")
            .serial_number("TEST0000")
            .build(),
        ))
    });

    USB_LOGGER.init();
    info!("USB initialized");

    // Setup NRF24L01
    let spi_sck = gpiob
        .pb3
        .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr)
        .into_af5(&mut gpiob.moder, &mut gpiob.afrl);
    let spi_miso = gpiob
        .pb4
        .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr)
        .into_af5(&mut gpiob.moder, &mut gpiob.afrl);
    let spi_mosi = gpiob
        .pb5
        .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr)
        .into_af5(&mut gpiob.moder, &mut gpiob.afrl);
    let spi = Spi::spi1(
        dp.SPI1,
        (spi_sck, spi_miso, spi_mosi),
        spi_mode(),
        clock_mhz(),
        clocks,
        &mut rcc.apb2,
    );
    let nrf24_ce = gpiob
        .pb9
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let nrf24_csn = gpiob
        .pb8
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut nrf24l01 =
        NRF24L01::new(nrf24_ce, nrf24_csn, spi).expect("Failed to initialize NRF24L01");

    nrf24l01
        .set_rf(&DataRate::R2Mbps, 3)
        .expect("Failed to set RF power");
    nrf24l01
        .set_frequency(NRF24_CHANNEL)
        .expect("Failed to set channel");
    nrf24l01
        .set_crc(CrcMode::OneByte)
        .expect("Failed to set CRC setting");
    nrf24l01
        .set_auto_ack(&[true, true, true, true, true, true])
        .expect("Failed to enable auto ACK");
    nrf24l01
        .set_rx_addr(0, NRF24_ADDRESS)
        .expect("Failed to set Rx 0 address");
    nrf24l01
        .set_pipes_rx_lengths(&[None; 6])
        .expect("Failed to set payload length");
    nrf24l01
        .set_pipes_rx_enable(&[true, false, false, false, false, false])
        .expect("Failed to enable Rxs");
    nrf24l01
        .set_interrupt_mask(false, true, true)
        .expect("Failed to set interrupt mask");

    free(|cs| {
        NRF24
            .borrow(cs)
            .replace(Some(NRF24Mode::Rx(nrf24l01.rx().unwrap())));
    });

    info!("NRF24L01 initialized");

    unsafe {
        NVIC::unmask(Interrupt::OTG_FS);
        NVIC::unmask(Interrupt::EXTI9_5);
    }

    let mut btn_state = false;

    let mut add_off: u16 = 1;
    loop {
        if usr_btn.is_low().unwrap() {
            btn_state = true;
        } else if btn_state && usr_btn.is_high().unwrap() {
            btn_state = false;
            free(|cs| {
                let mut usb_hid_cursor_ref = USB_HID_CURSOR.borrow(cs).borrow_mut();
                let usb_hid_cursor = usb_hid_cursor_ref.as_mut().unwrap();

                usb_hid_cursor.push_input(&action).ok();
                debug!("action: {:?}", &action);
            });

            action.x = action.x.wrapping_add(add_off.wrapping_mul(2));
            action.y = action.y.wrapping_add(add_off.wrapping_add(152));
        }

        free(|cs| {
            let mut nrf24l01_ref = NRF24.borrow(cs).borrow_mut();
            let nrf24l01 = nrf24l01_ref.as_mut().unwrap();

            nrf24l01.configuration_mut().clear_interrupts().ok();

            let nrf24l01_rx = nrf24l01.to_rx();
            while nrf24l01_rx.can_read().unwrap().is_some() {
                let packet = nrf24l01_rx.read().unwrap();

                match core::str::from_utf8(packet.as_ref()) {
                    Ok(s) => debug!("Received string: {}", s),
                    Err(_) => debug!("Received binary: {:?}", packet.as_ref()),
                }
            }
        });

        add_off = add_off.wrapping_add(2);
    }
}

#[interrupt]
fn OTG_FS() {
    free(|cs| {
        let mut usb_dev_ref = USB_DEV.borrow(cs).borrow_mut();
        let usb_dev = usb_dev_ref.as_mut().unwrap();
        let mut usb_hid_cursor_ref = USB_HID_CURSOR.borrow(cs).borrow_mut();
        let usb_hid_cursor = usb_hid_cursor_ref.as_mut().unwrap();
        // let mut usb_hid_mouse_ref = USB_HID_MOUSE.borrow(cs).borrow_mut();
        // let usb_hid_mouse = usb_hid_mouse_ref.as_mut().unwrap();
        let mut usb_ser_ref = USB_SER.borrow(cs).borrow_mut();
        let usb_ser = usb_ser_ref.as_mut().unwrap();

        let mut buf = [0u8; 64];
        if usb_dev.poll(&mut [usb_ser, usb_hid_cursor/* , usb_hid_mouse*/]) {
            usb_ser.read(&mut buf).ok();
        }
    });
}
