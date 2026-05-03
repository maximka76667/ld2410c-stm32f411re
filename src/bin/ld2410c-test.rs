#![no_std]
#![no_main]

#[path = "../fmt.rs"]
mod fmt;
use ld2410c_stm32f411re::ld2410c::Ld2410c;

#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts, dma, peripherals,
    usart::{self, Config, Uart},
};

use fmt::info;

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
    DMA2_STREAM7 => dma::InterruptHandler<peripherals::DMA2_CH7>;
    DMA2_STREAM2 => dma::InterruptHandler<peripherals::DMA2_CH2>;
});

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

    let mut driver = Ld2410c::new(usart);
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
            Ok(None) => {}
            Err(e) => info!("UART Error: {:?}", e),
        }
    }
}
