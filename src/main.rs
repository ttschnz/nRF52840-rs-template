#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    peripherals,
    Peri, 
    gpio::AnyPin, 
    usb::{self, vbus_detect::SoftwareVbusDetect},
    interrupt::{self, InterruptExt, Priority},
};
use nrf_softdevice::Softdevice;
use static_cell::StaticCell;
use defmt_rtt as _;
use embassy_time::Timer;


mod panic_handler;
mod config;
use config::softdevice_config;
mod tasks;
use tasks::{
    blink_task,     
    usb_log_task,
    softdevice_task,
    usb_log::start_hfxo
};

bind_interrupts!(struct Irqs {
    // USB Driver
    USBD => usb::InterruptHandler<peripherals::USBD>;
});


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut hal_config = embassy_nrf::config::Config::default();
    hal_config.gpiote_interrupt_priority = Priority::P2;
    hal_config.time_interrupt_priority = Priority::P2;
    let p = embassy_nrf::init(hal_config);
    
    // SoftDevice config & task
    let sd = Softdevice::enable(&softdevice_config());
    interrupt::USBD.set_priority(Priority::P2);

    let mut status = 0u32;
    unsafe { nrf_softdevice::raw::sd_power_usbregstatus_get(&mut status) };
    let detected = status & 0b01 != 0; // USBREGSTATUS.VBUSDETECT
    let ready    = status & 0b10 != 0; // USBREGSTATUS.OUTPUTRDY

    static VBUS: StaticCell<SoftwareVbusDetect> = StaticCell::new();
    let vbus: &'static SoftwareVbusDetect = VBUS.init(SoftwareVbusDetect::new(detected, ready));
    spawner.spawn(softdevice_task(sd, vbus).expect("softdevice task"));
    
    // Start high-frequency crystal for USB
    start_hfxo().await;
    
    // USB Serial Logging
    let usb_driver = usb::Driver::new(p.USBD, Irqs, vbus);
    spawner.spawn(usb_log_task(usb_driver).expect("usb_log task"));
    usb_log!("logging started");

    // Blink Definitions
    let led_red_port: Peri<'static, AnyPin> = p.P0_26.into();
    let led_green_port: Peri<'static, AnyPin> = p.P0_30.into();
    let led_blue_port: Peri<'static, AnyPin> = p.P0_06.into();
    spawner.spawn(blink_task(led_red_port, led_green_port, led_blue_port).expect("blink task"));
    usb_log!("blink-task started");
}

