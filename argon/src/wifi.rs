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
    for x in iter {
        if x == [b'O', b'K'] {
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

    fn check_esp(&mut self) -> Result<(), WifiError> {
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

    fn read(&mut self, buf: &mut [u8]) -> Result<(), WifiError> {
        let mut buf_pos = 0;
        let mut is_ok = true;

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
