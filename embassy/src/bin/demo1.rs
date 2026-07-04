#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // 1. 直接在 main 里把硬件外设初始化好
    let led = Output::new(p.PIN_25, Level::Low);
    let pin2 = Input::new(p.PIN_2, Pull::Up);

    // 2. 完美的解耦：直接把引脚的所有权传给 Task，不需要任何全局 Mutex！
    spawner.spawn(smart_led_controller(led, pin2).unwrap());
}

#[embassy_executor::task]
async fn smart_led_controller(mut led: Output<'static>, mut pin2: Input<'static>) {
    loop {
        // 如果 PIN2 被按下（Low 状态）
        if pin2.is_low() {
            led.set_high(); // 瞬间点亮！
            pin2.wait_for_high().await; // 挂起 Task，静静等待按键松开，不占任何 CPU
            continue;
        }

        // --- 下面是正常的闪烁逻辑，但引入了 select 来实现“瞬间打断” ---
        led.set_high();
        // 关键点：让“等待 1000ms”和“等待引脚变低”同时竞争！谁先发生就执行谁！
        // 一旦引脚变低，Timer 瞬间被干掉，立刻进入下一次循环去执行上面的 led.set_high()
        if let Either::Second(_) = select(Timer::after_millis(1000), pin2.wait_for_low()).await {
            continue; // 说明被按键瞬间打断了，直接跳到循环开头，立马长亮
        }

        led.set_low();
        if let Either::Second(_) = select(Timer::after_millis(1000), pin2.wait_for_low()).await {
            continue; // 同样，熄灭状态下也能被瞬间打断
        }
    }
}

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}
