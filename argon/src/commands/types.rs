use atat::atat_derive::AtatEnum;
use core::fmt::{Display, Formatter, Result};
use heapless::String;
#[derive(Debug, PartialEq, Clone, AtatEnum)]
pub enum DNSMode {
    AUTOMATIC = 0,
    MANUAL = 1,
}
#[derive(Debug, PartialEq, Clone, AtatEnum, Copy)]
pub enum HTTPMethode {
    HEAD = 1,
    GET = 2,
    POST = 3,
    PUT = 4,
    DELTE = 5,
}

#[derive(Debug, PartialEq, Clone, AtatEnum, Copy)]
pub enum ContentType {
    X_WWW_FORM_URLENCODED = 0,
    JSON,
    MULTIPART_FORM_DATA,
    TEXT_XML,
}

#[derive(Debug, PartialEq, Clone, AtatEnum, Copy)]
pub enum TransportType {
    TCP = 1,
    SSL = 2,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    pub key: String<32>,
    pub value: String<64>,
}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}={}", self.key, self.value)
    }
}
