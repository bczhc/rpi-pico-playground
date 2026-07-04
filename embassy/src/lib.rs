#![no_std]
#![feature(decl_macro)]

pub mod infrared;

#[inline(always)]
pub fn current_nano() -> u64 {
    embassy_time::Instant::now().as_nanos()
}

#[inline(always)]
pub fn current_micro() -> u64 {
    embassy_time::Instant::now().as_micros()
}

/// Spin delay.
pub macro delay_ms($m:expr) {{
    let start = current_nano();
    loop {
        if current_nano() - start >= $m * 1_000_000 {
            break;
        }
    }
}}
