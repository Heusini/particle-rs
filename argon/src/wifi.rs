use at_commands::builder::CommandBuilder;
use at_commands::parser::CommandParser;
use core::fmt::Write;
use nrf52840_hal::gpio;
use nrf52840_hal::gpio::p0;
use nrf52840_hal::pac::UARTE1;
use nrf52840_hal::prelude::*;
use nrf52840_hal::timer::OneShot;
use nrf52840_hal::uarte;
use nrf52840_hal::Delay;
use nrf52840_hal::Timer;

use crate::muxer;
use crate::proto;

pub struct WIFI {
    uarte: uarte::Uarte<UARTE1>,
    wifi_en: p0::P0_24<gpio::Output<gpio::OpenDrain>>,
    bootmode: p0::P0_16<gpio::Output<gpio::PushPull>>,
    timer: Timer<nrf52840_hal::pac::TIMER4, OneShot>,
}

#[derive(Debug)]
pub enum WifiError {
    CouldNotStartEsp,
    WifiError,
    FmtError(core::fmt::Error),
    UartError(nrf52840_hal::uarte::Error),
    PinError(()),
    AtCommandError(usize),
}

impl From<usize> for WifiError {
    fn from(error: usize) -> Self {
        WifiError::AtCommandError(error)
    }
}

impl From<core::fmt::Error> for WifiError {
    fn from(error: core::fmt::Error) -> Self {
        WifiError::FmtError(error)
    }
}

impl From<nrf52840_hal::uarte::Error> for WifiError {
    fn from(error: nrf52840_hal::uarte::Error) -> Self {
        WifiError::UartError(error)
    }
}

#[allow(dead_code)]
fn actual_length(buf: &[u8]) -> u32 {
    let mut count = 0;
    for i in 0..buf.len() {
        if buf[i] == 0_u8 {
            return count;
        }
        count += 1;
    }
    return count;
}

#[allow(dead_code)]
fn compare(b1: &[u8], b2: &[u8]) -> bool {
    let length_b1 = actual_length(b1);
    let same = length_b1 == actual_length(b2);

    if same {
        for i in 0..length_b1 {
            if b1.get(i as usize) != b2.get(i as usize) {
                return false;
            }
        }
    }

    same
}

fn check_ok(buf: &[u8]) -> bool {
    let iter = buf.windows(2);
    let iter2 = buf.windows(5);
    for x in iter {
        if x == [b'O', b'K'] {
            return true;
        }
    }

    for x in iter2 {
        if x == [b'E', b'R', b'R', b'O', b'R'] {
            return true;
        }
    }
    false
}

impl WIFI {
    pub fn new(
        uarte: uarte::Uarte<UARTE1>,
        wifi_en: p0::P0_24<gpio::Output<gpio::OpenDrain>>,
        bootmode: p0::P0_16<gpio::Output<gpio::PushPull>>,
        timer: Timer<nrf52840_hal::pac::TIMER4, OneShot>,
    ) -> Self {
        WIFI {
            uarte,
            wifi_en,
            bootmode,
            timer,
        }
    }

    pub fn muxer_test(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let sendbuf: [u8; 4] = [b'A', b'T', b'\r', b'\n'];

        let muxer = muxer::Muxer::new();
        muxer.send_channel(
            &mut self.uarte,
            1_u8,
            proto::FrameType::UIH as u8,
            false,
            &sendbuf,
        );

        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn check_esp(&mut self) -> Result<(), WifiError> {
        let mut buffer: [u8; 16] = [0; 16];
        let mut is_ok = false;
        while !is_ok {
            self.uarte.write_str("AT\r\n")?;

            buffer = [0; 16];
            is_ok = match self
                .uarte
                .read_timeout(&mut buffer, &mut self.timer, 1_000_000_u32)
            {
                Ok(_) => true,
                Err(uarte::Error::Timeout(n)) => {
                    if n > 0 {
                        true
                    } else {
                        false
                    }
                }
                Err(_) => false,
            }
        }
        if CommandParser::parse(&buffer)
            .expect_identifier(b"AT\r\n")
            .expect_identifier(b"\r\nOK\r\n")
            .finish()
            .is_ok()
        {
            return Ok(());
        }
        Err(WifiError::CouldNotStartEsp)
    }

    pub fn connect(&mut self, ssid: &str, pw: &str) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 64] = [0; 64];

        CommandBuilder::create_set(&mut sendbuf, true)
            .named("+CWJAP")
            .with_string_parameter(ssid)
            .with_string_parameter(pw)
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn disconnect(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 64] = [0; 64];

