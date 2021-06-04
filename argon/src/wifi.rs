use at_commands::builder::CommandBuilder;
use atat::{AtatCmd, ComQueue, IngressManager, ResQueue, UrcQueue};
use heapless::spsc::Queue;
use nrf52840_hal::gpio;
use nrf52840_hal::gpio::p0;
use nrf52840_hal::pac::UARTE1;
use nrf52840_hal::prelude::*;
use nrf52840_hal::timer::OneShot;
use nrf52840_hal::uarte;
use nrf52840_hal::Delay;
use nrf52840_hal::Timer;

use crate::commands::responses::*;
use crate::commands::types::*;
use crate::commands::*;
use heapless::String;
use no_std_net::Ipv4Addr;

pub struct WIFI {
    uarte: uarte::Uarte<UARTE1>,
    wifi_en: p0::P0_24<gpio::Output<gpio::OpenDrain>>,
    bootmode: p0::P0_16<gpio::Output<gpio::PushPull>>,
    timer: Timer<nrf52840_hal::pac::TIMER4, OneShot>,
    ingress: IngressManager<atat::DefaultDigester, atat::DefaultUrcMatcher, 4096, 5>,
    res_c:
        heapless::spsc::Consumer<'static, Result<heapless::Vec<u8, 4096>, atat::InternalError>, 2>,
}

#[derive(Debug)]
pub enum WifiError {
    CouldNotStartEsp,
    WifiError,
    FmtError(core::fmt::Error),
    UartError(nrf52840_hal::uarte::Error),
    PinError(()),
    AtCommandError(usize),
    AtatError(atat::Error),
}

impl From<atat::Error> for WifiError {
    fn from(error: atat::Error) -> Self {
        WifiError::AtatError(error)
    }
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
        static mut RES_Q: ResQueue<4096> = Queue::new();
        let (res_p, res_c) = unsafe { RES_Q.split() };
        static mut URC_Q: UrcQueue<4096, 5> = Queue::new();
        let (urc_p, _urc_c) = unsafe { URC_Q.split() };
        static mut COM_Q: ComQueue = Queue::new();
        let (_com_p, com_c) = unsafe { COM_Q.split() };

        let ingress = IngressManager::new(res_p, urc_p, com_c);
        WIFI {
            uarte,
            wifi_en,
            bootmode,
            timer,
            ingress,
            res_c,
        }
    }

    pub fn check_esp(&mut self) -> Result<EmptyResponse, WifiError> {
        for _ in 0..5 {
            let at_ok = self.send_command(&At {}, 1);
            if at_ok.is_ok() {
                return at_ok;
            }
        }

        Err(WifiError::CouldNotStartEsp)
    }

    fn send_command<T, const LEN: usize>(
        &mut self,
        command: &T,
        timeout: usize,
    ) -> Result<T::Response, WifiError>
    where
        T: atat::AtatCmd<LEN>,
        atat::Error: From<atat::Error<<T as AtatCmd<LEN>>::Error>>,
    {
        let mut buf: [u8; 2048] = [0; 2048];
        self.uarte.write(&command.as_bytes())?;

        let len = self.read(&mut buf, timeout)?;

        self.ingress.write(&buf[..len]);
        self.ingress.digest();

        if let Some(result) = self.res_c.dequeue() {
            match command.parse(result.as_deref()) {
                Ok(cmd) => return Ok(cmd),
                Err(e) => return Err(WifiError::AtatError(e.into())),
            }
        }

        Err(WifiError::AtatError(atat::Error::InvalidResponse))
    }

    pub fn connect(
        &mut self,
        ssid: impl Into<String<64>>,
        pw: impl Into<String<128>>,
    ) -> Result<WifiConnectResponse, WifiError> {
        self.send_command(
            &WifiConnect {
                ssid: ssid.into(),
                pw: pw.into(),
            },
            10,
        )
    }

    pub fn disconnect(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 64] = [0; 64];

        CommandBuilder::create_execute(&mut sendbuf, true)
            .named("+CWQAP")
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf, 10)?;

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
        self.read(&mut buf, 10)?;

        Ok(buf)
    }

    pub fn get_dns(&mut self) -> Result<DNSResponse, WifiError> {
        self.send_command(&GetDNSCmd {}, 10)
    }

    pub fn set_dns(
        &mut self,
        dns_mode: DNSMode,
        dns_ip1: Option<Ipv4Addr>,
        dns_ip2: Option<Ipv4Addr>,
        dns_ip3: Option<Ipv4Addr>,
    ) -> Result<EmptyResponse, WifiError> {
        self.send_command(
            &SetDNSCmd {
                dns_mode,
                dns_ip1,
                dns_ip2,
                dns_ip3,
            },
            10,
        )
    }
    pub fn http_get(
        &mut self,
        url: &str,
        transport_type: TransportType,
    ) -> Result<HTTPResponse, WifiError> {
        self.send_command(
            &HttpCmd::new(HTTPMethode::GET, url, transport_type, None),
            10,
        )
        // let mut buf: [u8; 1024] = [0; 1024];
        // let mut sendbuf: [u8; 128] = [0; 128];

        // CommandBuilder::create_set(&mut sendbuf, true)
        //     .named("+HTTPCLIENT")
        //     .with_int_parameter(2)
        //     .with_int_parameter(3)
        //     .with_string_parameter(url)
        //     .with_optional_string_parameter(None)
        //     .with_optional_string_parameter(None)
        //     .with_int_parameter(2)
        //     .finish()?;

        // self.uarte.write(&sendbuf)?;
        // self.read(&mut buf, 10)?;

        // Ok(buf)
    }

    pub fn get_ip(&mut self) -> Result<IPResponse, WifiError> {
        self.send_command(&IPCmd {}, 10)
    }

    pub fn set_dhcp(&mut self) -> Result<EmptyResponse, WifiError> {
        self.send_command(
            &DHCPSet {
                operate: 1,
                mode: 1,
            },
            10,
        )
    }

    pub fn get_dhcp(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 128] = [0; 128];

        CommandBuilder::create_query(&mut sendbuf, true)
            .named("+CWDHCP")
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf, 10)?;

        Ok(buf)
    }

    pub fn read(&mut self, buf: &mut [u8], timeout_s: usize) -> Result<usize, WifiError> {
        let mut buf_pos = 0;
        let mut is_ok = true;

        let mut time_out = 0;

        let mut length = 0;
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
                        length += n;
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

        Ok(length)
    }

    pub fn scan(&mut self) -> Result<[u8; 255], WifiError> {
        let mut buf: [u8; 255] = [0; 255];
        let mut sendbuf: [u8; 32] = [0; 32];

        CommandBuilder::create_execute(&mut sendbuf, true)
            .named("+CWLAP")
            .finish()?;

        self.uarte.write(&sendbuf)?;
        self.read(&mut buf, 10)?;
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
