#![no_main]
#![no_std]

extern crate embedded_hal_spy;
extern crate panic_halt;

use core::fmt::Write;
use cortex_m_rt::entry;
use nrf52840_hal::gpio::Level;
use nrf52840_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
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

    uart.write_str("Starting\r\n");
    let mut delay = Delay::new(board.SYST);

    if board.WIFI.on(&mut delay).is_ok() {
        uart.write_str("Wifi module is online\r\n").unwrap();
    } else {
        uart.write_str("Wifi module is offline\r\n").unwrap();
    }

    // // uart.write_str("scanning\r\n");
    // // let buf = board.WIFI.scan().unwrap();

    // // uart.write(&buf);
    // // uart.write_str("Finished Writing Buffer\r\n");
    // uart.write_str("Set syslog\r\n");
    // let buf = board.WIFI.enable_at_log().unwrap();
    // uart.write(&buf);
    // uart.write_str("set syslog\r\n");

    uart.write_str("Disable ipv6\r\n");
    let buf = board.WIFI.disable_ipv6().unwrap();
    uart.write(&buf);
    uart.write_str("disabled ipv6\r\n");

    uart.write_str("Get dhcp\r\n");
    let buf = board.WIFI.get_dhcp().unwrap();
    uart.write(&buf);
    uart.write_str("got dhcp\r\n");

    uart.write_str("Set dhcp\r\n");
    let buf = board.WIFI.set_dhcp().unwrap();
    uart.write_fmt(format_args!("dhcp resp: {:?}\r\n", &buf));
    uart.write_str("set dhcp\r\n");

    uart.write_str("Get dhcp\r\n");
    let buf = board.WIFI.get_dhcp().unwrap();
    uart.write(&buf);
    uart.write_str("got dhcp\r\n");

    uart.write_str("Connecting to Wifi\r\n");
    let buf = board.WIFI.connect("WGLAN", "WGinet19").unwrap();
    uart.write(&buf);
    uart.write_str("Finished Connecting to Wifi\r\n");

    uart.write_str("Get mac\r\n");
    let buf = board.WIFI.get_mac().unwrap();
    uart.write(&buf);
    uart.write_str("got mac\r\n");

    uart.write_str("Get ip\r\n");
    let buf = board.WIFI.get_ip().unwrap();
    uart.write(&buf);
    uart.write_str("got ip\r\n");

    delay.delay_ms(1000_u32);

    uart.write_str("Set dns\r\n");
    let buf = board.WIFI.set_dns().unwrap();
    uart.write(&buf);
    uart.write_str("set dns\r\n");

    uart.write_str("Get dns\r\n");
    let buf = board.WIFI.get_dns().unwrap();
    uart.write(&buf);
    uart.write_str("got dns\r\n");

    delay.delay_ms(1000_u32);

    // ########################################### HTTP ##############################
    uart.write_str("HTTP:\r\n");
    let buf = board.WIFI.http("https://google.de/").unwrap();
    uart.write(&buf);
    uart.write_str("finished http\r\n");
    // ###############################################################################

    uart.write_str("Get dns\r\n");
    let buf = board.WIFI.get_dns().unwrap();
    uart.write(&buf);
    uart.write_str("got dns\r\n");

    // send data over tcp
    // uart.write_str("Connect tcp\r\n");
    // let buf = board.WIFI.tcp_connect("192.168.0.118", 8080).unwrap();
    // uart.write(&buf);
    // uart.write_str("connected\r\n");

    // uart.write_str("Send data over tcp\r\n");
    // let buf = board.WIFI.tcp_send("Hello world!").unwrap();
    // uart.write(&buf);
    // uart.write_str("connected\r\n");

    // ############################################ MQTT ################################
    // delay.delay_ms(1000_u32);
    // uart.write_str("Testing mqtt\r\n");
    // let buf = board.WIFI.m3qtt_conncfg().unwrap();
    // uart.write(&buf);
    // uart.write_str("finished mqtt\r\n");

    // uart.write_str("Testing mqtt\r\n");
    // let buf = board.WIFI.mqtt_query().unwrap();
    // uart.write(&buf);
    // uart.write_str("finished mqtt\r\n");

    // ##################################################################################

    // uart.write_str("Disconnecting\r\n");
    // let buf = board.WIFI.connect("devolo-846", "WGinet19").unwrap();
    // uart.write(&buf);
    // uart.write_str("Disconnected\r\n");
    loop {}
}
