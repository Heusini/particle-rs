#![no_main]
#![no_std]

// extern crate base64;
extern crate embedded_hal_spy;
extern crate panic_halt;

use atat::AtatCmd;
use core::fmt::Write;
use cortex_m_rt::entry;
use heapless::{String, Vec};
use nrf52840_hal::gpio::{Level, OpenDrainConfig};
use nrf52840_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use nrf52840_hal::uarte;
use nrf52840_hal::Delay;
use nrf52840_hal::Timer;
use particle_argon::commands::types::*;
use particle_argon::commands::types::*;
use particle_argon::commands::*;
use particle_argon::wifi;
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
        content_type: ContentType::Json,
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

fn escape_seq(msg: &[u8], out: &mut [u8]) -> usize {
    let mut out_pos = 0;
    for i in 0..msg.len() {
        if msg[i] == b'\\' || msg[i] == b'"' || msg[i] == b',' {
            out[out_pos] = b'\\';
            out_pos += 1;
            out[out_pos] = msg[i];
            out_pos += 1;
        } else {
            out[out_pos] = msg[i];
            out_pos += 1;
        }
    }
    out_pos
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

    let wifi_pins = uarte::Pins {
        txd: board
            .pins
            .p1_05
            .take()
            .unwrap()
            .into_push_pull_output(Level::Low)
            .degrade(),
        rxd: board.pins.p1_04.take().unwrap().degrade(),
        cts: Some(board.pins.p1_06.take().unwrap().degrade()),
        rts: Some(
            board
                .pins
                .p1_07
                .take()
                .unwrap()
                .into_push_pull_output(Level::High)
                .degrade(),
        ),
    };
    let uarte = uarte::Uarte::new(
        board.UARTE1,
        wifi_pins,
        uarte::Parity::EXCLUDED,
        uarte::Baudrate::BAUD921600,
    );

    let wifi_en = board
        .pins
        .p0_24
        .take()
        .unwrap()
        .into_open_drain_output(OpenDrainConfig::Standard0Disconnect1, Level::Low);
    let bootmode = board
        .pins
        .p0_16
        .take()
        .unwrap()
        .into_push_pull_output(Level::Low);
    let mut timer = Timer::new(board.TIMER4);
    timer.disable_interrupt();

    let mut wifi = wifi::WIFI::new(uarte, wifi_en, bootmode, timer);

    uart.write_str("Starting\r\n");
    let mut delay = Delay::new(board.SYST);

    if wifi.on().is_ok() {
        uart.write_str("Wifi module is online\r\n").unwrap();
    } else {
        uart.write_str("Wifi module is offline\r\n").unwrap();
    }

    uart.write_str("Set dhcp\r\n");
    let buf = wifi.set_dhcp().unwrap();
    uart.write_fmt(format_args!("dhcp resp: {:?}\r\n", &buf));
    uart.write_str("set dhcp\r\n");

    uart.write_str("Started wificon\r\n");
    let wificon = wifi.connect("WGLAN", "WGinet19");
    uart.write_fmt(format_args!("wificon: {:?}\r\n", wificon));
    uart.write_str("Finished wificon\r\n");

    uart.write_str("Get ip\r\n");
    let buf = wifi.get_ip().unwrap();
    uart.write_fmt(format_args!("IP: {:?}\r\n", buf));
    uart.write_str("got ip\r\n");

    uart.write_str("Set dns\r\n");
    let buf = wifi.set_dns(
        DNSMode::MANUAL,
        Some("192.168.0.1".parse().unwrap()),
        Some("192.168.0.1".parse().unwrap()),
        None,
    );
    uart.write_fmt(format_args!("Set DNS: {:?}\r\n", buf));
    uart.write_str("set dns\r\n");

    uart.write_str("Get dns\r\n");
    let buf = wifi.get_dns();
    uart.write_fmt(format_args!("DNS: {:?}\r\n", buf));
    uart.write_str("got dns\r\n");

    uart.write_str("HTTP:\r\n");
    let buf = wifi.http_get("http://192.168.0.118:8000/file", TransportType::TCP);
    uart.write_fmt(format_args!("HTTP response: {:?}\r\n", buf));
    uart.write_str("finished http\r\n");

    wifi.enable_at_log();

    uart.write_str("HTTP Post: \r\n");
    let text = "{\"msg\":\"hello world\",\"enc\":\"d3JL+R4Mn553msIhQ39sww==\"}";
    // let mut buf = [0u8; 128];
    // let pos = escape_seq(text, &mut buf);
    let resp = wifi.http_post(
        "http://192.168.0.118:8000/message",
        TransportType::TCP,
        String::from(text),
        ContentType::Json,
    );
    uart.write_fmt(format_args!("HTTP response: {:?}\r\n", resp));
    uart.write_str("finished http\r\n");

    // let mut read_buf = [0u8; 2048];
    // let decode_conf = base64::Config::new(base64::CharacterSet::Standard, true);
    // let data = base64::decode_config_slice(&buf.unwrap().data, decode_conf, &mut read_buf);

    // uart.write(&read_buf);

    loop {}
}
