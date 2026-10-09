#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::{Peri, gpio::{AnyPin,Level, Output, OutputDrive}};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

mod tasks;
use tasks::blink_task;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let led_red_port: Peri<'static, AnyPin> = p.P0_26.into();
    let led_green_port: Peri<'static, AnyPin> = p.P0_30.into();
    let led_blue_port: Peri<'static, AnyPin> = p.P0_06.into();
    
    spawner.spawn(blink_task(led_red_port, led_green_port, led_blue_port).expect("blink task"));
}

