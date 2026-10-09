mod blink;
pub use blink::blink_task;

pub mod softdevice;
pub use softdevice::softdevice_task;

pub mod usb_log;
pub use usb_log::usb_log_task;
