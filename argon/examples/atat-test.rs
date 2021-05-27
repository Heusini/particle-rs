#![no_main]
#![no_std]

extern crate embedded_hal_spy;
extern crate panic_halt;

use atat::AtatCmd;
use core::fmt::Write;
use cortex_m_rt::entry;
use heapless::Vec;
use nrf52840_hal::gpio::Level;
use nrf52840_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use nrf52840_hal::uarte;
use nrf52840_hal::Delay;
use particle_argon::commands::types::*;
use particle_argon::commands::types::*;
use particle_argon::commands::*;
use particle_argon::Board;

#[allow(dead_code)]
fn test_httpcmd_asbyte() {
    let mut ka = Vec::<Header, 16>::new();
    let h = Header {
        key: "asdf".into(),
        value: "nope".into(),
    };
    let h2 = Header {
        key: "asdf".into(),
        value: "yes".into(),
    };
    ka.push(h).unwrap();
    ka.push(h2).unwrap();

    let httpcmd = HttpCmd {
        methode: HTTPMethode::GET,
        content_type: ContentType::JSON,
        url: "asdf".into(),
        host: None,
        path: None,
        transport_type: TransportType::TCP,
        data: Some("asdfadsf".into()),
        header: Some(ka),
    };

    let buf = httpcmd.as_bytes();

    let x = core::str::from_utf8(&buf).unwrap();
    let b = true;
}

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

    // test_httpcmd_asbyte();
    // loop {}

    uart.write_str("Starting\r\n");
    let mut delay = Delay::new(board.SYST);

    if board.WIFI.on(&mut delay).is_ok() {
        uart.write_str("Wifi module is online\r\n").unwrap();
    } else {
        uart.write_str("Wifi module is offline\r\n").unwrap();
    }

    uart.write_str("Set dhcp\r\n");
    let buf = board.WIFI.set_dhcp().unwrap();
    uart.write_fmt(format_args!("dhcp resp: {:?}\r\n", &buf));
    uart.write_str("set dhcp\r\n");

    uart.write_str("Started wificon\r\n");
    let wificon = board.WIFI.connect("WGLAN", "WGinet19");
    uart.write_fmt(format_args!("wificon: {:?}\r\n", wificon));
    uart.write_str("Finished wificon\r\n");

    uart.write_str("Get ip\r\n");
    let buf = board.WIFI.get_ip().unwrap();
    uart.write_fmt(format_args!("IP: {:?}\r\n", buf));
    uart.write_str("got ip\r\n");

    uart.write_str("Set dns\r\n");
    let buf = board.WIFI.set_dns(
        DNSMode::MANUAL,
        Some("192.168.0.1".parse().unwrap()),
        Some("192.168.0.1".parse().unwrap()),
        None,
    );
    uart.write_fmt(format_args!("Set DNS: {:?}\r\n", buf));
    uart.write_str("set dns\r\n");

    uart.write_str("Get dns\r\n");
    let buf = board.WIFI.get_dns();
    uart.write_fmt(format_args!("DNS: {:?}\r\n", buf));
    uart.write_str("got dns\r\n");

    uart.write_str("HTTP:\r\n");
    let buf = board.WIFI.http_get("https://google.de/");
    uart.write_fmt(format_args!("HTTP response: {:?}\r\n", buf));
    uart.write_str("finished http\r\n");
    loop {}
}
