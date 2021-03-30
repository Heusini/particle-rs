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
    let tx = board.pins.tx.into_push_pull_output(Level::Low).degrade();
    let rx = board.pins.rx.into_floating_input().degrade();
    let pins = uarte::Pins {
        txd: tx,
        rxd: rx,
        cts: None,
        rts: None,
    };

    let mut delay = Delay::new(board.SYST);
    let mut bootmode = board.pins.bootmode.into_push_pull_output(Level::Low);
    // let mut wifi_en = board.pins.wifi_en.into_push_pull_output(Level::Low);

    let mut wifi_en = board.pins.wifi_en.into_open_drain_output(
        nrf52840_hal::gpio::OpenDrainConfig::Standard0Disconnect1,
        Level::High,
    );

    wifi_en.set_low().unwrap();
    delay.delay_ms(5000_u32);

    // let cts = board.pins.cts.into_floating_input().degrade();
    // let mut rts = board.pins.rts.into_push_pull_output(Level::High).degrade();

    // let mut tx = board.pins.esptx.into_push_pull_output(Level::Low);
    // let mut rx = board.pins.esprx.into_floating_input();
    //
    let wifi_pins = uarte::Pins {
        txd: board
            .pins
            .esprx
            .into_push_pull_output(Level::High)
            .degrade(),
        rxd: board.pins.esptx.into_floating_input().degrade(),
        // cts: None,
        // rts: None,
        cts: Some(board.pins.rts.into_floating_input().degrade()),
        rts: Some(board.pins.cts.into_push_pull_output(Level::High).degrade()),
    };
    let mut t = Timer::new(board.TIMER0);

    let parity = uarte::Parity::EXCLUDED;
    let baudrate = uarte::Baudrate::BAUD115200;
    let wifi_baudrate = uarte::Baudrate::BAUD921600;
    let mut uart = uarte::Uarte::new(board.UARTE0, pins, parity, baudrate);
    let mut wifi_uart = uarte::Uarte::new(board.UARTE1, wifi_pins, parity, wifi_baudrate);

    let mut led = board.pins.d7.into_push_pull_output(Level::High);

    let mut buffer: [u8; 4] = [0; 4];
    let mut read_buf: [u8; 16] = [0; 16];

    let _result = CommandBuilder::create_execute(&mut buffer, true)
        .named("")
        .finish()
        .unwrap();
    let rboot = bootmode.set_high();
    delay.delay_ms(100_u32);
    uart.write_fmt(format_args!("Bootmode resutl: {:?}\r\n", rboot))
        .unwrap();
    let rwifi = wifi_en.set_low();
    uart.write_fmt(format_args!("Wifi resutl set low: {:?}\r\n", rwifi))
        .unwrap();
    delay.delay_ms(100_u32);
    let rwifi = wifi_en.set_high();
    uart.write_fmt(format_args!("Wifi resutl set high: {:?}\r\n", rwifi))
        .unwrap();
    delay.delay_ms(100_u32);

    // tx.set_high().unwrap();
    // rts.set_high().unwrap();
    loop {
        // led.set_high().unwrap();

        // if cts.is_high().unwrap() {
        //     uart.write_str("cts is high\r\n").unwrap();
        //     delay.delay_ms(1000_u32);
        // } else {
        //     uart.write_str("cts is low\r\n").unwrap();
        //     delay.delay_ms(1000_u32);
        // }
        //
        //
        // ############################################################################
        // if cts.is_low().unwrap() {
        //     uart.write_str("cts is low\r\n").unwrap();
        //     let write_result = wifi_uart.write(&buffer);
        //     uart.write_fmt(format_args!(
        //         "Write Result: {:?}\r\nWrite buffer was: {:?}\r\n",
        //         write_result, &buffer
        //     ))
        //     .unwrap();

        //     if rts.set_low().is_ok() {
        //         uart.write_str("Reading\r\n").unwrap();
        //         let read_result = match wifi_uart.read_timeout(&mut read_buf, &mut t, 2_000_001_u32)
        //         {
        //             Ok(_) => 128,
        //             Err(uarte::Error::Timeout(n)) => n,
        //             Err(uarte::Error::TxBufferTooLong) => 200,
        //             Err(uarte::Error::RxBufferTooLong) => 201,
        //             Err(uarte::Error::BufferNotInRAM) => 202,
        //             Err(uarte::Error::Transmit) => 203,
        //             Err(uarte::Error::Receive) => 204,
        //         };

        //         let _x = uart.write_fmt(format_args!("read bytes: {}\r\n", read_result));
        //         let _x = uart.write_fmt(format_args!("readbuffer: {:?}\r\n", &read_buf));
        //         rts.set_high().unwrap();
        //     }
        // } else {
        //     rts.set_low().unwrap();
        //     uart.write_str("Reading\r\n").unwrap();
        //     let read_result = match wifi_uart.read_timeout(&mut read_buf, &mut t, 2_000_001_u32) {
        //         Ok(_) => 128,
        //         Err(uarte::Error::Timeout(n)) => n,
        //         Err(uarte::Error::TxBufferTooLong) => 200,
        //         Err(uarte::Error::RxBufferTooLong) => 201,
        //         Err(uarte::Error::BufferNotInRAM) => 202,
        //         Err(uarte::Error::Transmit) => 203,
        //         Err(uarte::Error::Receive) => 204,
        //     };

        //     let _x = uart.write_fmt(format_args!("read bytes: {}\r\n", read_result));
        //     let _x = uart.write_fmt(format_args!("readbuffer: {:?}\r\n", &read_buf));
        //     rts.set_high().unwrap();
        // }
        // #############################################################################

        // led.set_high().unwrap();
        // delay.delay_ms(100_u32);
        // uart.write_fmt(format_args!("Start writing to wifi\r\n"))
        //     .unwrap();
        uart.write_fmt(format_args!("Write buffer {:?}\r\n", &buffer))
            .unwrap();
        let write_result = wifi_uart.write(&buffer);
        uart.write_fmt(format_args!("Write Result buffer: {:?}\r\n", &write_result))
            .unwrap();

        // let read_res = wifi_uart.read(&mut read_buf);
        // let x = read_buf[0];

        let read_result = match wifi_uart.read_timeout(&mut read_buf, &mut t, 5_000_000_u32) {
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
        uart.write(&read_buf).unwrap();

        delay.delay_ms(1000_u32);

        // led.set_low().unwrap();
        // delay.delay_ms(100_u32);
    }
}
