#![no_std]
#![no_main]

use core::future::pending;

use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::{
    Config, bind_interrupts,
    i2c::{self, I2c, SlaveAddrConfig, SlaveCommandKind},
    peripherals,
    time::khz,
};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let config = Config::default();
    let p = embassy_stm32::init(config);
    let i2c_address = 0x20;

    join(
        async {
            let mut i2c_peripheral = I2c::new(p.I2C1, p.PB6, p.PB7, Irqs, p.DMA1_CH6, p.DMA1_CH7, {
                let mut config = i2c::Config::default();
                config.frequency = khz(100);
                config
            })
            .into_slave_multimaster(SlaveAddrConfig::basic(i2c_address));
            loop {
                info!("Ready to receive I2C commands");
                let command = match i2c_peripheral.listen().await {
                    Ok(command) => command,
                    Err(e) => {
                        warn!("I2C error: {}", e);
                        continue;
                    }
                };
                info!("Received I2C command: {}", command);
                match command.kind {
                    SlaveCommandKind::Read => {}
                    SlaveCommandKind::Write => {
                        info!("I2C write command started");
                        let mut buffer = [Default::default(); 512];
                        let bytes_read = i2c_peripheral.respond_to_write(&mut buffer).await.unwrap();
                        info!("Received {} bytes: {}", bytes_read, &buffer[..bytes_read]);
                    }
                }
            }
        },
        async {
            let mut i2c_controller = I2c::new(p.I2C2, p.PB10, p.PB11, Irqs, p.DMA1_CH4, p.DMA1_CH5, {
                let mut config = i2c::Config::default();
                config.frequency = khz(100);
                config
            });
            info!("writing data");
            i2c_controller.write(i2c_address, &[10, 20]).await.unwrap();
            info!("done writing data");
        },
    )
    .await;
    pending().await
}
