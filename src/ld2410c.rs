use embassy_stm32::{mode, usart};

const FRAME_HEADER: [u8; 4] = [0xF4, 0xF3, 0xF2, 0xF1];

pub struct TargetData {
    pub status: u8,
    pub movement_distance: u16,
    pub movement_energy: u8,
    pub stationary_distance: u16,
    pub stationary_energy: u8,
    pub detection_distance: u16,
}

pub struct Ld2410c<'d> {
    uart: usart::Uart<'d, mode::Async>,
}

impl<'d> Ld2410c<'d> {
    pub fn new(uart: usart::Uart<'d, mode::Async>) -> Self {
        Self { uart }
    }

    pub async fn read_frame(&mut self, buf: &mut [u8]) -> Result<Option<TargetData>, usart::Error> {
        let n = self.uart.read_until_idle(buf).await?;
        Ok(parse_frame(&buf[..n]))
    }
}

fn parse_frame(data: &[u8]) -> Option<TargetData> {
    if data.len() < 17 || data[0..4] != FRAME_HEADER {
        return None;
    }

    Some(TargetData {
        status: data[8],
        movement_distance: u16::from_le_bytes([data[9], data[10]]),
        movement_energy: data[11],
        stationary_distance: u16::from_le_bytes([data[12], data[13]]),
        stationary_energy: data[14],
        detection_distance: u16::from_le_bytes([data[15], data[16]]),
    })
}
