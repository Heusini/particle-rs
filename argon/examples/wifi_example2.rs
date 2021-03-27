#![no_main]
#![no_std]

extern crate embedded_hal_spy;
extern crate panic_halt;

use at_commands::builder::CommandBuilder;
use core::fmt::Write;
use cortex_m_rt::entry;
use nrf52840_hal::gpio::Level;
use nrf52840_hal::uarte;
use nrf52840_hal::Delay;
use nrf52840_hal::Timer;
use particle_argon::{prelude::*, Board};

// simple spi example from nrf52 example: https://github.com/nrf-rs/nrf-hal/blob/master/examples/spi-demo/src/main.rs
#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();

    let pins = uarte::Pins {
        txd: board.pins.tx.into_push_pull_output(Level::Low).degrade(),
        rxd: board.pins.rx.into_floating_input().degrade(),
        cts: None,
        rts: None,
    };

    let mut bootmode = board.pins.bootmode.into_push_pull_output(Level::Low);
    // let mut wifi_en = board.pins.wifi_en.into_push_pull_output(Level::Low);

    let mut wifi_en = board.pins.wifi_en.into_open_drain_output(
        nrf52840_hal::gpio::OpenDrainConfig::Standard0Disconnect1,
        Level::Low,
    );

    wifi_en.set_low().unwrap();

    let wifi_pins = uarte::Pins {
        txd: board.pins.esptx.into_push_pull_output(Level::Low).degrade(),
        rxd: board.pins.esprx.into_floating_input().degrade(),
        cts: None,
        rts: None,
        // cts: Some(board.pins.cts.into_floating_input().degrade()),
        // rts: Some(board.pins.rts.into_push_pull_output(Level::Low).degrade()),
    };
    let mut t = Timer::new(board.TIMER0);
    let mut delay = Delay::new(board.SYST);

    let parity = uarte::Parity::EXCLUDED;
    let baudrate = uarte::Baudrate::BAUD115200;
    let wifi_baudrate = uarte::Baudrate::BAUD921600;
    let mut uart = uarte::Uarte::new(board.UARTE0, pins, parity, baudrate);
    let mut wifi_uart = uarte::Uarte::new(board.UARTE1, wifi_pins, parity, wifi_baudrate);

    let mut buffer: [u8; 16] = [0; 16];
    let mut read_buf: [u8; 16] = [0; 16];

    bootmode.set_high().unwrap();
    delay.delay_ms(100_u32);
    wifi_en.set_high().unwrap();
    delay.delay_ms(100_u32);

    loop {
        uart.write_str("Writing to WIFI UART: AT\r\n").unwrap();
        let write_result = wifi_uart.write_str("AT\r\n");
        uart.write_fmt(format_args!("Write Result: {:?}", write_result))
            .unwrap();

        let read_result = match wifi_uart.read_timeout(&mut read_buf, &mut t, 2_000_001_u32) {
            Ok(_) => 128,
            Err(uarte::Error::Timeout(n)) => n,
            Err(uarte::Error::TxBufferTooLong) => 200,
            Err(uarte::Error::RxBufferTooLong) => 201,
            Err(uarte::Error::BufferNotInRAM) => 202,
            Err(uarte::Error::Transmit) => 203,
            Err(uarte::Error::Receive) => 204,
        };

        uart.write_fmt(format_args!("read bytes: {}\r\n", read_result))
            .unwrap();
        uart.write_fmt(format_args!("readbuffer: {:?}\r\n", &read_buf))
            .unwrap();
    }
}
