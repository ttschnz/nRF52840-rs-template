use embassy_nrf::{
    Peri,
    gpio::{AnyPin, Level, Output, OutputDrive},
};
use embassy_time::Timer;

use crate::usb_log;

const DELAY_DUR: u64 = 200;

#[embassy_executor::task]
pub async fn blink_task(
    r: Peri<'static, AnyPin>,
    g: Peri<'static, AnyPin>,
    b: Peri<'static, AnyPin>,
) {
    let mut led_red = Output::new(r, Level::High, OutputDrive::Standard);
    let mut led_green = Output::new(g, Level::High, OutputDrive::Standard);
    let mut led_blue = Output::new(b, Level::High, OutputDrive::Standard);

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

        usb_log!("blink");
    }
}
