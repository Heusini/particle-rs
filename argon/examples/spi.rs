#![no_main]
#![no_std]

extern crate embedded_hal_spy;
extern crate panic_halt;

use embedded_hal::blocking::spi::Transfer;

use core::fmt::Write;
use cortex_m_rt::entry;
use nrf52840_hal::gpio::Level;
use nrf52840_hal::uarte;
use nrf52840_hal::Delay;
use nrf52840_hal::Spim;
use particle_argon::{prelude::*, Board};

// simple spi example from nrf52 example: https://github.com/nrf-rs/nrf-hal/blob/master/examples/spi-demo/src/main.rs
#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    // chipselect not needed here but needed for spi.transfer() functions
    //let cs = board.pins.a5.into_push_pull_output(Level::Low);

    let pins = uarte::Pins {
        txd: board.pins.tx.into_push_pull_output(Level::Low).degrade(),
        rxd: board.pins.rx.into_floating_input().degrade(),
        cts: None,
        rts: None,
    };

    let mut uart = uarte::Uarte::new(
        board.UARTE0,
        pins,
        uarte::Parity::EXCLUDED,
        uarte::Baudrate::BAUD115200,
    );

    let pins = nrf52840_hal::spim::Pins {
        sck: board.pins.sck.into_push_pull_output(Level::Low).degrade(),
        miso: Some(board.pins.miso.degrade()),
        mosi: Some(board.pins.mosi.into_push_pull_output(Level::Low).degrade()),
    };

    let mut spi = Spim::new(
        board.SPIM1,
        pins,
        nrf52840_hal::spim::Frequency::K125,
        nrf52840_hal::spim::MODE_0,
        0,
    );

    let message = b"Hello Welt!";

    let mut tx_buffer = *message;
    let mut rx_buffer = [0u8; 256];
    // let reference_data = "Hello World!\n";

    let mut delay = Delay::new(board.SYST);
    let mut led = board.pins.d7.into_push_pull_output(Level::Low);

    // let mut eh_spi = embedded_hal_spy::new(spi, |_| {});
    // use embedded_hal::blocking::spi::Write;
    uart.write_str("hello\r\n").unwrap();

    let mut cs = board.pins.a5.into_push_pull_output(Level::Low).degrade();

    loop {
        match spi.transfer_split_uneven(&mut cs, &mut tx_buffer, &mut rx_buffer) {
            Ok(_) => {
                uart.write(&rx_buffer).unwrap();
                uart.write_str("\r\n");
            }
            Err(_) => led.set_high().unwrap(),
        }
        // cs.set_high();
    }
}
