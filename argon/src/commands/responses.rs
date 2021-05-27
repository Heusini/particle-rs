use crate::commands::types::*;
use atat::atat_derive::AtatResp;
use heapless::String;

#[derive(Clone, Debug, PartialEq, AtatResp)]
pub struct EmptyResponse {}

#[derive(Clone, Debug, PartialEq, AtatResp)]
pub struct WifiConnectResponse {
    #[at_arg(position = 0)]
    pub connected: bool,
    #[at_arg(position = 1)]
    pub got_ip: bool,
}

#[derive(Clone, Debug, PartialEq, AtatResp)]
pub struct DNSResponse {
    pub dns_mode: DNSMode,
    pub dns_1: Option<String<64>>,
    pub dns_2: Option<String<64>>,
    pub dns_3: Option<String<64>>,
}

#[derive(Clone, Debug, PartialEq, AtatResp)]
pub struct IPResponse {
    pub stat_ip: String<16>,
    pub stat_mac: String<32>,
}

#[derive(Clone, Debug, PartialEq, AtatResp)]
pub struct HTTPResponse {
    pub size: u32,
    pub data: String<2048>,
}
