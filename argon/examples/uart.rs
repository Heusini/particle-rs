#![no_main]
#![no_std]

extern crate embedded_hal_spy;
extern crate panic_halt;

use cortex_m_rt::entry;
use nrf52840_hal::gpio::Level;
use nrf52840_hal::uarte;
use nrf52840_hal::Delay;
use particle_argon::{prelude::*, Board};

// simple spi example from nrf52 example: https://github.com/nrf-rs/nrf-hal/blob/master/examples/spi-demo/src/main.rs
#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    let tx = board.pins.tx.into_push_pull_output(Level::Low).degrade();
    let rx = board.pins.rx.into_floating_input().degrade();
    let pins = uarte::Pins {
        txd: tx,
        rxd: rx,
        cts: None,
        rts: None,
    };
    let parity = uarte::Parity::EXCLUDED;
    let baudrate = uarte::Baudrate::BAUD115200;
    let mut uart = uarte::Uarte::new(board.UARTE0, pins, parity, baudrate);

    let mut delay = Delay::new(board.SYST);
    let mut led = board.pins.d7.into_push_pull_output(Level::Low);

    let _data = b"Hello World";
    let mut buf: [u8; 128] = [97; 128];
    buf[126] = b'\r';
    buf[127] = b'\n';

    loop {
        led.set_high().unwrap();
        delay.delay_ms(100_u32);
        let _x = uart.write(&buf);
        led.set_low().unwrap();
        delay.delay_ms(100_u32);
    }
}
