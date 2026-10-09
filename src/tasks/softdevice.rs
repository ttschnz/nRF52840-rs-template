use nrf_softdevice::{Softdevice, SocEvent};
use embassy_nrf::usb::vbus_detect::SoftwareVbusDetect;
use crate::usb_log;

#[embassy_executor::task]
pub async fn softdevice_task(sd: &'static Softdevice, vbus: &'static SoftwareVbusDetect) -> ! {
    usb_log!("starting softdevice task");
    sd.run_with_callback(|event| match event {
        SocEvent::PowerUsbDetected => vbus.detected(true),
        SocEvent::PowerUsbRemoved => vbus.detected(false),
        SocEvent::PowerUsbPowerReady => vbus.ready(),
        _ => {}
    }).await
}
