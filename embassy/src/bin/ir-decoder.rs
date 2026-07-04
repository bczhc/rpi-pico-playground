#![feature(ascii_char)]
#![no_std]
#![no_main]

use bitvec::prelude::Lsb0;
use bitvec::view::BitView;
use defmt::{panic, unwrap, Format};
use demo1::infrared::*;
use demo1::{current_micro, current_nano};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_time::{Instant, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config};
use log::{info, log};
use utf8_iter::Utf8CharsError;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(unwrap!(logger_task(driver)));

    let pin = Input::new(p.PIN_2, Pull::Up);
    let mut led = Output::new(p.PIN_25, Level::Low);

    let mut decoder = IrDecoder::new();
    let mut bit_counter = 0_usize;
    let mut bit_string = heapless::String::<1024>::new();

    loop {
        let signal = pin.is_low();
        let now = Instant::now().as_micros();
        let event = decoder.tick(signal, now);

        let mut print_and_clear_bits = || {
            if !bit_string.is_empty() {
                info!("{bit_string}");

                // print as text (UTF-8)
                let text_len = (bit_string.len() + 7) / 8;
                let mut text_data = [0_u8; 128];
                let text_bits_view = text_data.view_bits_mut::<Lsb0>();
                for (i, c) in bit_string.chars().enumerate() {
                    match c {
                        '1' => text_bits_view.set(i, true),
                        _ => text_bits_view.set(i, false),
                    };
                }
                let text_data = &text_data[..text_len];
                let utf8 = utf8_iter::ErrorReportingUtf8Chars::new(text_data);
                bit_string.clear();
                for c in utf8 {
                    match c {
                        Ok(c) => _ = bit_string.push(c),
                        Err(_) => _ = bit_string.push(char::REPLACEMENT_CHARACTER),
                    };
                }

                info!("UTF-8 Text: {bit_string}");
                bit_string.clear();
            }
        };

        if let Some(event) = event {
            match event {
                Event::DecodedBit(_) | Event::DecodedUnknown => {
                    let bit_char = if let Event::DecodedBit(b) = event {
                        char::from_digit(u32::from(b), 10).unwrap()
                    } else {
                        '?'
                    };
                    let _r = bit_string.push(bit_char);
                    // if bit_counter >= 8 && bit_counter % 8 == 0 {
                    //     info!(" ");
                    // }
                    bit_counter += 1;
                }
                Event::NewFrameReady => {
                    print_and_clear_bits();
                    info!("---------------------- frame ----------------------");
                    bit_counter = 0;
                }
                Event::DataFragReady => {
                    print_and_clear_bits();
                    bit_counter = 0;
                }
            }
        }

        Timer::after_secs(0).await;
    }
}

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}
