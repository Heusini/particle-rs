#![no_main]
#![no_std]

extern crate embedded_hal_spy;
extern crate panic_halt;

use core::fmt::Write;
use cortex_m_rt::entry;
use nrf52840_hal::gpio::Level;
use nrf52840_hal::uarte;
use nrf52840_hal::Delay;
use particle_argon::Board;

// simple spi example from nrf52 example: https://github.com/nrf-rs/nrf-hal/blob/master/examples/spi-demo/src/main.rs
#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();
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

    let mut delay = Delay::new(board.SYST);

    if board.WIFI.on(&mut delay).is_ok() {
        uart.write_str("Wifi is online\r\n").unwrap();
    } else {
        uart.write_str("Wifi is offline\r\n").unwrap();
    }

    uart.write_str("scanning\r\n");
    let buf = board.WIFI.scan().unwrap();

    uart.write(&buf);
    uart.write_str("Finished Writing Buffer\r\n");

    uart.write_str("Connecting to Wifi\r\n");
    let buf = board.WIFI.connect("devolo-846", "WGinet19").unwrap();
    uart.write(&buf);
    uart.write_str("Finished Connecting to Wifi\r\n");

    loop {}
}
