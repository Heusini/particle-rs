use nrf52840_hal::gpio;
use nrf52840_hal::gpio::p0;
use nrf52840_hal::gpio::p1;

/// Maps the pins to the names printed on the device
pub struct Pins {
    pub rst: p0::P0_18<gpio::Input<gpio::Floating>>,
    pub mode: p0::P0_11<gpio::Input<gpio::Floating>>,
    pub a0: p0::P0_03<gpio::Input<gpio::Floating>>,
    pub a1: p0::P0_04<gpio::Input<gpio::Floating>>,
    pub a2: p0::P0_28<gpio::Input<gpio::Floating>>,
    pub a3: p0::P0_29<gpio::Input<gpio::Floating>>,
    pub a4: p0::P0_30<gpio::Input<gpio::Floating>>,
    pub a5: p0::P0_31<gpio::Input<gpio::Floating>>,
    pub sck: p1::P1_15<gpio::Input<gpio::Floating>>,
    pub mosi: p1::P1_13<gpio::Input<gpio::Floating>>,
    pub miso: p1::P1_14<gpio::Input<gpio::Floating>>,
    pub rx: p0::P0_08<gpio::Input<gpio::Floating>>,
    pub tx: p0::P0_06<gpio::Input<gpio::Floating>>,
    pub d0: p0::P0_26<gpio::Input<gpio::Floating>>,
    pub d1: p0::P0_27<gpio::Input<gpio::Floating>>,
    pub d2: p1::P1_01<gpio::Input<gpio::Floating>>,
    pub d3: p1::P1_02<gpio::Input<gpio::Floating>>,
    pub d4: p1::P1_08<gpio::Input<gpio::Floating>>,
    pub d5: p1::P1_10<gpio::Input<gpio::Floating>>,
    pub d6: p1::P1_11<gpio::Input<gpio::Floating>>,
    pub d7: p1::P1_12<gpio::Input<gpio::Floating>>,
    pub d8: p1::P1_03<gpio::Input<gpio::Floating>>,
}

impl Pins {
    pub fn new(p0: nrf52840_hal::pac::P0, p1: nrf52840_hal::pac::P1) -> Self {
        let pins0 = p0::Parts::new(p0);
        let pins1 = p1::Parts::new(p1);

        Self {
            rst: pins0.p0_18.into_floating_input(),
            mode: pins0.p0_11.into_floating_input(),
            a0: pins0.p0_03.into_floating_input(),
            a1: pins0.p0_04.into_floating_input(),
            a2: pins0.p0_28.into_floating_input(),
            a3: pins0.p0_29.into_floating_input(),
            a4: pins0.p0_30.into_floating_input(),
            a5: pins0.p0_31.into_floating_input(),
            sck: pins1.p1_15.into_floating_input(),
            mosi: pins1.p1_13.into_floating_input(),
            miso: pins1.p1_14.into_floating_input(),
            rx: pins0.p0_08.into_floating_input(),
            tx: pins0.p0_06.into_floating_input(),
            d0: pins0.p0_26.into_floating_input(),
            d1: pins0.p0_27.into_floating_input(),
            d2: pins1.p1_01.into_floating_input(),
            d3: pins1.p1_02.into_floating_input(),
            d4: pins1.p1_08.into_floating_input(),
            d5: pins1.p1_10.into_floating_input(),
            d6: pins1.p1_11.into_floating_input(),
            d7: pins1.p1_12.into_floating_input(),
            d8: pins1.p1_03.into_floating_input(),
        }
    }
}
