use crate::details::crcTable;
use crate::proto;
use nrf52840_hal::Uarte;
pub struct Muxer {
    channel: isize,
    initiator: bool,
}

fn crc(data: &[u8]) -> u8 {
    let mut c: u8 = 0xff;
    for i in 0..data.len() {
        c = crcTable[(c ^ data[i]) as usize];
    }
    c
}

fn fcs(data: &[u8]) -> u8 {
    !crc(data)
}

impl Muxer {
    pub fn new() -> Self {
        Muxer {
            channel: 1,
            initiator: true,
        }
    }

    pub fn send_channel<T>(
        &self,
        uarte: &mut nrf52840_hal::uarte::Uarte<T>,
        channel: u8,
        control: u8,
        cmd: bool,
        data: &[u8],
    ) where
        T: nrf52840_hal::uarte::Instance,
    {
        let mut header: [u8; 5] = [0; 5];
        let mut footer: [u8; 2] = [0; 2];

        let hlen: usize = if data.len() <= 0x7f { 4 } else { 5 };

        header[0] = proto::Frame::BASIC_FLAG as u8;
        header[1] = (channel << 2) | proto::Extension::EA as u8;

        if self.initiator && cmd || !self.initiator && !cmd {
            header[1] |= proto::Address::CR as u8;
        }

        header[2] = control;
        header[3] = (data.len() as u8 & 0x7f) << 1;

        if hlen == 5 {
            header[4] = data.len() as u8 >> 7;
        } else {
            header[3] |= proto::Extension::EA as u8;
        }

        footer[0] = fcs(&header[1..hlen]);
        footer[1] = proto::Frame::BASIC_FLAG as u8;

        uarte.write(&header[0..hlen]).unwrap();
        uarte.write(data).unwrap();
        uarte.write(&footer).unwrap();
    }
}
