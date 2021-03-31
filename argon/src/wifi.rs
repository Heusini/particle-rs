use nrf52840_hal::gpio;
use nrf52840_hal::gpio::p0;
use nrf52840_hal::gpio::Output;
use nrf52840_hal::gpio::Pin;
use nrf52840_hal::pac::UARTE1;
use nrf52840_hal::prelude::*;
use nrf52840_hal::uarte;
use nrf52840_hal::Delay;

pub struct WIFI {
    uarte: uarte::Uarte<UARTE1>,
    wifi_en: p0::P0_16<gpio::Output<gpio::OpenDrain>>,
    bootmode: p0::P0_24<gpio::Output<gpio::PushPull>>,
    delay: Delay,
}

impl WIFI {
    pub fn new(
        uarte: uarte::Uarte<UARTE1>,
        wifi_en: p0::P0_16<gpio::Output<gpio::OpenDrain>>,
        bootmode: p0::P0_24<gpio::Output<gpio::PushPull>>,
        delay: Delay,
    ) -> Self {
        WIFI {
            uarte,
            wifi_en,
            bootmode,
            delay,
        }
    }

    pub fn on(&mut self) -> Result<(), <Pin<Output<gpio::OpenDrain>> as OutputPin>::Error> {
        self.wifi_en.set_low()?;
        self.delay.delay_ms(100_u32);
        self.bootmode.set_high()?;
        self.delay.delay_ms(100_u32);
        self.wifi_en.set_high()?;
        self.delay.delay_ms(100_u32);

        Ok(())
    }
}
