use embassy_nrf::{
    gpio::{Level, Output, OutputDrive},
};
use cortex_m::asm::delay;
use crate::usb_log;

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    let p = unsafe { embassy_nrf::Peripherals::steal() };
    let mut red = Output::new(p.P0_26, Level::High, OutputDrive::Standard);
    let mut green = Output::new(p.P0_30, Level::High, OutputDrive::Standard);
    let mut blue = Output::new(p.P0_06, Level::High, OutputDrive::Standard);
    green.set_high();
    blue.set_high();
    loop {
        red.set_low();
        delay(3_000_000); // ~0.05 s flicker
        red.set_high();
        delay(3_000_000);
    }
}
