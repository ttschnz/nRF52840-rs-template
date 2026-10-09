use core::{fmt::Write as _};

use embassy_futures::join::join;
use embassy_nrf::usb::{Driver, vbus_detect::SoftwareVbusDetect};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_usb::{
    Builder, Config,
    class::cdc_acm::{CdcAcmClass, State},
    driver::EndpointError,
};
use embassy_time::Timer;
use heapless::String;


/// Longest message in bytes. Longer messages are truncated.
pub const LINE_LEN: usize = 128;
/// How many messages can wait for the USB task. If the queue is full
/// (e.g. no terminal open), new messages are dropped.
const QUEUE_LEN: usize = 16;
/// Full-speed bulk endpoint size.
const MAX_PACKET: usize = 64;

pub static LOG: Channel<CriticalSectionRawMutex, String<LINE_LEN>, QUEUE_LEN> = Channel::new();

/// Driver type when VBUS is reported by the SoftDevice (see softdevice_task).
pub type UsbDriver = Driver<'static, &'static SoftwareVbusDetect>;

/// Non-blocking. Safe to call from any task or from sync code.
pub fn log_args(args: core::fmt::Arguments<'_>) {
    let mut s = String::<LINE_LEN>::new();
    let _ = s.write_fmt(args); // too long -> truncated
    let _ = LOG.try_send(s); // queue full -> dropped
}

#[macro_export]
macro_rules! usb_log {
    ($($arg:tt)*) => {
        $crate::tasks::usb_log::log_args(format_args!($($arg)*))
    };
}

async fn send<'d, D: embassy_usb::driver::Driver<'d>>(
    class: &mut CdcAcmClass<'d, D>,
    data: &[u8],
) -> Result<(), EndpointError> {
    for chunk in data.chunks(MAX_PACKET) {
        class.write_packet(chunk).await?;
    }
    Ok(())
}

/// Forwards queued lines until the host disconnects.
async fn pump<'d, D: embassy_usb::driver::Driver<'d>>(
    class: &mut CdcAcmClass<'d, D>,
) -> Result<(), EndpointError> {
    loop {
        let line = LOG.receive().await;
        send(class, line.as_bytes()).await?;
        send(class, b"\r\n").await?;
    }
}

pub async fn start_hfxo() {
    unsafe {
        nrf_softdevice::raw::sd_clock_hfclk_request();
        let mut running = 0u32;
        while running == 0 {
            nrf_softdevice::raw::sd_clock_hfclk_is_running(&mut running);
            Timer::after_millis(1).await;
        }
    }
}

#[embassy_executor::task]
pub async fn usb_log_task(driver: UsbDriver) {
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("nRF debug");
    config.product = Some("USB log");
    config.serial_number = Some("0001");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // These live inside the task's future, so they are 'static once spawned.
    let mut config_descriptor = [0u8; 256];
    let mut bos_descriptor = [0u8; 256];
    let mut control_buf = [0u8; 64];
    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no MS OS descriptors
        &mut control_buf,
    );
    let mut class = CdcAcmClass::new(&mut builder, &mut state, MAX_PACKET as u16);
    let mut usb = builder.build();

    let usb_fut = usb.run();
    let log_fut = async {
        loop {
            class.wait_connection().await;
            let _ = pump(&mut class).await; // Err = unplugged, wait again
        }
    };
    join(usb_fut, log_fut).await;
}
