#![no_std]
#![no_main]

#[path = "../fmt.rs"]
mod fmt;
#[path = "../ld2410c.rs"]
mod ld2410c;

use crate::fmt::warn;
#[cfg(not(feature = "defmt"))]
use panic_halt as _;

#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts, dma, mode, peripherals,
    usart::{self, Config, Uart},
};

use fmt::info;
use ld2410c::{Ld2410c, UartReader};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
    DMA2_STREAM7 => dma::InterruptHandler<peripherals::DMA2_CH7>;
    DMA2_STREAM2 => dma::InterruptHandler<peripherals::DMA2_CH2>;
});

// I needed it 'cause of orphan rule
struct Ld2410cUart<'d>(Uart<'d, mode::Async>);

impl UartReader for Ld2410cUart<'_> {
    type Error = usart::Error;

    async fn read_until_idle(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.read_until_idle(buf).await
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Program started!");

    let p = embassy_stm32::init(Default::default());

    let mut usart_config = Config::default();
    usart_config.baudrate = 256_000;

    let usart = Uart::new(
        p.USART1,
        p.PA10,
        p.PA9,
        p.DMA2_CH7,
        p.DMA2_CH2,
        Irqs,
        usart_config,
    )
    .unwrap();

    let mut driver = Ld2410c::new(Ld2410cUart(usart));
    let mut buf = [0u8; 128];

    loop {
        match driver.read_frame(&mut buf).await {
            Ok(Some(d)) => {
                info!("Status: {}", d.status);
                info!("Movement target distance: {} cm", d.movement_distance);
                info!("Exercise target energy value: {}", d.movement_energy);
                info!(
                    "Distance to stationary target: {} cm",
                    d.stationary_distance
                );
                info!("Stationary target energy value: {}", d.stationary_energy);
                info!("Detection distance: {} cm", d.detection_distance);
            }
            Ok(None) => warn!("Unknown frame"),
            Err(e) => info!("UART Error: {:?}", e),
        }
    }
}