        CommandBuilder::create_execute(&mut sendbuf, true)
            .named("+CWQAP")
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn set_reconnect(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_set(&mut sendbuf, true)
            .named("+CWRECONNCFG")
            .with_int_parameter(1)
            .with_int_parameter(100)
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn http(&mut self, url: &str) -> Result<[u8; 1024], WifiError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_set(&mut sendbuf, true)
            .named("+HTTPCLIENT")
            .with_int_parameter(2)
            .with_int_parameter(3)
            .with_string_parameter(url)
            .with_optional_string_parameter(None)
            .with_optional_string_parameter(None)
            .with_int_parameter(1)
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn set_cmux(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_set(&mut sendbuf, true)
            .named("+CMUX")
            .with_int_parameter(0)
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn get_ip(&mut self) -> Result<[u8; 1024], WifiError> {
        let mut buf: [u8; 1024] = [0; 1024];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_execute(&mut sendbuf, true)
            .named("+CIFSR")
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }
    pub fn tcp_send(&mut self, data: &str) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];

        let length: i32 = data.len() as i32;

        CommandBuilder::create_set(&mut sendbuf, true)
            .named("+CIPSEND")
            .with_int_parameter(length)
            .finish()?;

        self.uarte.write(&sendbuf)?;

        self.read(&mut buf)?;
        self.uarte.write_str(data)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn tcp_connect(&mut self, ip: &str, port: i32) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_set(&mut sendbuf, true)
            .named("+CIPSTART")
            .with_string_parameter("TCP")
            .with_string_parameter(ip)
            .with_int_parameter(port)
            .with_int_parameter(20)
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn set_dhcp(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_set(&mut sendbuf, true)
            .named("+CWDHCP")
            .with_int_parameter(1)
            .with_int_parameter(1)
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn get_dhcp(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_query(&mut sendbuf, true)
            .named("+CWDHCP")
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn get_mac(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_query(&mut sendbuf, true)
            .named("+CIPSTAMAC")
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn get_at_commands(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_query(&mut sendbuf, true)
            .named("+CMD")
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn enable_at_log(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_set(&mut sendbuf, true)
            .named("+SYSLOG")
            .with_int_parameter(1)
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }
    pub fn m3qtt_conncfg(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_set(&mut sendbuf, true)
            .named("+MQTTCONNCFG")
            .with_int_parameter(0)
            .with_int_parameter(0)
            .with_string_parameter("lwt")
            .with_string_parameter("dead")
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }
    pub fn mqtt_query(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];
        CommandBuilder::create_query(&mut sendbuf, true)
            .named("+MQTTCONN")
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn mqtt_connect(&mut self) -> Result<[u8; 2048], WifiError> {
        let mut buf: [u8; 2048] = [0; 2048];
        let mut sendbuf: [u8; 64] = [0; 64];

        CommandBuilder::create_set(&mut sendbuf, true)
            .named("+MQTTCONN")
            .with_int_parameter(0)
            .with_string_parameter("192.168.0.118")
            .with_int_parameter(1883)
            .with_int_parameter(0)
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn mqtt(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_set(&mut sendbuf, true)
            .named("+MQTTUSERCFG")
            .with_int_parameter(0)
            .with_int_parameter(1)
            .with_string_parameter("ESP32")
            .with_string_parameter("espressif")
            .with_string_parameter("1234567890")
            .with_int_parameter(0)
            .with_int_parameter(0)
            .with_string_parameter("")
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;

        Ok(buf)
    }

    pub fn tcp_check(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_execute(&mut sendbuf, true)
            .named("+CIPSTATUS")
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;
        Ok(buf)
    }

    pub fn read(&mut self, buf: &mut [u8], timeout_s: usize) -> Result<(), WifiError> {
        let mut buf_pos = 0;
        let mut is_ok = true;

        let mut time_out = 0;

        while is_ok {
            if buf_pos >= buf.len() {
                break;
            }

            is_ok =
                match self
                    .uarte
                    .read_timeout(&mut buf[buf_pos..], &mut self.timer, 1_000_000_u32)
                {
                    Ok(_) => false,
                    Err(nrf52840_hal::uarte::Error::Timeout(n)) => {
                        if n == 0 {
                            if time_out > timeout_s {
                                break;
                            }
                            time_out += 1;
                            continue;
                        }
                        if check_ok(&buf[buf_pos..buf_pos + n]) {
                            false
                        } else {
                            buf_pos += n;
                            true
                        }
                    }
                    Err(e) => return Err(e.into()),
                }
        }

        Ok(())
    }

    pub fn scan(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 32] = [0; 32];

        CommandBuilder::create_execute(&mut sendbuf, true)
            .named("+CWLAP")
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf)?;
        // let mut buf_pos = 0;
        // let mut is_ok = true;
        // while is_ok {
        //     if buf_pos >= 1024 {
        //         break;
        //     }
        //     is_ok = match self
        //         .uarte
        //         .read_timeout(&mut buf[buf_pos..], &mut self.timer, 1_000_u32)
        //     {
        //         Ok(_) => false,
        //         Err(nrf52840_hal::uarte::Error::Timeout(n)) => {
        //             if n == 0 {
        //                 continue;
        //             }
        //             if check_ok(&buf[buf_pos..buf_pos + n]) {
        //                 false
        //             } else {
        //                 buf_pos += n;
        //                 true
        //             }
        //         }
        //         Err(_) => false,
        //     }
        // }
        Ok(buf)
    }

    pub fn on(&mut self, delay: &mut Delay) -> Result<(), WifiError> {
        self.wifi_en.set_low().unwrap();
        delay.delay_ms(100_u32);
        self.bootmode.set_high().unwrap();
        delay.delay_ms(100_u32);
        self.wifi_en.set_high().unwrap();
        delay.delay_ms(100_u32);

        self.check_esp()?;

        Ok(())
    }
}
