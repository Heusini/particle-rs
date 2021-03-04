#![no_main]
#![no_std]

extern crate embedded_hal_spy;
extern crate panic_halt;

use cortex_m_rt::entry;
use nrf52840_hal::gpio::Level;
use nrf52840_hal::Delay;
use nrf52840_hal::Spim;
use particle_argon::{prelude::*, Board};

// simple spi example from nrf52 example: https://github.com/nrf-rs/nrf-hal/blob/master/examples/spi-demo/src/main.rs
#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    // chipselect not needed here but needed for spi.transfer() functions
    //let cs = board.pins.a5.into_push_pull_output(Level::Low);

    let pins = nrf52840_hal::spim::Pins {
        sck: board.pins.sck.into_push_pull_output(Level::Low).degrade(),
        miso: Some(board.pins.miso.degrade()),
        mosi: Some(board.pins.mosi.into_push_pull_output(Level::Low).degrade()),
    };

    let spi = Spim::new(
        board.SPIM1,
        pins,
        nrf52840_hal::spim::Frequency::M2,
        nrf52840_hal::spim::MODE_0,
        0,
    );

    let reference_data = b"Hello World!\n";

    let mut delay = Delay::new(board.SYST);
    let mut led = board.pins.d7.into_push_pull_output(Level::Low);

    let mut eh_spi = embedded_hal_spy::new(spi, |_| {});
    use embedded_hal::blocking::spi::Write;

    loop {
        match eh_spi.write(reference_data) {
            Ok(_) => led.set_high().unwrap(),
            Err(_) => led.set_low().unwrap(),
        }
        delay.delay_ms(100_u32);
    }
}
