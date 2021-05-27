pub mod responses;
pub mod types;

use atat::atat_derive::AtatCmd;
use atat::AtatCmd;
use core::fmt::Write;
use heapless::String;
use heapless::Vec;
use no_std_net::Ipv4Addr;
use responses::*;
use types::*;

#[derive(Clone, AtatCmd)]
#[at_cmd("", EmptyResponse)]
pub struct At;

#[derive(Clone, AtatCmd)]
#[at_cmd("+CWDHCP", EmptyResponse)]
pub struct DHCPSet {
    #[at_arg(position = 0)]
    pub operate: u8,
    #[at_arg(position = 1)]
    pub mode: u8,
}

#[derive(Clone)]
pub struct WifiConnect {
    pub ssid: String<64>,
    pub pw: String<128>,
}

pub struct IPCmd {}

impl AtatCmd<16> for IPCmd {
    type Response = IPResponse;
    type Error = atat::GenericError;

    fn as_bytes(&self) -> Vec<u8, 16> {
        let mut buf: Vec<u8, 16> = Vec::new();
        write!(buf, "AT+CIFSR\r\n").unwrap();
        buf
    }

    fn parse(
        &self,
        resp: Result<&[u8], &atat::InternalError>,
    ) -> Result<Self::Response, atat::Error> {
        let resp = core::str::from_utf8(resp?).unwrap();

        let mut ip_resp = IPResponse {
            stat_ip: String::new(),
            stat_mac: String::new(),
        };

        for line in resp.lines() {
            if line.contains("STAIP") {
                ip_resp.stat_ip = line.split(',').nth(1).unwrap().trim_matches('\"').into();
            } else if line.contains("STAMAC") {
                ip_resp.stat_mac = line.split(',').nth(1).unwrap().trim_matches('\"').into();
            }
        }

        Ok(ip_resp)
    }
}

impl AtatCmd<256> for WifiConnect {
    type Response = WifiConnectResponse;
    type Error = atat::GenericError;

    fn as_bytes(&self) -> Vec<u8, 256> {
        let mut buf: Vec<u8, 256> = Vec::new();

        write!(
            buf,
            "AT+CWJAP=\"{}\",\"{}\"\r\n",
            self.ssid.as_str(),
            self.pw.as_str()
        )
        .unwrap();
        buf
    }
    fn parse(
        &self,
        resp: Result<&[u8], &atat::InternalError>,
    ) -> Result<Self::Response, atat::Error> {
        let resp = core::str::from_utf8(resp?).unwrap();
        let mut response = WifiConnectResponse {
            connected: false,
            got_ip: false,
        };

        for line in resp.lines() {
            match line {
                "WIFI DISCONNECTED" => response.connected = false,
                "WIFI CONNECTED" => response.connected = true,
                "WIFI GOT IP" => response.got_ip = true,
                _ => { /* throw away unknown lines for now */ }
            }
        }
        Ok(response)
    }
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+CIPDNS", EmptyResponse)]
pub struct SetDNSCmd {
    pub dns_mode: DNSMode,
    #[at_arg(position = 1, len = 39)]
    pub dns_ip1: Option<Ipv4Addr>,
    #[at_arg(position = 2, len = 39)]
    pub dns_ip2: Option<Ipv4Addr>,
    #[at_arg(position = 3, len = 39)]
    pub dns_ip3: Option<Ipv4Addr>,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+CIPDNS?", DNSResponse)]
pub struct GetDNSCmd {}

pub struct HttpCmd {
    pub methode: HTTPMethode,
    pub content_type: ContentType,
    pub url: String<256>,
    pub host: Option<String<64>>,
    pub path: Option<String<192>>,
    pub transport_type: TransportType,
    pub data: Option<String<2048>>,
    pub header: Option<Vec<Header, 16>>,
}

impl HttpCmd {
    pub fn new(methode: HTTPMethode, url: &str, data: Option<String<2048>>) -> Self {
        Self {
            methode,
            content_type: ContentType::TEXT_XML,
            url: url.into(),
            host: None,
            path: None,
            transport_type: TransportType::SSL,
            data,
            header: None,
        }
    }
}

impl AtatCmd<4352> for HttpCmd {
    type Response = HTTPResponse;
    type Error = atat::GenericError;
    fn as_bytes(&self) -> Vec<u8, 4352> {
        let mut buf: Vec<u8, 4352> = Vec::new();

        write!(
            buf,
            "AT+HTTPCLIENT={},{},\"{}\"",
            self.methode as u8, self.content_type as u8, self.url,
        )
        .unwrap();
        if self.host.is_some() && self.path.is_some() {
            write!(
                buf,
                ",\"{}\",\"{}\"",
                self.host.as_ref().unwrap(),
                self.path.as_ref().unwrap()
            )
            .unwrap();
        } else {
            write!(buf, ",,").unwrap();
        }
        write!(buf, ",{}", self.transport_type as u8).unwrap();

        if let Some(data) = self.data.as_ref() {
            write!(buf, ",\"{}\"", data).unwrap();
        }

        if let Some(header) = self.header.as_deref() {
            write!(buf, ",\"").unwrap();
            for i in 0..header.len() {
                if i == header.len() - 1 {
                    write!(buf, "{}", header[i]).unwrap();
                } else {
                    write!(buf, "{}&", header[i]).unwrap();
                }
            }
            write!(buf, "\"").unwrap();
        }

        write!(buf, "\r\n").unwrap();

        let ka = core::str::from_utf8(&buf).unwrap();
        let x = true;

        buf
    }

    fn parse(
        &self,
        resp: Result<&[u8], &atat::InternalError>,
    ) -> Result<Self::Response, atat::Error> {
        let resp = core::str::from_utf8(resp?).unwrap();
        let mut response = HTTPResponse {
            size: 0,
            data: String::<2048>::new(),
        };

        for line in resp.lines() {
            if line.contains("HTTPCLIENT") {
                let size: u32 = line
                    .split(':')
                    .nth(1)
                    .unwrap_or("0,0")
                    .split(',')
                    .nth(0)
                    .unwrap()
                    .parse()
                    .unwrap();

                if size > 2048 {
                    return Err(atat::Error::Overflow);
                } else {
                    response.size = size;
                    response.data = line.split(',').nth(1).unwrap_or("").into();
                }
            }
        }

        Ok(response)
    }
}
