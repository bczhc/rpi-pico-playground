#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_futures::select::{select, Either};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // 1. 直接在 main 里把硬件外设初始化好
    let mut led = Output::new(p.PIN_25, Level::Low);
    let mut pin2 = Output::new(p.PIN_2, Level::Low);

    let fut1 = async {
        loop {
            led.set_high();
            Timer::after_millis(1000).await;
            led.set_low();
            Timer::after_millis(1000).await;
        }
    };
    let fut2 = async {
        loop {
            pin2.set_high();
            Timer::after_millis(100).await;
            pin2.set_low();
            Timer::after_millis(100).await;
        }
    };

    join(fut1, fut2).await;

    loop {
        Timer::after_secs(1).await;
    }
}
