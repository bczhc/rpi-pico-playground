//! This example shows how to use USB (Universal Serial Bus) in the RP2040 chip as well as how to create multiple usb classes for one device
//!
//! This creates a USB serial port that echos. It will also print out logging information on a separate serial device

#![no_std]
#![no_main]

use defmt::{panic, unwrap};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_time::Timer;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config};
use {defmt_rtt as _, panic_probe as _};
use log::{info, log};
use demo1::{current_micro, current_nano};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello there!");

    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);

    spawner.spawn(unwrap!(logger_task(driver)));

    let mut counter = 0_u64;
    let mut start = current_micro();
    loop {
        if current_micro() - start >= 1000_000 {
            info!("Counter: {counter}");
            counter = 0;
            start = current_micro();
            continue;
        }
        counter += 1;
        Timer::after_secs(0).await;
    }
}

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}
