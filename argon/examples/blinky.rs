#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m_rt::entry;
use nrf52840_hal::gpio::Level;
use nrf52840_hal::Delay;
use particle_argon::{prelude::*, Board};

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();

    let mut timer = Delay::new(board.SYST);

    let mut led = board.pins.d7.into_push_pull_output(Level::Low);

    led.set_high().unwrap();

    loop {
        led.set_high().unwrap();
        timer.delay_ms(1000_u32);
        led.set_low().unwrap();
        timer.delay_ms(1000_u32);
    }
}
