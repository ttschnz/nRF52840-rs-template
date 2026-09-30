#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use defmt::info;

const DELAY_DUR:u64 = 200;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led_red = Output::new(p.P0_26, Level::High, OutputDrive::Standard);
    let mut led_green = Output::new(p.P0_30, Level::High, OutputDrive::Standard);
    let mut led_blue = Output::new(p.P0_06, Level::High, OutputDrive::Standard);
    
    info!("Starting...");
    
    led_red.set_high();
    led_green.set_high();
    led_blue.set_high();

    loop {
        led_red.set_low();
        Timer::after_millis(DELAY_DUR).await;
        led_red.set_high();

        led_green.set_low();
        Timer::after_millis(DELAY_DUR).await;
        led_green.set_high();

        led_blue.set_low();
        Timer::after_millis(DELAY_DUR).await;
        led_blue.set_high();
    }
}

