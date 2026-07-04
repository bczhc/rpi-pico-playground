#![feature(ascii_char)]
#![no_std]
#![no_main]

use bitvec::prelude::{BitSlice, Lsb0};
use bitvec::view::BitView;
use defmt::{panic, unwrap};
use demo1::infrared::*;
use demo1::{current_micro, current_nano};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::pwm::Pwm;
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_rp::{bind_interrupts, pwm};
use embassy_time::{Instant, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config};
use log::{info, log};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

static AUDIO_DATA: &[u8] = include_bytes!("/home/bczhc/t/2026-07-04 11:39:33+08:00/a.bin");

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(unwrap!(logger_task(driver)));

    let pin = Input::new(p.PIN_2, Pull::Up);
    let mut led = Output::new(p.PIN_25, Level::Low);

    let mut pwm_config = pwm::Config::default();
    pwm_config.divider = (16).into();
    pwm_config.top = 205;
    pwm_config.compare_a = 205 / 4;

    let mut pwm = Pwm::new_output_a(
        p.PWM_SLICE7,
        p.PIN_14,
        pwm::Config::default(),
    );
    pwm.set_config(&pwm_config);
    pwm_config.enable = true;
    pwm.set_config(&pwm_config);

    let mut pwm_set = |on: bool| {
        pwm_config.enable = on;
        pwm.set_config(&pwm_config);
    };

    /*const FREQ: u32 = 3000;

    let freq = FREQ;
    let alter_interval = 1_000_000 / (freq * 2);
    let high_time = alter_interval as u64;
    let period_end_at = alter_interval as u64 * 2;

    let mut play = true;
    loop {
        if play {
            // 1s
            let timer_start = Instant::now();
            let mut start = Instant::now();
            loop {
                let elapsed = start.elapsed().as_micros();

                if elapsed >= 0 && elapsed < high_time {
                    pwm_set(true);
                }
                if elapsed >= high_time && elapsed < period_end_at {
                    pwm_set(false);
                }
                if elapsed >= period_end_at {
                    start = Instant::now();
                    continue;
                }

                if timer_start.elapsed().as_micros() > 1_000_000 {
                    play = false;
                    pwm_set(false);
                    break;
                }
            }
        } else {
            Timer::after_secs(1).await;
            play = true;
        }
    }*/

    const REST_SAMPLES_MIN: usize = 16;
    struct RestDetector {
        rest: bool,
        rest_sample_counter: usize,
    }

    impl RestDetector {
        fn tick(&mut self, sample: bool) {
            if !sample {
                self.rest_sample_counter += 1;
            } else {
                self.rest_sample_counter = 0;
                self.rest = false;
            }
            if self.rest_sample_counter >= REST_SAMPLES_MIN {
                self.rest = true;
            }
        }

        fn new() -> Self {
            Self {
                rest: true,
                rest_sample_counter: 0,
            }
        }
    }

    let sample_rate = 2000_u32;
    let sample_duration = (1_000_000 / sample_rate) as u64;

    let mut sample_n = 0_usize;
    let bit_view = AUDIO_DATA.view_bits::<Lsb0>();
    let mut rest_detector = RestDetector::new();
    let mut last_active_start: Option<u64> = None;
    loop {
        if sample_n >= bit_view.len() {
            sample_n = 0;
        }

        let sample = bit_view[sample_n];
        rest_detector.tick(sample);

        if sample {
            pwm_set(true);
            Timer::after_micros(sample_duration).await;
        } else {
            pwm_set(false);
            Timer::after_micros(sample_duration).await;
        }

        if !rest_detector.rest {
            if last_active_start.is_none() {
                last_active_start = Some(current_micro());
            }
            if current_micro() - last_active_start.unwrap() > 1000_000 {
                // do a hard sleep to let the ir receiver have a rest due to its "anti-interference" quirk
                pwm_set(false);
                Timer::after_millis(100).await;
                last_active_start = None;
            }
        } else {
            last_active_start = None;
        }

        sample_n += 1;
    }
}

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}
