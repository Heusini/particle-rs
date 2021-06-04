const BUF_SIZE: usize = 4096;

#[allow(dead_code)]
pub struct Serial<T> {
    pub stream: nrf52840_hal::Uarte<T>,
    buffer: [u8; BUF_SIZE],
    buf_pos: u32,
}

impl<T> Serial<T> {
    pub fn new(stream: nrf52840_hal::Uarte<T>) -> Serial<T> {
        Serial {
            stream,
            buffer: [0; BUF_SIZE],
            buf_pos: 0,
        }
    }
}
