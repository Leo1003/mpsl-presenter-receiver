use core::{
    convert::Infallible,
    mem::{replace, zeroed},
};
use embedded_nrf24l01::{Configuration, Device, RxMode, StandbyMode, TxMode, NRF24L01};
use stm32l4::stm32l4x6::SPI1;
use stm32l4xx_hal::{gpio::*, spi::Spi};

pub type NRF24Device = NRF24L01<
    Infallible,
    PB9<Output<PushPull>>,
    PB8<Output<PushPull>>,
    Spi<
        SPI1,
        (
            PB3<Alternate<AF5, Input<Floating>>>,
            PB4<Alternate<AF5, Input<Floating>>>,
            PB5<Alternate<AF5, Input<Floating>>>,
        ),
    >,
>;

#[derive(Debug)]
pub enum NRF24Mode<D: Device> {
    Standby(StandbyMode<D>),
    Tx(TxMode<D>),
    Rx(RxMode<D>),
}

#[allow(dead_code)]
#[allow(clippy::wrong_self_convention)]
impl<D: Device> NRF24Mode<D> {
    pub fn to_standby(&mut self) -> &mut StandbyMode<D> {
        match self {
            NRF24Mode::Standby(_) => (),
            NRF24Mode::Tx(d) => {
                // Safety: The value is temporary taken
                let nrf = replace(d, unsafe { zeroed() });
                *self = NRF24Mode::Standby(nrf.standby().ok().unwrap());
            }
            NRF24Mode::Rx(d) => {
                // Safety: The value is temporary taken
                let nrf = replace(d, unsafe { zeroed() });
                *self = NRF24Mode::Standby(nrf.standby());
            }
        };
        self.standby_ref().unwrap()
    }

    pub fn to_tx(&mut self) -> &mut TxMode<D> {
        match self {
            NRF24Mode::Standby(d) => {
                // Safety: The value is temporary taken
                let nrf = replace(d, unsafe { zeroed() });
                *self = NRF24Mode::Tx(nrf.tx().ok().unwrap());
            }
            NRF24Mode::Tx(_) => (),
            NRF24Mode::Rx(d) => {
                // Safety: The value is temporary taken
                let nrf = replace(d, unsafe { zeroed() });
                *self = NRF24Mode::Tx(nrf.standby().tx().ok().unwrap());
            }
        };
        self.tx_ref().unwrap()
    }

    pub fn to_rx(&mut self) -> &mut RxMode<D> {
        match self {
            NRF24Mode::Standby(d) => {
                // Safety: The value is temporary taken
                let nrf = replace(d, unsafe { zeroed() });
                *self = NRF24Mode::Rx(nrf.rx().ok().unwrap());
            }
            NRF24Mode::Tx(d) => {
                // Safety: The value is temporary taken
                let nrf = replace(d, unsafe { zeroed() });
                *self = NRF24Mode::Rx(nrf.standby().ok().unwrap().rx().ok().unwrap());
            }
            NRF24Mode::Rx(_) => (),
        };
        self.rx_ref().unwrap()
    }

    pub fn standby_ref(&mut self) -> Option<&mut StandbyMode<D>> {
        if let NRF24Mode::Standby(dev_ref) = self {
            Some(dev_ref)
        } else {
            None
        }
    }

    pub fn tx_ref(&mut self) -> Option<&mut TxMode<D>> {
        if let NRF24Mode::Tx(dev_ref) = self {
            Some(dev_ref)
        } else {
            None
        }
    }

    pub fn rx_ref(&mut self) -> Option<&mut RxMode<D>> {
        if let NRF24Mode::Rx(dev_ref) = self {
            Some(dev_ref)
        } else {
            None
        }
    }

    pub fn configuration_ref(&self) -> &dyn Configuration<Inner = D> {
        match self {
            NRF24Mode::Standby(d) => d,
            NRF24Mode::Tx(d) => d,
            NRF24Mode::Rx(d) => d,
        }
    }

    pub fn configuration_mut(&mut self) -> &mut dyn Configuration<Inner = D> {
        match self {
            NRF24Mode::Standby(d) => d,
            NRF24Mode::Tx(d) => d,
            NRF24Mode::Rx(d) => d,
        }
    }
}
